//! The account editor: a small window owned by the main window, so it stays
//! on top of it and closes with it.

use firehop_core::domain::AccountId;
use tauri::webview::WebviewBuilder;
use tauri::window::WindowBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl};

use crate::{EDITOR_BACKGROUND, EDITOR_WINDOW, MAIN_WINDOW};

const WIDTH: f64 = 440.0;
const HEIGHT: f64 = 600.0;

/// Opens the editor for one account, replacing any editor already open.
pub fn open<R: Runtime>(app: &AppHandle<R>, id: AccountId) -> tauri::Result<()> {
    if let Some(existing) = app.get_window(EDITOR_WINDOW) {
        existing.destroy()?;
    }
    let main = app
        .get_window(MAIN_WINDOW)
        .ok_or(tauri::Error::WindowNotFound)?;

    let window = WindowBuilder::new(app, EDITOR_WINDOW)
        .title("Edit account")
        .inner_size(WIDTH, HEIGHT)
        .position_centered_over(&main)?
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .owner(&main)?
        .build()?;

    window.add_child(
        WebviewBuilder::new(
            EDITOR_WINDOW,
            WebviewUrl::App(format!("editor.html?id={id}").into()),
        )
        .background_color(EDITOR_BACKGROUND),
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(WIDTH, HEIGHT),
    )?;
    Ok(())
}

/// Closes the editor if it is open (e.g. after its account was removed).
pub fn close<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_window(EDITOR_WINDOW) {
        let _ = window.destroy();
    }
}

trait CenteredOver<R: Runtime>: Sized {
    fn position_centered_over(self, owner: &tauri::Window<R>) -> tauri::Result<Self>;
}

impl<R: Runtime, M: Manager<R>> CenteredOver<R> for WindowBuilder<'_, R, M> {
    /// Places the window in the middle of `owner` rather than of the screen,
    /// so it opens where the user is looking.
    fn position_centered_over(self, owner: &tauri::Window<R>) -> tauri::Result<Self> {
        let scale = owner.scale_factor()?;
        let origin = owner.outer_position()?.to_logical::<f64>(scale);
        let size = owner.outer_size()?.to_logical::<f64>(scale);
        Ok(self.position(
            origin.x + (size.width - WIDTH) / 2.0,
            origin.y + (size.height - HEIGHT) / 2.0,
        ))
    }
}
