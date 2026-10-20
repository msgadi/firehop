use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable identity of a linked account.
///
/// It also names the account's isolated browser-profile directory and its
/// avatar file, so it must never change once created.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountId(Uuid);

impl AccountId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.simple())
    }
}

impl FromStr for AccountId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidName {
    #[error("name cannot be empty")]
    Empty,
    #[error("name cannot be longer than {max} characters")]
    TooLong { max: usize },
}

/// Trims `raw` and checks it is non-empty and at most `max` characters long.
fn bounded_text(raw: &str, max: usize) -> Result<String, InvalidName> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Err(InvalidName::Empty)
    } else if trimmed.chars().count() > max {
        Err(InvalidName::TooLong { max })
    } else {
        Ok(trimmed.to_owned())
    }
}

/// The short label shown in the strip, e.g. "Work" or "Personal". Its
/// initials become the avatar when there is no photo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AccountName(String);

impl AccountName {
    pub const MAX_CHARS: usize = 32;

    pub fn parse(raw: &str) -> Result<Self, InvalidName> {
        bounded_text(raw, Self::MAX_CHARS).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Up to two letters for the avatar: "Work Phone" → "WP", "Account 2" → "A2".
    pub fn initials(&self) -> String {
        let mut words = self.0.split_whitespace();
        let first = words.next().and_then(|w| w.chars().next());
        let second = words.next().and_then(|w| w.chars().next());
        first
            .into_iter()
            .chain(second)
            .flat_map(char::to_uppercase)
            .collect()
    }
}

impl TryFrom<String> for AccountName {
    type Error = InvalidName;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<AccountName> for String {
    fn from(name: AccountName) -> Self {
        name.0
    }
}

/// The owner's full name for the account, e.g. "Mohammed Gadi". Optional:
/// shown in tooltips and the editor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FullName(String);

impl FullName {
    pub const MAX_CHARS: usize = 64;

    /// Blank input means "no full name" and gives `Ok(None)`.
    pub fn parse_optional(raw: &str) -> Result<Option<Self>, InvalidName> {
        match bounded_text(raw, Self::MAX_CHARS) {
            Ok(text) => Ok(Some(Self(text))),
            Err(InvalidName::Empty) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FullName {
    type Error = InvalidName;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        bounded_text(&raw, Self::MAX_CHARS).map(Self)
    }
}

impl From<FullName> for String {
    fn from(name: FullName) -> Self {
        name.0
    }
}

/// Index into the account accent palette. The UI owns the actual colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct Accent(u8);

impl Accent {
    pub const COUNT: u8 = 8;

    pub fn new(index: u8) -> Self {
        Self(index % Self::COUNT)
    }

    pub fn index(self) -> u8 {
        self.0
    }
}

impl From<u8> for Accent {
    fn from(index: u8) -> Self {
        Self::new(index)
    }
}

impl From<Accent> for u8 {
    fn from(accent: Accent) -> Self {
        accent.0
    }
}

/// The user-editable parts of an account, applied together from the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: AccountName,
    pub full_name: Option<FullName>,
    pub accent: Accent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    id: AccountId,
    name: AccountName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    full_name: Option<FullName>,
    accent: Accent,
    /// Whether a custom avatar image is stored for this account.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    has_photo: bool,
}

impl Account {
    pub fn new(id: AccountId, name: AccountName, accent: Accent) -> Self {
        Self {
            id,
            name,
            full_name: None,
            accent,
            has_photo: false,
        }
    }

    pub fn id(&self) -> AccountId {
        self.id
    }

    pub fn name(&self) -> &AccountName {
        &self.name
    }

    pub fn full_name(&self) -> Option<&FullName> {
        self.full_name.as_ref()
    }

    pub fn accent(&self) -> Accent {
        self.accent
    }

    pub fn has_photo(&self) -> bool {
        self.has_photo
    }

    pub(super) fn apply(&mut self, profile: Profile) {
        self.name = profile.name;
        self.full_name = profile.full_name;
        self.accent = profile.accent;
    }

    pub(super) fn set_has_photo(&mut self, has_photo: bool) {
        self.has_photo = has_photo;
    }
}
