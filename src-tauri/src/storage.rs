/*!
 * Encrypted persistence for application state and the driver cache.
 *
 * <p>All files written by this module go through [`security::seal`]. Reads
 * accept a legacy plaintext payload once, then rewrite it in sealed form. Writes
 * use a temporary file plus rename so a crash cannot truncate the state.</p>
 */

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::models::{Driver, RestorePoint, Settings};
use crate::security;

/**
 * Mutable state persisted between application launches.
 */
pub const SCHEMA_VERSION: u32 = 4;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub restore_points: Vec<RestorePoint>,
    #[serde(default)]
    pub settings: Settings,
    pub last_check: Option<String>,
    #[serde(default)]
    pub installed_components: Vec<String>,
}

/**
 * A single encrypted JSON document on disk.
 */
pub struct EncryptedStore {
    path: Option<PathBuf>,
}

impl EncryptedStore {
    /**
     * Creates a store for an optional path. `None` keeps the data in memory.
     */
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    /**
     * Returns the backing path, if persistence is enabled.
     */
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /**
     * Loads and decrypts a document, migrating legacy plaintext on success.
     *
     * A missing file is `Ok(None)`. Read, decryption and parse failures are
     * returned so the caller can surface corruption instead of silently
     * replacing user data with defaults.
     */
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

    /**
     * Encrypts and atomically writes a document.
     */
    pub fn save<T: Serialize>(&self, value: &T) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let plaintext = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        let sealed = security::seal(&plaintext)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        static SAVE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let temporary = path.with_extension(format!(
            "json.{}.tmp",
            SAVE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
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

/**
 * Encrypted cache of the last successful driver scan.
 */
pub struct DriverCache {
    store: EncryptedStore,
}

/**
 * Bump when the driver filter or cache contract changes.
 */
const DRIVER_CACHE_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct CachedDrivers {
    cache_version: u32,
    drivers: Vec<Driver>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CachePayload {
    Versioned(CachedDrivers),
    Legacy(Vec<Driver>),
}

impl DriverCache {
    /**
     * Creates a cache next to the persisted application state.
     */
    pub fn new(state_file: Option<&Path>) -> Self {
        let path = state_file
            .and_then(Path::parent)
            .map(|directory| directory.join("installed-drivers.json"));
        Self {
            store: EncryptedStore::new(path),
        }
    }

    /**
     * Loads a non-empty cached driver list.
     */
    pub fn load(&self) -> Result<Option<Vec<Driver>>, String> {
        let Some(payload) = self.store.load::<CachePayload>()? else {
            return Ok(None);
        };
        let drivers = match payload {
            CachePayload::Versioned(cache) if cache.cache_version == DRIVER_CACHE_VERSION => {
                cache.drivers
            }
            CachePayload::Versioned(_) | CachePayload::Legacy(_) => return Ok(None),
        };
        Ok((!drivers.is_empty()).then_some(drivers))
    }

    /**
     * Replaces the cache with the latest scan.
     */
    pub fn save(&self, drivers: &[Driver]) -> Result<(), String> {
        self.store.save(&CachedDrivers {
            cache_version: DRIVER_CACHE_VERSION,
            drivers: drivers.to_vec(),
        })
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

    #[test]
    fn reads_legacy_state_without_version_or_drivers() {
        let state: PersistedState = serde_json::from_value(json!({
            "settings": { "language": "de" },
            "restore_points": [],
            "installed_components": ["vcredist-x64"]
        }))
        .expect("legacy state");
        assert_eq!(state.version, 0);
        assert_eq!(state.settings.language, "de");
        assert_eq!(state.installed_components, vec!["vcredist-x64"]);
    }

    #[test]
    fn driver_cache_version_mismatch_invalidates_the_cache() {
        let directory = std::env::temp_dir().join(format!(
            "driver-updater-cache-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let state_path = directory.join("state.json");
        let cache = DriverCache::new(Some(&state_path));
        let driver = Driver {
            id: "pnp-1".to_string(),
            name: "Voice Clarity".to_string(),
            vendor: "Microsoft".to_string(),
            category: "audio".to_string(),
            icon: crate::models::DriverIcon {
                kind: "audio".to_string(),
                color: "#000".to_string(),
            },
            current: "1.0.0".to_string(),
            available: None,
            status: "current".to_string(),
            size_mb: 0,
            recommended: false,
            device: "Voice Clarity".to_string(),
            released: String::new(),
            device_id: "SWD\\DEVICE".to_string(),
            hardware_ids: vec!["SWC\\VEN_MSFT&AUDIO".to_string()],
            inf_name: String::new(),
            update_id: None,
            reboot_required: false,
        };
        cache
            .save(std::slice::from_ref(&driver))
            .expect("save cache");
        assert_eq!(
            cache.load().expect("load cache").map(|items| items.len()),
            Some(1)
        );

        cache
            .store
            .save(&CachedDrivers {
                cache_version: 1,
                drivers: vec![driver],
            })
            .expect("save old cache");
        assert!(cache.load().expect("load old cache").is_none());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn settings_without_auto_install_default_to_false() {
        let settings: Settings = serde_json::from_value(json!({
            "language": "en",
            "theme": "system",
            "autoCheck": true,
            "checkInterval": "daily",
            "restorePointBeforeInstall": true,
            "keepBackups": true,
            "backupPath": "",
            "autoRestorePoints": false,
            "restorePointPeriod": "weekly"
        }))
        .expect("legacy settings");
        assert!(!settings.auto_install);
    }

    #[test]
    fn corrupt_state_returns_an_error_instead_of_defaults() {
        let directory = std::env::temp_dir().join(format!(
            "driver-updater-corrupt-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&directory).expect("create directory");
        let path = directory.join("state.json");
        fs::write(&path, b"not valid json").expect("write corrupt state");
        let store = EncryptedStore::new(Some(path));
        let loaded: Result<Option<serde_json::Value>, String> = store.load();
        assert!(loaded.is_err());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn concurrent_saves_leave_a_readable_document() {
        let directory = std::env::temp_dir().join(format!(
            "driver-updater-concurrent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let path = directory.join("state.json");
        let store = std::sync::Arc::new(EncryptedStore::new(Some(path)));
        let mut handles = Vec::new();
        for index in 0..4 {
            let store = std::sync::Arc::clone(&store);
            handles.push(std::thread::spawn(move || {
                store.save(&json!({ "writer": index })).expect("save");
            }));
        }
        for handle in handles {
            handle.join().expect("join");
        }
        let loaded: serde_json::Value = store.load().expect("load").expect("state");
        assert!(loaded.get("writer").is_some());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn migrates_legacy_localized_restore_point_name() {
        let point: RestorePoint = serde_json::from_value(json!({
            "id": "rp-1",
            "name": { "ru": "Точка", "en": "Restore point" },
            "kind": "manual",
            "createdAt": "2026-01-01T00:00:00Z",
            "sizeMb": 1
        }))
        .expect("legacy restore point");
        assert_eq!(point.name, "Restore point");
    }
}
