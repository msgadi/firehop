use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{fs, thread};

use tauri::webview::{NewWindowResponse, PermissionKind, PermissionResponse, WebviewBuilder};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewUrl, Window,
};
use tauri_plugin_opener::OpenerExt;
use url::Url;

use firehop_core::application::{SessionError, Sessions};
use firehop_core::domain::AccountId;
use firehop_core::layout::{self, Rect};
use firehop_core::policy::{self, Capability, WHATSAPP_WEB};

use crate::platform::{self, Priority};
use crate::{MAIN_WINDOW, STRIP_WEBVIEW};

const ACCOUNT_LABEL_PREFIX: &str = "account-";

/// Background behind a session while WhatsApp Web paints its first frame, so
/// switching or opening never flashes white. Matches WhatsApp's dark canvas.
const LOADING_BACKGROUND: tauri::window::Color = tauri::window::Color(0x16, 0x17, 0x17, 0xff);

/// Chromium switches for every session.
/// - The `--disable-features` list keeps wry's defaults, which a custom value would otherwise drop.
/// - The `--js-flags` and memory-pressure switches are the ones WhatsApp's own
///   Windows client passes to WebView2 to keep background sessions small
///   (see docs/research/whatsapp-desktop-teardown.md).
const BROWSER_ARGS: &str = concat!(
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection ",
    "--js-flags=--scavenger_max_new_space_capacity_mb=8 ",
    "--msWebView2SimulateMemoryPressureWhenInactive=1",
);

/// Something a running session reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    TitleChanged { id: AccountId, title: String },
}

/// [`Sessions`] backed by one child webview per account inside the main window.
///
/// Each webview gets its own WebView2 user-data folder, so accounts share no
/// cookies, storage or service workers. They behave like separate browsers.
pub struct WebviewSessions<R: Runtime> {
    app: AppHandle<R>,
    profiles_root: PathBuf,
    events: Sender<SessionEvent>,
}

impl<R: Runtime> WebviewSessions<R> {
    pub fn new(app: AppHandle<R>, profiles_root: PathBuf, events: Sender<SessionEvent>) -> Self {
        Self {
            app,
            profiles_root,
            events,
        }
    }

    fn window(&self) -> Result<Window<R>, SessionError> {
        self.app
            .get_window(MAIN_WINDOW)
            .ok_or_else(|| SessionError("main window is gone".into()))
    }

    fn webview(&self, id: AccountId) -> Option<Webview<R>> {
        self.app.get_webview(&label_for(id))
    }

    fn profile_dir(&self, id: AccountId) -> PathBuf {
        self.profiles_root.join(id.to_string())
    }

    fn builder(&self, id: AccountId) -> WebviewBuilder<R> {
        let home = Url::parse(WHATSAPP_WEB).expect("WHATSAPP_WEB is a valid URL");
        let nav_app = self.app.clone();
        let popup_app = self.app.clone();
        let events = self.events.clone();

        WebviewBuilder::new(label_for(id), WebviewUrl::External(home))
            .data_directory(self.profile_dir(id))
            .additional_browser_args(BROWSER_ARGS)
            .background_color(LOADING_BACKGROUND)
            .zoom_hotkeys_enabled(true)
            .on_navigation(move |url| {
                let stay = policy::may_navigate_in_app(url);
                if !stay {
                    open_externally(&nav_app, url);
                }
                stay
            })
            .on_new_window(move |url, _features| {
                // WhatsApp's own pop-ups (e.g. a call window) keep the session's
                // profile. Any other link goes to the user's browser.
                if policy::may_navigate_in_app(&url) {
                    NewWindowResponse::Allow
                } else {
                    open_externally(&popup_app, &url);
                    NewWindowResponse::Deny
                }
            })
            .on_permission_request(|_, kind| {
                if policy::grants(capability_of(kind)) {
                    PermissionResponse::Allow
                } else {
                    PermissionResponse::Default
                }
            })
            .on_document_title_changed(move |_, title| {
                // The receiver only goes away during shutdown, when nobody needs the count.
                let _ = events.send(SessionEvent::TitleChanged { id, title });
            })
    }
}

impl<R: Runtime> Sessions for WebviewSessions<R> {
    fn open(&self, id: AccountId) -> Result<(), SessionError> {
        if self.webview(id).is_some() {
            return Ok(());
        }
        let window = self.window()?;
        let area = content_area(&window)?;
        let webview = window
            .add_child(self.builder(id), position(area), size(area))
            .map_err(|err| SessionError(format!("cannot start session {id}: {err}")))?;
        webview.hide().map_err(session_error)?;
        platform::set_priority(&webview, Priority::Background);
        Ok(())
    }

