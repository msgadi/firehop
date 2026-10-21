use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::account::{Accent, Account, AccountId, AccountName, InvalidName, Profile};

/// Upper bound on linked accounts. Every account is a live browser engine, so
/// this keeps memory use predictable.
pub const MAX_ACCOUNTS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RosterError {
    #[error("at most {MAX_ACCOUNTS} accounts can be linked")]
    Full,
    #[error("no account with id {0}")]
    NotFound(AccountId),
    #[error(transparent)]
    InvalidName(#[from] InvalidName),
}

/// The ordered list of linked accounts and which one is on screen.
///
/// This is the aggregate root: every change goes through it, so these
/// invariants always hold:
/// - ids are unique and there are at most [`MAX_ACCOUNTS`] accounts;
/// - `active` is `None` only when there are no accounts, and otherwise names one of them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RosterRecord", into = "RosterRecord")]
pub struct Roster {
    accounts: Vec<Account>,
    active: Option<AccountId>,
}

impl Roster {
    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }

    pub fn active(&self) -> Option<AccountId> {
        self.active
    }

    pub fn is_full(&self) -> bool {
        self.accounts.len() >= MAX_ACCOUNTS
    }

    pub fn contains(&self, id: AccountId) -> bool {
        self.position(id).is_some()
    }

    /// Links a new account with a default name and makes it active.
    pub fn add(&mut self) -> Result<AccountId, RosterError> {
        if self.is_full() {
            return Err(RosterError::Full);
        }
        let id = AccountId::generate();
        let account = Account::new(id, self.next_default_name(), self.least_used_accent());
        self.accounts.push(account);
        self.active = Some(id);
        Ok(id)
    }

    pub fn activate(&mut self, id: AccountId) -> Result<(), RosterError> {
        self.require(id)?;
        self.active = Some(id);
        Ok(())
    }

    pub fn get(&self, id: AccountId) -> Result<&Account, RosterError> {
        self.require(id).map(|index| &self.accounts[index])
    }

    /// Replaces the account's name, full name and accent in one step.
    pub fn update_profile(&mut self, id: AccountId, profile: Profile) -> Result<(), RosterError> {
        let index = self.require(id)?;
        self.accounts[index].apply(profile);
        Ok(())
    }

    pub fn set_has_photo(&mut self, id: AccountId, has_photo: bool) -> Result<(), RosterError> {
        let index = self.require(id)?;
        self.accounts[index].set_has_photo(has_photo);
        Ok(())
    }

    /// Moves an account to `index` in the strip order (clamped to the last
    /// slot). The other accounts keep their relative order.
    pub fn move_to(&mut self, id: AccountId, index: usize) -> Result<(), RosterError> {
        let from = self.require(id)?;
        let account = self.accounts.remove(from);
        let to = index.min(self.accounts.len());
        self.accounts.insert(to, account);
        Ok(())
    }

    /// Unlinks an account. If it was active, the account that took its slot
    /// (or the new last one) becomes active.
    pub fn remove(&mut self, id: AccountId) -> Result<Account, RosterError> {
        let index = self.require(id)?;
        let removed = self.accounts.remove(index);
        if self.active == Some(id) {
            let neighbour = index.min(self.accounts.len().saturating_sub(1));
            self.active = self.accounts.get(neighbour).map(Account::id);
        }
        Ok(removed)
    }

    fn position(&self, id: AccountId) -> Option<usize> {
        self.accounts.iter().position(|a| a.id() == id)
    }

    fn require(&self, id: AccountId) -> Result<usize, RosterError> {
        self.position(id).ok_or(RosterError::NotFound(id))
    }

    fn next_default_name(&self) -> AccountName {
        let taken: HashSet<&str> = self.accounts.iter().map(|a| a.name().as_str()).collect();
        // At most MAX_ACCOUNTS names are taken, so one of the first MAX_ACCOUNTS + 1 is free.
        (1..=MAX_ACCOUNTS + 1)
            .map(|n| format!("Account {n}"))
            .find(|candidate| !taken.contains(candidate.as_str()))
            .and_then(|candidate| AccountName::parse(&candidate).ok())
            .expect("a free default name always exists within the account limit")
    }

    fn least_used_accent(&self) -> Accent {
        let mut usage = [0_usize; Accent::COUNT as usize];
        for account in &self.accounts {
            usage[usize::from(account.accent().index())] += 1;
        }
        let least = (0..Accent::COUNT)
            .min_by_key(|&i| usage[usize::from(i)])
            .unwrap_or(0);
        Accent::new(least)
    }
}

/// On-disk shape of a [`Roster`]. Loading goes through [`From`], which repairs
/// a hand-edited or corrupted file instead of rejecting it.
#[derive(Serialize, Deserialize)]
struct RosterRecord {
    accounts: Vec<Account>,
    active: Option<AccountId>,
}

impl From<RosterRecord> for Roster {
    fn from(record: RosterRecord) -> Self {
        let mut seen = HashSet::new();
        let accounts: Vec<Account> = record
            .accounts
            .into_iter()
            .filter(|a| seen.insert(a.id()))
            .take(MAX_ACCOUNTS)
            .collect();
        let active = record
            .active
            .filter(|id| accounts.iter().any(|a| a.id() == *id))
            .or_else(|| accounts.first().map(Account::id));
        Self { accounts, active }
    }
}

impl From<Roster> for RosterRecord {
    fn from(roster: Roster) -> Self {
        Self {
            accounts: roster.accounts,
            active: roster.active,
        }
    }
}
