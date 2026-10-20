//! Everything in Firehop that does not need a window: account rules, use
//! cases, security policy, layout maths and roster storage.
//!
//! This crate has no Tauri or platform dependency, so the compiler enforces the
//! dependency rule: the shell depends on the core, never the other way round.
//!
//! - [`domain`]: account rules, pure Rust.
//! - [`application`]: use cases, written against ports (traits).
//! - [`policy`]: what account webviews may navigate to, open and access.
//! - [`layout`]: window geometry.
//! - [`storage`]: the JSON roster store adapter.
//! - [`avatars`]: the account photo store adapter.

pub mod application;
pub mod avatars;
pub mod domain;
pub mod layout;
pub mod policy;
pub mod storage;
