//! Use cases. Orchestrates the domain through ports and knows nothing about Tauri or WebView2.

mod error;
mod ports;
mod service;
mod view;

pub use error::AppError;
pub use ports::{
    AvatarError, AvatarStore, RosterObserver, RosterStore, SessionError, Sessions, StoreError,
};
pub use service::{AccountService, Ports, ProfileInput};
pub use view::{AccountView, Extras, RosterView};
