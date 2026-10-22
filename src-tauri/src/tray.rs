//! The notification-area (tray) icon. Closing the main window only hides it,
//! so messages and calls keep arriving; the tray menu is the way to quit.

use tauri::menu::{MenuBuilder, MenuEvent};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Runtime};

use crate::{APP_NAME, MAIN_WINDOW};

const TRAY: &str = "main";
const OPEN: &str = "open";
const QUIT: &str = "quit";

/// Adds the tray icon with its "Open" / "Quit" menu.
pub fn install<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text(OPEN, format!("Open {APP_NAME}"))
        .separator()
        .text(QUIT, format!("Quit {APP_NAME}"))
        .build()?;

    let mut tray = TrayIconBuilder::with_id(TRAY)
        .tooltip(APP_NAME)
        .menu(&menu)
        // Left click opens the window, like most tray apps; right click shows the menu.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, &event))
        .on_tray_icon_event(|tray, event| on_click(tray, &event));
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Brings the main window back from the tray, the taskbar or behind other windows.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_window(MAIN_WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Puts the total unread count in the tray tooltip, since a hidden window has
/// no taskbar button to carry the badge.
pub fn show_unread<R: Runtime>(app: &AppHandle<R>, unread: u32) {
    let Some(tray) = app.tray_by_id(TRAY) else {
        return;
    };
    let tooltip = match unread {
        0 => APP_NAME.to_owned(),
        unread => format!("{APP_NAME} · {unread} unread"),
    };
    if let Err(err) = tray.set_tooltip(Some(tooltip)) {
        log::debug!("could not update the tray tooltip: {err}");
    }
}

fn on_menu<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    match event.id().as_ref() {
        OPEN => show_main(app),
        QUIT => app.exit(0),
        _ => {}
    }
}

fn on_click<R: Runtime>(tray: &TrayIcon<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = *event
    {
        show_main(tray.app_handle());
    }
}
