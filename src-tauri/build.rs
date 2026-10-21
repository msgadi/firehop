fn main() {
    // Declaring the app's commands generates an `allow-<command>` permission for
    // each one. Capabilities then grant them to specific webviews only.
    let commands = &[
        "roster",
        "add_account",
        "activate_account",
        "move_account",
        "accent_color",
        "open_account_editor",
        "update_account",
        "choose_account_photo",
        "clear_account_photo",
        "reload_account",
        "remove_account",
    ];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(commands)),
    )
    .expect("failed to run tauri-build");
}