    fn focus(&self, id: AccountId) -> Result<(), SessionError> {
        let target = self
            .webview(id)
            .ok_or_else(|| SessionError(format!("session {id} is not running")))?;
        // Show the new one before hiding the rest, so there is never an empty frame.
        target.show().map_err(session_error)?;
        platform::set_priority(&target, Priority::Foreground);
        for other in self.window()?.webviews() {
            if account_of(other.label()).is_some_and(|other_id| other_id != id) {
                other.hide().map_err(session_error)?;
                platform::set_priority(&other, Priority::Background);
            }
        }
        target.set_focus().map_err(session_error)
    }

    fn reload(&self, id: AccountId) -> Result<(), SessionError> {
        let webview = self
            .webview(id)
            .ok_or_else(|| SessionError(format!("session {id} is not running")))?;
        webview.reload().map_err(session_error)
    }

    fn destroy(&self, id: AccountId) -> Result<(), SessionError> {
        if let Some(webview) = self.webview(id) {
            webview.close().map_err(session_error)?;
        }
        erase_profile_eventually(self.profile_dir(id));
        Ok(())
    }
}

/// Lays out the strip and every session after the window changes size.
pub fn relayout<R: Runtime>(window: &Window<R>) {
    let Ok(scale) = window.scale_factor() else {
        return;
    };
    let Ok(inner) = window.inner_size() else {
        return;
    };
    let logical = inner.to_logical::<f64>(scale);
    let areas = layout::split(logical.width, logical.height);
    for webview in window.webviews() {
        let area = if webview.label() == STRIP_WEBVIEW {
            areas.strip
        } else {
            areas.content
        };
        if let Err(err) = webview
            .set_position(position(area))
            .and_then(|()| webview.set_size(size(area)))
        {
            log::warn!("could not lay out webview {}: {err}", webview.label());
        }
    }
}

pub fn strip_area<R: Runtime>(window: &Window<R>) -> tauri::Result<Rect> {
    areas(window).map(|a| a.strip)
}

fn content_area<R: Runtime>(window: &Window<R>) -> Result<Rect, SessionError> {
    areas(window).map(|a| a.content).map_err(session_error)
}

fn areas<R: Runtime>(window: &Window<R>) -> tauri::Result<layout::Layout> {
    let logical = window
        .inner_size()?
        .to_logical::<f64>(window.scale_factor()?);
    Ok(layout::split(logical.width, logical.height))
}

pub fn position(area: Rect) -> LogicalPosition<f64> {
    LogicalPosition::new(area.x, area.y)
}

pub fn size(area: Rect) -> LogicalSize<f64> {
    LogicalSize::new(area.width, area.height)
}

fn label_for(id: AccountId) -> String {
    format!("{ACCOUNT_LABEL_PREFIX}{id}")
}

fn account_of(label: &str) -> Option<AccountId> {
    label.strip_prefix(ACCOUNT_LABEL_PREFIX)?.parse().ok()
}

fn capability_of(kind: PermissionKind) -> Capability {
    match kind {
        PermissionKind::Microphone => Capability::Microphone,
        PermissionKind::Camera => Capability::Camera,
        PermissionKind::Notifications => Capability::Notifications,
        PermissionKind::ClipboardRead => Capability::ClipboardRead,
        _ => Capability::Other,
    }
}

fn open_externally<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    if !policy::may_open_externally(url) {
        log::warn!("blocked link with disallowed scheme {:?}", url.scheme());
        return;
    }
    if let Err(err) = app.opener().open_url(url.as_str(), None::<&str>) {
        log::warn!("could not open link in the default browser: {err}");
    }
}

/// WebView2 browser processes keep the profile locked for a moment after the
/// webview closes, so keep retrying the delete in the background for a while.
fn erase_profile_eventually(dir: PathBuf) {
    const ATTEMPTS: u32 = 20;
    const PAUSE: Duration = Duration::from_millis(500);
    thread::spawn(move || {
        for _ in 0..ATTEMPTS {
            match fs::remove_dir_all(&dir) {
                Ok(()) => return,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
                Err(_) => thread::sleep(PAUSE),
            }
        }
        log::error!("gave up erasing {}; delete it manually", dir.display());
    });
}

fn session_error(err: impl std::fmt::Display) -> SessionError {
    SessionError(err.to_string())
}
