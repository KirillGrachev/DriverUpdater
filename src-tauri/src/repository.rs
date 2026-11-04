//! Application service for driver inventory, updates, restore points and backups.
//!
//! <p>Installed drivers come from the native detector. Available driver updates
//! come from Windows Update; restore points and driver package backups use the
//! real Windows providers. There is no fixture fallback in the production
//! repository: a failed scan produces an empty inventory plus a visible error.</p>

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::backup::{DriverBackup, DriverBackupProvider, WindowsDriverBackups};
use crate::detection::{
    days_to_civil, ComponentDetector, DriverDetector, NativeDetector, SystemInfoProvider,
};
use crate::models::{
    CheckUpdatesResult, Driver, InitialData, LocalizedText, RestorePoint, Settings, Summary,
};
use crate::restore::{RestorePointProvider, WindowsRestorePoints};
use crate::storage::{DriverCache, EncryptedStore, PersistedState, SCHEMA_VERSION};
use crate::windows_update::{merge_updates, DriverUpdateProvider, WindowsUpdateProvider};

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
    detector: Box<dyn DriverDetector>,
    component_detector: Box<dyn ComponentDetector>,
    system_info: Box<dyn SystemInfoProvider>,
    update_provider: Box<dyn DriverUpdateProvider>,
    restore_provider: Box<dyn RestorePointProvider>,
    backup_provider: Box<dyn DriverBackupProvider>,
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
            Ok(Some(drivers)) if drivers.iter().any(|driver| driver.device_id.is_empty()) => {
                scan_error =
                    Some("driver cache predates stable device ids; rescanning".to_string());
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
            detector: Box::<NativeDetector>::default(),
            component_detector: Box::<NativeDetector>::default(),
            system_info: Box::<NativeDetector>::default(),
            update_provider: Box::<WindowsUpdateProvider>::default(),
            restore_provider: Box::<WindowsRestorePoints>::default(),
            backup_provider: Box::new(WindowsDriverBackups::new(backup_dir)),
        };

        match repo.store.load::<PersistedState>() {
            Ok(Some(state)) => {
                if state.version > SCHEMA_VERSION {
                    repo.scan_error = Some(format!(
                        "state schema {} is newer than this app supports",
                        state.version
                    ));
                } else {
                    repo.drivers = state.drivers;
                    repo.restore_points = state.restore_points;
                    repo.settings = state.settings;
                    repo.settings.backup_path = repo.backup_dir.to_string_lossy().into_owned();
                    repo.last_check = state.last_check;
                    repo.installed_components = state.installed_components;
                    if state.version < 2 {
                        // v1 could contain fixture restore points, fixture drivers
                        // and synthetic installed component ids. Keep user settings
                        // but discard the demo data once.
                        repo.restore_points.clear();
                        repo.installed_components.clear();
                        repo.settings.notifications = false;
                        repo.settings.auto_check = false;
                        repo.settings.auto_restore_points = false;
                        repo.settings.telemetry = false;
                        if !had_cache {
                            repo.drivers.clear();
                        }
                        repo.last_check = None;
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                repo.scan_error = Some(format!("saved state could not be read: {error}"));
            }
        }
        repo.devices_total = repo.drivers.len() as u32;
        repo
    }

    fn persist(&self) -> Result<(), String> {
        let state = PersistedState {
            version: SCHEMA_VERSION,
            drivers: self.drivers.clone(),
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

    /// Scans Windows once when no fresh driver cache exists.
    pub fn ensure_fresh(&mut self) -> Result<(), String> {
        if !self.needs_scan {
            return Ok(());
        }
        let drivers = match self.detector.detect().filter(|drivers| !drivers.is_empty()) {
            Some(drivers) => drivers,
            None => {
                let error = "Windows driver scan returned no devices".to_string();
                self.scan_error = Some(error.clone());
                return Err(error);
            }
        };
        if let Err(error) = self.cache.save(&drivers) {
            self.scan_error = Some(format!("driver cache could not be saved: {error}"));
            return Err(error);
        }
        self.drivers = drivers;
        self.devices_total = self.drivers.len() as u32;
        self.last_check = Some(now_iso());
        self.needs_scan = false;
        self.scan_error = None;
        Ok(())
    }

    pub fn snapshot(&self) -> InitialData {
        InitialData {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            summary: self.summary(),
            system_info: self.system_info.info(),
            scan_error: self.scan_error.clone(),
        }
    }

    /// Rescans installed drivers and asks Windows Update for driver candidates.
    pub fn check_updates(&mut self) -> Result<CheckUpdatesResult, String> {
        let mut drivers = match self.detector.detect().filter(|drivers| !drivers.is_empty()) {
            Some(drivers) => drivers,
            None => {
                let error = "Windows driver scan returned no devices".to_string();
                self.scan_error = Some(error.clone());
                return Err(error);
            }
        };
        let updates = match self.update_provider.find_updates() {
            Ok(updates) => updates,
            Err(error) => {
                self.scan_error = Some(format!("Windows Update search failed: {error}"));
                return Err(error);
            }
        };
        merge_updates(&mut drivers, &updates);
        if let Err(error) = self.cache.save(&drivers) {
            self.scan_error = Some(format!("driver cache could not be saved: {error}"));
            return Err(error);
        }
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

    pub fn install_driver(&mut self, id: &str) -> Result<Driver, String> {
        let index = self
            .drivers
            .iter()
            .position(|driver| driver.id == id)
            .ok_or_else(|| format!("Unknown driver: {id}"))?;
        let update_id = self.drivers[index].update_id.clone().ok_or_else(|| {
            "No Windows Update candidate is associated with this driver".to_string()
        })?;

        if self.settings.restore_point_before_install {
            self.restore_provider
                .create(&LocalizedText {
                    ru: format!(
                        "Driver Updater: перед обновлением {}",
                        self.drivers[index].name
                    ),
                    en: format!(
                        "Driver Updater: before updating {}",
                        self.drivers[index].name
                    ),
                })
                .map_err(|error| format!("restore point was not created: {error}"))?;
        }
        if self.settings.keep_backups {
            self.backup_provider
                .create()
                .map_err(|error| format!("driver backup was not created: {error}"))?;
        }

        let outcome = self.update_provider.install_update(&update_id)?;
        let driver = &mut self.drivers[index];
        if let Some(available) = driver.available.take() {
            driver.current = available;
        }
        driver.status = "current".to_string();
        driver.update_id = None;
        driver.recommended = false;
        driver.reboot_required = outcome.reboot_required;
        let installed = driver.clone();
        self.persist()?;
        Ok(installed)
    }

    pub fn install_updates(&mut self, ids: &[String]) -> Result<Vec<Driver>, String> {
        for id in ids {
            let driver = self
                .drivers
                .iter()
                .find(|driver| &driver.id == id)
                .ok_or_else(|| format!("Unknown driver: {id}"))?;
            if driver.update_id.is_none() {
                return Err(format!("No Windows Update candidate for driver: {id}"));
            }
        }
        let mut updated = Vec::new();
        for id in ids {
            updated.push(self.install_driver(id)?);
        }
        Ok(updated)
    }

    pub fn restore_points(&self) -> Vec<RestorePoint> {
        self.restore_points.clone()
    }

    pub fn reload_restore_points(&mut self) -> Result<(), String> {
        self.restore_points = self.restore_provider.list()?;
        Ok(())
    }

    pub fn create_restore_point(&mut self, name: LocalizedText) -> Result<RestorePoint, String> {
        let point = self.restore_provider.create(&name)?;
        self.reload_restore_points()?;
        Ok(point)
    }

    pub fn delete_restore_point(&mut self, id: &str) -> Result<(), String> {
        self.restore_provider.delete(id)?;
        self.reload_restore_points()?;
        Ok(())
    }

    pub fn restore_from_point(&mut self, id: &str) -> Result<(), String> {
        self.restore_provider.restore(id)
    }

    pub fn list_driver_backups(&self) -> Result<Vec<DriverBackup>, String> {
        self.backup_provider.list()
    }

    pub fn create_driver_backup(&self) -> Result<DriverBackup, String> {
        self.backup_provider.create()
    }

    pub fn restore_driver_backup(&self, id: &str) -> Result<(), String> {
        self.backup_provider.restore(id)
    }

    pub fn delete_driver_backup(&self, id: &str) -> Result<(), String> {
        self.backup_provider.delete(id)
    }

    pub fn save_settings(&mut self, mut settings: Settings) -> Result<Settings, String> {
        settings.backup_path = self.backup_dir.to_string_lossy().into_owned();
        self.settings = settings;
        self.persist()?;
        Ok(self.settings.clone())
    }
}
