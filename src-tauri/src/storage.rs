//! Encrypted persistence for application state and the driver cache.
//!
//! <p>All files written by this module go through [`security::seal`]. Reads
//! accept a legacy plaintext payload once, then rewrite it in sealed form. Writes
//! use a temporary file plus rename so a crash cannot truncate the state.</p>

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::models::{Driver, RestorePoint, Settings};
use crate::security;

/// Mutable state persisted between application launches.
pub const SCHEMA_VERSION: u32 = 3;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub drivers: Vec<Driver>,
    #[serde(default)]
    pub restore_points: Vec<RestorePoint>,
    #[serde(default)]
    pub settings: Settings,
    pub last_check: Option<String>,
    #[serde(default)]
    pub installed_components: Vec<String>,
}

/// A single encrypted JSON document on disk.
pub struct EncryptedStore {
    path: Option<PathBuf>,
}

impl EncryptedStore {
    /// Creates a store for an optional path. `None` keeps the data in memory.
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    /// Returns the backing path, if persistence is enabled.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Loads and decrypts a document, migrating legacy plaintext on success.
    ///
    /// A missing file is `Ok(None)`. Read, decryption and parse failures are
    /// returned so the caller can surface corruption instead of silently
    /// replacing user data with defaults.
    pub fn load<T: DeserializeOwned + Serialize>(&self) -> Result<Option<T>, String> {
        let Some(path) = self.path.as_ref() else {
            return Ok(None);
        };
        let payload = match fs::read(path) {
            Ok(payload) => payload,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("failed to read state: {error}")),
        };
        let was_sealed = security::is_sealed(&payload);
        let plaintext = security::open(&payload)?;
        let value = serde_json::from_slice(&plaintext)
            .map_err(|error| format!("state is not valid JSON: {error}"))?;

        if !was_sealed {
            self.save(&value)?;
        }
        Ok(Some(value))
    }

    /// Encrypts and atomically writes a document.
    pub fn save<T: Serialize>(&self, value: &T) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let plaintext = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        let sealed = security::seal(&plaintext)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let temporary = path.with_extension("json.tmp");
        let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
        file.write_all(&sealed).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(&temporary, path).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            format!("failed to replace state atomically: {error}")
        })?;
        Ok(())
    }
}

/// Encrypted cache of the last successful driver scan.
pub struct DriverCache {
    store: EncryptedStore,
}

impl DriverCache {
    /// Creates a cache next to the persisted application state.
    pub fn new(state_file: Option<&Path>) -> Self {
        let path = state_file
            .and_then(Path::parent)
            .map(|directory| directory.join("installed-drivers.json"));
        Self {
            store: EncryptedStore::new(path),
        }
    }

    /// Loads a non-empty cached driver list.
    pub fn load(&self) -> Result<Option<Vec<Driver>>, String> {
        let Some(drivers) = self.store.load::<Vec<Driver>>()? else {
            return Ok(None);
        };
        Ok((!drivers.is_empty()).then_some(drivers))
    }

    /// Replaces the cache with the latest scan.
    pub fn save(&self, drivers: &[Driver]) -> Result<(), String> {
        self.store.save(&drivers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn writes_encrypted_json_and_reads_it_back() {
        let directory = std::env::temp_dir().join(format!(
            "driver-updater-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let path = directory.join("state.json");
        let store = EncryptedStore::new(Some(path.clone()));
        let expected = json!({ "hostname": "test-pc", "count": 3 });

        store.save(&expected).expect("save");
        #[cfg(target_os = "windows")]
        {
            let raw = fs::read(&path).expect("read raw");
            assert!(crate::security::is_sealed(&raw));
        }

        let loaded: serde_json::Value = store.load().expect("load").expect("state");
        assert_eq!(loaded, expected);
        let _ = fs::remove_dir_all(directory);
    }
}
