//! Interfaces the application core needs from the outside world.
//!
//! The core depends only on these traits; `infrastructure` and `interface`
//! provide the real implementations.

use std::path::Path;

use crate::domain::{AccountId, Roster};

use super::view::RosterView;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AvatarError(pub String);

/// Custom account photos. Images are normalised on import (decoded, cropped
/// square, downscaled, re-encoded) and handed back as `data:` URLs.
pub trait AvatarStore: Send + Sync {
    /// Imports the image file at `source` as the account's avatar and returns its data URL.
    fn import(&self, id: AccountId, source: &Path) -> Result<String, AvatarError>;
    /// The stored avatar's data URL, or `None` when the account has none.
    fn load(&self, id: AccountId) -> Result<Option<String>, AvatarError>;
    /// Deletes the stored avatar. Succeeds if there was none.
    fn remove(&self, id: AccountId) -> Result<(), AvatarError>;
}

#[derive(Debug, thiserror::Error)]
#[error("could not persist accounts: {0}")]
pub struct StoreError(pub String);

/// Durable storage for the roster.
pub trait RosterStore: Send + Sync {
    /// Returns an empty roster when nothing has been saved yet.
    fn load(&self) -> Result<Roster, StoreError>;
    fn save(&self, roster: &Roster) -> Result<(), StoreError>;
}

#[derive(Debug, thiserror::Error)]
#[error("account session failed: {0}")]
pub struct SessionError(pub String);

/// The running WhatsApp Web instances, one isolated browser profile per account.
pub trait Sessions: Send + Sync {
    /// Starts the account's session in the background. Does nothing if it is already running.
    fn open(&self, id: AccountId) -> Result<(), SessionError>;
    /// Brings one session on screen and moves every other session to the background.
    fn focus(&self, id: AccountId) -> Result<(), SessionError>;
    fn reload(&self, id: AccountId) -> Result<(), SessionError>;
    /// Stops the session and erases its browser profile, which logs that device out.
    fn destroy(&self, id: AccountId) -> Result<(), SessionError>;
}

/// Receives a fresh view whenever anything the account strip shows has changed.
pub trait RosterObserver: Send + Sync {
    fn roster_changed(&self, view: &RosterView);
}
