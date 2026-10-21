//! IPC commands. Only our own local pages may call them: the strip and the
//! account editor (see `capabilities/`). WhatsApp's pages get no IPC access at all.
//!
//! Commands are `async` so Tauri runs them off the UI thread. Creating
//! webviews or dialogs from a synchronous command can deadlock on Windows.

use std::sync::Arc;

use firehop_core::application::{AccountService, AppError, ProfileInput, RosterView};
use firehop_core::domain::AccountId;
use serde::{Deserialize, Serialize, Serializer};
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use crate::{editor, platform};

type Service<'a> = State<'a, Arc<AccountService>>;

/// Everything a command can fail with, handed to the UI as a plain message.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    App(#[from] AppError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("{0:?} is not a valid account id")]
    MalformedId(String),
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

type Outcome<T = ()> = Result<T, CommandError>;

fn parse_id(raw: &str) -> Result<AccountId, CommandError> {
    raw.parse()
        .map_err(|_| CommandError::MalformedId(raw.to_owned()))
}

/// The editor's form, mirroring `ProfileForm` in `ui/model.ts`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileForm {
    name: String,
    full_name: String,
    accent: u8,
}

#[tauri::command]
pub async fn roster(service: Service<'_>) -> Outcome<RosterView> {
    Ok(service.view())
}

#[tauri::command]
pub async fn add_account(service: Service<'_>) -> Outcome {
    Ok(service.add()?)
}

#[tauri::command]
pub async fn activate_account(service: Service<'_>, id: String) -> Outcome {
    Ok(service.activate(parse_id(&id)?)?)
}

/// The system accent colour (`#rrggbb`), so the strip can draw the taskbar
/// badge the way Windows draws app badges. Web content cannot read it.
#[tauri::command]
pub async fn accent_color() -> Outcome<Option<String>> {
    Ok(platform::accent_color())
}

/// Moves an account to `index` in the strip (drag and drop, Alt+↑/↓, or the menu).
#[tauri::command]
pub async fn move_account(service: Service<'_>, id: String, index: usize) -> Outcome {
    Ok(service.move_account(parse_id(&id)?, index)?)
}

#[tauri::command]
pub async fn reload_account(service: Service<'_>, id: String) -> Outcome {
    Ok(service.reload(parse_id(&id)?)?)
}

#[tauri::command]
pub async fn remove_account<R: Runtime>(
    app: AppHandle<R>,
    service: Service<'_>,
    id: String,
) -> Outcome {
    service.remove(parse_id(&id)?)?;
    editor::close(&app);
    Ok(())
}

#[tauri::command]
pub async fn open_account_editor<R: Runtime>(
    app: AppHandle<R>,
    service: Service<'_>,
    id: String,
) -> Outcome {
    let id = parse_id(&id)?;
    service.ensure_exists(id)?;
    Ok(editor::open(&app, id)?)
}

#[tauri::command]
pub async fn update_account(service: Service<'_>, id: String, form: ProfileForm) -> Outcome {
    let input = ProfileInput {
        name: form.name,
        full_name: form.full_name,
        accent: form.accent,
    };
    Ok(service.update_profile(parse_id(&id)?, &input)?)
}

/// Shows the native file picker and imports the chosen image. Picking happens
/// here rather than in the page, so the UI never hands us arbitrary file paths.
/// Returns `false` when the user cancels.
#[tauri::command]
pub async fn choose_account_photo<R: Runtime>(
    app: AppHandle<R>,
    service: Service<'_>,
    id: String,
) -> Outcome<bool> {
    let id = parse_id(&id)?;
    let picked = app
        .dialog()
        .file()
        .set_title("Choose a photo")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
        .blocking_pick_file();
    let Some(path) = picked.and_then(|file| file.into_path().ok()) else {
        return Ok(false);
    };
    service.set_photo(id, &path)?;
    Ok(true)
}

#[tauri::command]
pub async fn clear_account_photo(service: Service<'_>, id: String) -> Outcome {
    Ok(service.clear_photo(parse_id(&id)?)?)
}
