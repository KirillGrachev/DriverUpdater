//! Application service for driver inventory, updates, restore points and backups.
//!
//! <p>Installed drivers come from the native detector. Available driver updates
//! come from Windows Update; restore points and driver package backups use the
//! real Windows providers. There is no fixture fallback in the production
//! repository: a failed scan produces an empty inventory plus a visible error.</p>

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::detection::{
    days_to_civil, is_ignored_device, ComponentDetector, NativeDetector, SystemInfoProvider,
};
use crate::models::{CheckUpdatesResult, Driver, InitialData, RestorePoint, Settings, Summary};
use crate::storage::{DriverCache, EncryptedStore, PersistedState, SCHEMA_VERSION};
use crate::windows_update::{merge_updates, DriverUpdate};

pub struct InstallPlan {
    pub update_id: String,
    pub restore_before_install: bool,
    pub keep_backups: bool,
    pub backup_dir: PathBuf,
}

pub struct DriverRepository {
    drivers: Vec<Driver>,
    restore_points: Vec<RestorePoint>,
    settings: Settings,
    last_check: Option<String>,
    devices_total: u32,
    store: EncryptedStore,
    cache: DriverCache,
    backup_dir: PathBuf,
    needs_scan: bool,
    scan_error: Option<String>,
    installed_components: Vec<String>,
    component_detector: Box<dyn ComponentDetector>,
    system_info: Box<dyn SystemInfoProvider>,
}

const SUPPORTED_LANGUAGES: &[&str] = &[
    "ru", "uk", "be", "en", "de", "es", "fr", "it", "pt", "pl", "cs", "sk", "bg", "hr", "sl", "nl",
    "da", "sv", "fi", "et", "lv", "lt", "ro", "hu", "el", "ga", "mt", "kk", "ky", "tg", "tk", "uz",
    "az", "ka", "hy",
];

fn default_language() -> String {
    let locale = sys_locale::get_locale().unwrap_or_default();
    let code = locale
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
    if SUPPORTED_LANGUAGES.contains(&code.as_str()) {
        code
    } else {
        "en".to_string()
    }
}

fn clear_update_state(drivers: &mut [Driver]) {
    for driver in drivers {
        driver.available = None;
        driver.status = "current".to_string();
        driver.update_id = None;
        driver.recommended = false;
        driver.reboot_required = false;
    }
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (hours, mins, sec) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let civil = days_to_civil(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        civil.0, civil.1, civil.2, hours, mins, sec
    )
}

