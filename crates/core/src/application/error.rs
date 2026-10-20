use serde::{Serialize, Serializer};

use crate::domain::RosterError;

use super::ports::{AvatarError, SessionError, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Roster(#[from] RosterError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Avatar(#[from] AvatarError),
    #[error("{0:?} is not a valid account id")]
    MalformedId(String),
}

/// Commands hand errors to the UI as plain messages.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
