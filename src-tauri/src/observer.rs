use firehop_core::application::{RosterObserver, RosterView};
use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime};

use crate::{APP_NAME, EDITOR_WINDOW, MAIN_WINDOW, STRIP_WEBVIEW};

/// Event the strip and editor listen to for roster updates. Must match `ui/backend.ts`.
pub const ROSTER_CHANGED: &str = "roster-changed";

/// Pushes roster updates to the strip and puts the total unread count in the
/// window title, which is also what the taskbar shows.
pub struct StripObserver<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> StripObserver<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> RosterObserver for StripObserver<R> {
    fn roster_changed(&self, view: &RosterView) {
        // Our own two pages only; WhatsApp's webviews never receive app events.
        for page in [STRIP_WEBVIEW, EDITOR_WINDOW] {
            if let Err(err) = self
                .app
                .emit_to(EventTarget::webview(page), ROSTER_CHANGED, view)
            {
                log::warn!("could not send the roster to {page}: {err}");
            }
        }
        if let Some(window) = self.app.get_window(MAIN_WINDOW) {
            let title = match view.total_unread() {
                0 => APP_NAME.to_owned(),
                unread => format!("({unread}) {APP_NAME}"),
            };
            if let Err(err) = window.set_title(&title) {
                log::debug!("could not update the window title: {err}");
            }
        }
    }
}
