use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{AccountId, Roster};

/// Everything the strip and the account editor need to render, in one serialisable snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterView {
    pub accounts: Vec<AccountView>,
    pub active_id: Option<String>,
    pub can_add: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub full_name: Option<String>,
    pub initials: String,
    pub accent: u8,
    /// The custom avatar as a `data:image/png` URL, if one is set.
    pub photo: Option<String>,
    pub unread: u32,
}

/// Runtime data that lives next to the roster but is never persisted in it.
#[derive(Default)]
pub struct Extras {
    pub unread: HashMap<AccountId, u32>,
    /// Avatar data URLs, cached so each update does not re-read the files.
    pub photos: HashMap<AccountId, String>,
}

impl RosterView {
    pub fn build(roster: &Roster, extras: &Extras) -> Self {
        let accounts = roster
            .accounts()
            .iter()
            .map(|account| AccountView {
                id: account.id().to_string(),
                name: account.name().as_str().to_owned(),
                full_name: account.full_name().map(|n| n.as_str().to_owned()),
                initials: account.name().initials(),
                accent: account.accent().index(),
                photo: extras.photos.get(&account.id()).cloned(),
                unread: extras.unread.get(&account.id()).copied().unwrap_or(0),
            })
            .collect();
        Self {
            accounts,
            active_id: roster.active().map(|id| id.to_string()),
            can_add: !roster.is_full(),
        }
    }

    pub fn total_unread(&self) -> u32 {
        self.accounts.iter().map(|a| a.unread).sum()
    }
}
