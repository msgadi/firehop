use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::application::{RosterStore, StoreError};
use crate::domain::Roster;

/// Keeps the roster in a JSON file and replaces it atomically, so a crash
/// mid-write leaves the previous version in place.
pub struct JsonRosterStore {
    path: PathBuf,
}

impl JsonRosterStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn write_atomically(&self, bytes: &[u8]) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let staging = self.path.with_extension("json.tmp");
        let mut file = fs::File::create(&staging)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&staging, &self.path)
    }
}

impl RosterStore for JsonRosterStore {
    fn load(&self) -> Result<Roster, StoreError> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|err| StoreError(format!("{} is unreadable: {err}", self.path.display()))),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Roster::default()),
            Err(err) => Err(StoreError(format!(
                "cannot read {}: {err}",
                self.path.display()
            ))),
        }
    }

    fn save(&self, roster: &Roster) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec_pretty(roster).map_err(|err| StoreError(err.to_string()))?;
        self.write_atomically(&bytes)
            .map_err(|err| StoreError(format!("cannot write {}: {err}", self.path.display())))
    }
}
