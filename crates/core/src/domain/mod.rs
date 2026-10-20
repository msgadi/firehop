//! Business rules for linked accounts.
//!
//! Pure Rust: no Tauri, no I/O, no platform code.

mod account;
mod roster;
mod unread;

pub use account::{Accent, Account, AccountId, AccountName, FullName, InvalidName, Profile};
pub use roster::{MAX_ACCOUNTS, Roster, RosterError};
pub use unread::unread_from_title;