impl DriverRepository {
    pub fn new(state_file: Option<PathBuf>, backup_dir: PathBuf) -> Self {
        let store = EncryptedStore::new(state_file);
        let cache = DriverCache::new(store.path());
        let mut scan_error = None;
        let cached = match cache.load() {
            Ok(Some(drivers))
                if drivers.iter().any(|driver| {
                    driver.device_id.is_empty()
                        || is_ignored_device(&driver.device_id, &driver.hardware_ids)
                }) =>
            {
                scan_error = Some("driver cache contains obsolete devices; rescanning".to_string());
                None
            }
            Ok(cached) => cached,
            Err(error) => {
                scan_error = Some(format!("driver cache could not be read: {error}"));
                None
            }
        };
        let had_cache = cached.is_some();
        let needs_scan = !had_cache;
        let drivers = cached.unwrap_or_default();
        let settings = Settings {
            backup_path: backup_dir.to_string_lossy().into_owned(),
            language: default_language(),
            ..Settings::default()
        };

        let mut repo = Self {
            drivers,
            restore_points: Vec::new(),
            settings,
            last_check: None,
            devices_total: 0,
            store,
            cache,
            backup_dir: backup_dir.clone(),
            needs_scan,
            scan_error,
            installed_components: Vec::new(),
            component_detector: Box::<NativeDetector>::default(),
            system_info: Box::<NativeDetector>::default(),
        };

        match repo.store.load::<PersistedState>() {
            Ok(Some(state)) => {
                if state.version > SCHEMA_VERSION {
                    repo.scan_error = Some(format!(
                        "state schema {} is newer than this app supports",
                        state.version
                    ));
                } else {
                    repo.restore_points = state.restore_points;
                    repo.settings = state.settings;
                    repo.settings.backup_path = repo.backup_dir.to_string_lossy().into_owned();
                    repo.last_check = state.last_check;
                    repo.installed_components = state.installed_components;
                    if state.version == 2 {
                        // v2 could still contain fixture restore points loaded
                        // before the real provider was introduced. Keep user
                        // settings but discard the demo data once.
                        repo.restore_points.clear();
                        repo.installed_components.clear();
                        repo.settings.auto_check = false;
                        repo.settings.auto_restore_points = false;
                        repo.last_check = None;
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                repo.scan_error = Some(format!("saved state could not be read: {error}"));
            }
        }
        if !had_cache {
            repo.last_check = None;
            clear_update_state(&mut repo.drivers);
        }
        repo.devices_total = repo.drivers.len() as u32;
        repo
    }

    fn persist(&self) -> Result<(), String> {
        let state = PersistedState {
            version: SCHEMA_VERSION,
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            last_check: self.last_check.clone(),
            installed_components: self.installed_components.clone(),
        };
        self.store.save(&state)
    }

    pub fn summary(&self) -> Summary {
        let updates_available = self.drivers.iter().filter(|d| d.status == "update").count() as u32;
        Summary {
            devices_total: self.devices_total,
            up_to_date: self.devices_total.saturating_sub(updates_available),
            updates_available,
            last_check: self.last_check.clone(),
        }
    }

    pub fn backup_dir(&self) -> &std::path::Path {
        self.backup_dir.as_path()
    }

    /// Detects installed runtimes and developer tools through the configured adapter.
    pub fn detect_components(&self) -> Vec<crate::models::InstalledComponent> {
        self.component_detector.detect()
    }

    /// Returns the component ids persisted by previous installs.
    pub fn installed_components(&self) -> Vec<String> {
        self.installed_components.clone()
    }

    /// Replaces the persisted component ids and returns the normalized list.
    pub fn save_installed_components(&mut self, ids: Vec<String>) -> Result<Vec<String>, String> {
        let mut unique = Vec::new();
        for id in ids {
            if !id.is_empty() && id.len() <= 64 && !unique.contains(&id) {
                unique.push(id);
            }
        }
        self.installed_components = unique.clone();
        self.persist()?;
        Ok(unique)
    }

    pub fn snapshot(&self) -> InitialData {
        InitialData {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            summary: self.summary(),
            system_info: self.system_info.info(),
            scan_error: self.scan_error.clone(),
            needs_scan: self.needs_scan,
        }
    }

    pub fn apply_initial_scan(&mut self, drivers: Option<Vec<Driver>>) -> Result<(), String> {
        let Some(drivers) = drivers.filter(|drivers| !drivers.is_empty()) else {
            let error = "Windows driver scan returned no devices".to_string();
            self.scan_error = Some(error.clone());
            return Err(error);
        };
        let mut drivers = drivers;
        clear_update_state(&mut drivers);
        self.cache.save(&drivers)?;
        self.drivers = drivers;
        self.devices_total = self.drivers.len() as u32;
        self.last_check = None;
        self.needs_scan = false;
        self.scan_error = None;
        Ok(())
    }

    pub fn apply_check_result(
        &mut self,
        drivers: Option<Vec<Driver>>,
        updates: Result<Vec<DriverUpdate>, String>,
    ) -> Result<CheckUpdatesResult, String> {
        let mut drivers = match drivers.filter(|drivers| !drivers.is_empty()) {
            Some(drivers) => drivers,
            None if !self.drivers.is_empty() => {
                self.scan_error =
                    Some("Windows driver scan failed; using the cached inventory".to_string());
                self.drivers.clone()
            }
            None => {
                let error = "Windows driver scan returned no devices".to_string();
                self.scan_error = Some(error.clone());
                return Err(error);
            }
        };
        let updates = match updates {
            Ok(updates) => updates,
            Err(error) => {
                self.scan_error = Some(format!("Windows Update search failed: {error}"));
                return Err(error);
            }
        };
        merge_updates(&mut drivers, &updates);
        self.cache.save(&drivers)?;
        self.drivers = drivers;
        self.devices_total = self.drivers.len() as u32;
        self.last_check = Some(now_iso());
        self.needs_scan = false;
        self.scan_error = None;
        self.persist()?;
        Ok(CheckUpdatesResult {
            drivers: self.drivers.clone(),
            summary: self.summary(),
        })
    }

    pub fn backup_policy(&self) -> (bool, PathBuf) {
        (self.settings.keep_backups, self.backup_dir.clone())
    }

    pub fn restore_policy(&self) -> bool {
        self.settings.restore_point_before_install
    }

    pub fn prepare_install(&self, id: &str) -> Result<InstallPlan, String> {
        let driver = self
            .drivers
            .iter()
            .find(|driver| driver.id == id)
            .ok_or_else(|| format!("Unknown driver: {id}"))?;
        let update_id = driver.update_id.clone().ok_or_else(|| {
            "No Windows Update candidate is associated with this driver".to_string()
        })?;
        Ok(InstallPlan {
            update_id,
            restore_before_install: self.settings.restore_point_before_install,
            keep_backups: self.settings.keep_backups,
            backup_dir: self.backup_dir.clone(),
        })
    }

    pub fn finish_install(
        &mut self,
        id: &str,
        outcome: crate::windows_update::InstallOutcome,
    ) -> Result<Driver, String> {
        let driver = self
            .drivers
            .iter_mut()
            .find(|driver| driver.id == id)
            .ok_or_else(|| format!("Unknown driver: {id}"))?;
        if let Some(available) = driver.available.take() {
            driver.current = available;
        }
        driver.status = "current".to_string();
        driver.update_id = None;
        driver.recommended = false;
        driver.reboot_required = outcome.reboot_required;
        let installed = driver.clone();
        self.cache.save(&self.drivers)?;
        self.persist()?;
        Ok(installed)
    }

    pub fn set_restore_points(&mut self, points: Vec<RestorePoint>) {
        self.restore_points = points;
    }

    pub fn save_settings(&mut self, mut settings: Settings) -> Result<Settings, String> {
        settings.backup_path = self.backup_dir.to_string_lossy().into_owned();
        self.settings = settings;
        self.persist()?;
        Ok(self.settings.clone())
    }
}
