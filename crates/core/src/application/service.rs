use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::domain::{
    Accent, Account, AccountId, AccountName, FullName, Profile, Roster, RosterError,
    unread_from_title,
};

use super::error::AppError;
use super::ports::{AvatarStore, RosterObserver, RosterStore, Sessions};
use super::view::{Extras, RosterView};

/// What the account editor submits. Raw text; validated here.
#[derive(Debug, Clone)]
pub struct ProfileInput {
    pub name: String,
    pub full_name: String,
    pub accent: u8,
}

/// The adapters the service works through.
pub struct Ports {
    pub store: Arc<dyn RosterStore>,
    pub avatars: Arc<dyn AvatarStore>,
    pub sessions: Arc<dyn Sessions>,
    pub observer: Arc<dyn RosterObserver>,
}

/// The use cases behind the account strip and editor: link, switch,
/// customise, reload and unlink accounts.
///
/// Locking rule: state is changed and persisted under the lock, and the lock is
/// released before calling [`Sessions`]. Webview work can re-enter the event
/// loop, and holding the lock there could deadlock.
pub struct AccountService {
    state: Mutex<State>,
    ports: Ports,
}

#[derive(Default)]
struct State {
    roster: Roster,
    extras: Extras,
}

impl AccountService {
    pub fn new(ports: Ports) -> Self {
        Self {
            state: Mutex::default(),
            ports,
        }
    }

    /// Restores the saved accounts (linking a first one on first launch) and starts every session.
    pub fn start(&self) -> Result<(), AppError> {
        let (ids, active) = {
            let mut state = self.lock();
            state.roster = self.ports.store.load().unwrap_or_else(|err| {
                log::error!("{err}; starting with no accounts");
                Roster::default()
            });
            if state.roster.accounts().is_empty() {
                state.roster.add()?;
                self.ports.store.save(&state.roster)?;
            }
            state.extras.photos = self.load_photos(&state.roster);
            let ids: Vec<AccountId> = state.roster.accounts().iter().map(Account::id).collect();
            (ids, state.roster.active())
        };
        // Bring the active account up first so it is on screen as early as possible.
        if let Some(active) = active {
            self.ports.sessions.open(active)?;
            self.ports.sessions.focus(active)?;
        }
        for id in ids.into_iter().filter(|id| Some(*id) != active) {
            self.ports.sessions.open(id)?;
        }
        self.publish();
        Ok(())
    }

    pub fn view(&self) -> RosterView {
        let state = self.lock();
        RosterView::build(&state.roster, &state.extras)
    }

    pub fn add(&self) -> Result<(), AppError> {
        let id = self.mutate(Roster::add)?;
        self.ports.sessions.open(id)?;
        self.ports.sessions.focus(id)?;
        self.publish();
        Ok(())
    }

    pub fn activate(&self, id: AccountId) -> Result<(), AppError> {
        if self.lock().roster.active() == Some(id) {
            return Ok(());
        }
        self.mutate(|roster| roster.activate(id))?;
        self.ports.sessions.focus(id)?;
        self.publish();
        Ok(())
    }

    /// Reorders the strip. Order is presentation only, so sessions are untouched.
    pub fn move_account(&self, id: AccountId, index: usize) -> Result<(), AppError> {
        self.mutate(|roster| roster.move_to(id, index))?;
        self.publish();
        Ok(())
    }

    /// Fails if the account is gone, so callers can refuse to open an editor for it.
    pub fn ensure_exists(&self, id: AccountId) -> Result<(), AppError> {
        self.lock().roster.get(id)?;
        Ok(())
    }

    /// Saves name, full name and colour from the editor.
    pub fn update_profile(&self, id: AccountId, input: &ProfileInput) -> Result<(), AppError> {
        let profile = Profile {
            name: AccountName::parse(&input.name).map_err(RosterError::from)?,
            full_name: FullName::parse_optional(&input.full_name).map_err(RosterError::from)?,
            accent: Accent::new(input.accent),
        };
        self.mutate(|roster| roster.update_profile(id, profile))?;
        self.publish();
        Ok(())
    }

    /// Imports an image file as the account's photo.
    pub fn set_photo(&self, id: AccountId, source: &Path) -> Result<(), AppError> {
        self.ensure_exists(id)?;
        let data_url = self.ports.avatars.import(id, source)?;
        self.mutate(|roster| roster.set_has_photo(id, true))?;
        self.lock().extras.photos.insert(id, data_url);
        self.publish();
        Ok(())
    }

    pub fn clear_photo(&self, id: AccountId) -> Result<(), AppError> {
        self.mutate(|roster| roster.set_has_photo(id, false))?;
        self.lock().extras.photos.remove(&id);
        self.ports.avatars.remove(id)?;
        self.publish();
        Ok(())
    }

    pub fn reload(&self, id: AccountId) -> Result<(), AppError> {
        self.ensure_exists(id)?;
        self.ports.sessions.reload(id)?;
        Ok(())
    }

    /// Unlinks an account and erases its local data. The window always keeps at
    /// least one account, so unlinking the last one links a fresh, logged-out one.
    pub fn remove(&self, id: AccountId) -> Result<(), AppError> {
        let (replacement, next_active) = self.mutate(|roster| {
            roster.remove(id)?;
            let replacement = if roster.accounts().is_empty() {
                Some(roster.add()?)
            } else {
                None
            };
            Ok((replacement, roster.active()))
        })?;
        {
            let mut state = self.lock();
            state.extras.unread.remove(&id);
            state.extras.photos.remove(&id);
        }
        if let Err(err) = self.ports.avatars.remove(id) {
            log::warn!("could not delete the avatar of removed account {id}: {err}");
        }
        self.ports.sessions.destroy(id)?;
        if let Some(fresh) = replacement {
            self.ports.sessions.open(fresh)?;
        }
        if let Some(active) = next_active {
            self.ports.sessions.focus(active)?;
        }
        self.publish();
        Ok(())
    }

    /// Called when a session's page title changes. That is the only channel the
    /// web content has to us, and we only read the unread count from it.
    pub fn title_changed(&self, id: AccountId, title: &str) {
        let count = unread_from_title(title);
        let changed = {
            let mut state = self.lock();
            if !state.roster.contains(id) {
                return;
            }
            state.extras.unread.insert(id, count) != Some(count)
        };
        if changed {
            self.publish();
        }
    }

    fn load_photos(&self, roster: &Roster) -> std::collections::HashMap<AccountId, String> {
        roster
            .accounts()
            .iter()
            .filter(|account| account.has_photo())
            .filter_map(|account| match self.ports.avatars.load(account.id()) {
                Ok(photo) => photo.map(|url| (account.id(), url)),
                Err(err) => {
                    log::warn!("avatar of account {} is unreadable: {err}", account.id());
                    None
                }
            })
            .collect()
    }

    /// Applies a roster change and saves it. On failure nothing changes, in memory or on disk.
    fn mutate<T>(
        &self,
        change: impl FnOnce(&mut Roster) -> Result<T, RosterError>,
    ) -> Result<T, AppError> {
        let mut state = self.lock();
        let mut next = state.roster.clone();
        let outcome = change(&mut next)?;
        self.ports.store.save(&next)?;
        state.roster = next;
        Ok(outcome)
    }

    fn publish(&self) {
        let view = self.view();
        self.ports.observer.roster_changed(&view);
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // A panic while holding the lock cannot leave `State` half-written:
        // `mutate` swaps in a fully built roster. So recovering is safe.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
