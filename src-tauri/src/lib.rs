//! Firehop's Tauri shell: the window, the webviews and the WebView2 glue.
//!
//! The rules and use cases live in `firehop-core`. This crate only
//! implements its ports and wires everything together:
//! - [`sessions`]: one isolated WhatsApp Web webview per account ([`Sessions`] port);
//! - [`observer`]: pushes roster updates to the strip ([`RosterObserver`] port);
//! - [`commands`]: the IPC surface the strip and the account editor call;
//! - [`editor`]: the account editor window;
//! - [`platform`]: the few raw WebView2 calls Tauri does not expose;
//! - [`tray`]: the tray icon, which keeps Firehop running after the window closes;
//! - [`run`] / `compose`: the composition root.
//!
//! [`Sessions`]: firehop_core::application::Sessions
//! [`RosterObserver`]: firehop_core::application::RosterObserver

mod commands;
mod editor;
mod observer;
mod platform;
mod sessions;
mod tray;

use std::sync::{Arc, mpsc};
use std::thread;

use firehop_core::application::{AccountService, Ports};
use firehop_core::avatars::FsAvatarStore;
use firehop_core::storage::JsonRosterStore;
use tauri::webview::WebviewBuilder;
use tauri::window::{Color, WindowBuilder};
use tauri::{App, Manager, WebviewUrl, WindowEvent};
use tauri_plugin_window_state::StateFlags;

use observer::StripObserver;
use sessions::{SessionEvent, WebviewSessions};

pub(crate) const APP_NAME: &str = "Firehop";
pub(crate) const MAIN_WINDOW: &str = "main";
pub(crate) const STRIP_WEBVIEW: &str = "strip";
/// Label of the account editor window and of its webview.
pub(crate) const EDITOR_WINDOW: &str = "editor";

const ROSTER_FILE: &str = "roster.json";
const PROFILES_DIR: &str = "profiles";
const AVATARS_DIR: &str = "avatars";
/// Matches the strip's dark background, so it never flashes while loading.
const STRIP_BACKGROUND: Color = Color(0x1d, 0x1f, 0x1f, 0xff);
/// Matches the editor page background.
pub(crate) const EDITOR_BACKGROUND: Color = Color(0x16, 0x17, 0x17, 0xff);

/// Starts the app.
///
/// # Panics
///
/// If the event loop or the main window cannot be created.
pub fn run() {
    tauri::Builder::default()
        // Registered first, so a second launch focuses this instance before anything else starts.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                // Quitting from the tray while hidden must not make the next launch start hidden.
                .with_state_flags(StateFlags::all() - StateFlags::VISIBLE)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::roster,
            commands::add_account,
            commands::activate_account,
            commands::move_account,
            commands::accent_color,
            commands::reload_account,
            commands::remove_account,
            commands::open_account_editor,
            commands::update_account,
            commands::choose_account_photo,
            commands::clear_account_photo,
        ])
        .setup(|app| {
            compose(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start the application");
}

/// Composition root: builds the window and the adapters, injects them into the
/// service, and starts the sessions.
fn compose(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;

    let window = WindowBuilder::new(app, MAIN_WINDOW)
        .title(APP_NAME)
        .inner_size(1280.0, 820.0)
        .min_inner_size(760.0, 520.0)
        .build()?;

    let strip = sessions::strip_area(&window)?;
    window.add_child(
        WebviewBuilder::new(STRIP_WEBVIEW, WebviewUrl::App("index.html".into()))
            .background_color(STRIP_BACKGROUND),
        sessions::position(strip),
        sessions::size(strip),
    )?;

    let (events_tx, events_rx) = mpsc::channel();
    let service = Arc::new(AccountService::new(Ports {
        store: Arc::new(JsonRosterStore::new(data_dir.join(ROSTER_FILE))),
        avatars: Arc::new(FsAvatarStore::new(data_dir.join(AVATARS_DIR))),
        sessions: Arc::new(WebviewSessions::new(
            app.handle().clone(),
            data_dir.join(PROFILES_DIR),
            events_tx,
        )),
        observer: Arc::new(StripObserver::new(app.handle().clone())),
    }));
    app.manage(Arc::clone(&service));

    tray::install(app)?;

    let target = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            sessions::relayout(&target);
        }
        // Closing hides to the tray, like WhatsApp Desktop keeps running, so
        // messages and calls still arrive. "Quit" in the tray menu exits.
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            editor::close(target.app_handle());
            if let Err(err) = target.hide() {
                log::warn!("could not hide the window: {err}");
            }
        }
        _ => {}
    });

    // Session events arrive on the UI thread. Hand them to the service on a
    // worker thread, so the UI thread never waits on the service lock.
    let listener = Arc::clone(&service);
    thread::spawn(move || {
        for event in events_rx {
            match event {
                SessionEvent::TitleChanged { id, title } => listener.title_changed(id, &title),
            }
        }
    });

    service.start()?;
    Ok(())
}
