//! Fixture-backed repository with real Windows detection.
//!
//! <p>The repository is hidden behind plain methods so the command layer
//! ({@code commands.rs}) does not care where the data comes from. Fixtures are
//! shared with the webview: {@code shared/fixtures/*.json} is the single source
//! of test data for both halves of the app.</p>

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::detection::{
    days_to_civil, ComponentDetector, DriverDetector, NativeDetector, SystemInfoProvider,
};
use crate::models::{
    CheckUpdatesResult, Driver, InitialData, LocalizedText, RestorePoint, Settings, Summary,
};
use crate::storage::{DriverCache, EncryptedStore, PersistedState};

const FIXTURE_DRIVERS: &str = include_str!("../../shared/fixtures/drivers.json");
const FIXTURE_RESTORE_POINTS: &str = include_str!("../../shared/fixtures/restore_points.json");
const FIXTURE_SETTINGS: &str = include_str!("../../shared/fixtures/settings.json");

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
    installed_components: Vec<String>,
    detector: Box<dyn DriverDetector>,
    component_detector: Box<dyn ComponentDetector>,
    system_info: Box<dyn SystemInfoProvider>,
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC3339-like UTC timestamp; precise enough for persisted test data.
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
        Self::with_detectors(
            state_file,
            backup_dir,
            Box::<NativeDetector>::default(),
            Box::<NativeDetector>::default(),
            Box::<NativeDetector>::default(),
        )
    }

    fn with_detectors(
        state_file: Option<PathBuf>,
        backup_dir: PathBuf,
        detector: Box<dyn DriverDetector>,
        component_detector: Box<dyn ComponentDetector>,
        system_info: Box<dyn SystemInfoProvider>,
    ) -> Self {
        let store = EncryptedStore::new(state_file);
        let cache = DriverCache::new(store.path());
        let cached = cache.load();
        let needs_scan = cached.is_none();
        let drivers: Vec<Driver> = cached.clone().unwrap_or_else(|| {
            serde_json::from_str(FIXTURE_DRIVERS).expect("drivers fixture must be valid")
        });
        let restore_points: Vec<RestorePoint> = serde_json::from_str(FIXTURE_RESTORE_POINTS)
            .expect("restore points fixture must be valid");
        let mut settings: Settings =
            serde_json::from_str(FIXTURE_SETTINGS).expect("settings fixture must be valid");

        // The backup folder is derived from the install environment, never
        // taken from the fixture or from a stale persisted value.
        settings.backup_path = backup_dir.to_string_lossy().into_owned();

        let mut repo = Self {
            drivers,
            restore_points,
            settings,
            last_check: Some(now_iso()),
            devices_total: 0,
            store,
            cache,
            backup_dir,
            needs_scan,
            installed_components: Vec::new(),
            detector,
            component_detector,
            system_info,
        };

        if let Some(state) = repo.store.load::<PersistedState>() {
            // A real hardware scan always wins over old test fixtures.
            if cached.is_none() {
                repo.drivers = state.drivers;
            }
            repo.restore_points = state.restore_points;
            repo.settings = state.settings;
            repo.settings.backup_path = repo.backup_dir.to_string_lossy().into_owned();
            repo.last_check = state.last_check;
            repo.installed_components = state.installed_components;
        }
        repo.devices_total = repo.drivers.len() as u32;
        repo
    }

    fn persist(&self) {
        let state = PersistedState {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            last_check: self.last_check.clone(),
            installed_components: self.installed_components.clone(),
        };
        let _ = self.store.save(&state);
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
    pub fn save_installed_components(&mut self, ids: Vec<String>) -> Vec<String> {
        let mut unique = Vec::new();
        for id in ids {
            if !id.is_empty() && !unique.contains(&id) {
                unique.push(id);
            }
        }
        self.installed_components = unique.clone();
        self.persist();
        unique
    }

    /// Scans Windows once when no fresh driver cache exists.
    ///
    /// The setup hook only builds the in-memory repository; the actual CIM
    /// query runs in the blocking pool on the first IPC request.
    pub fn ensure_fresh(&mut self) {
        if !self.needs_scan {
            return;
        }
        self.needs_scan = false;
        let Some(drivers) = self.detector.detect() else {
            return;
        };
        self.cache.save(&drivers);
        self.drivers = drivers;
        self.devices_total = self.drivers.len() as u32;
    }

    pub fn snapshot(&self) -> InitialData {
        InitialData {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            summary: self.summary(),
            system_info: self.system_info.info(),
        }
    }

    pub fn check_updates(&mut self) -> CheckUpdatesResult {
        // Rescan the machine so newly connected devices and updated drivers
        // are reflected without restarting the app.
        if let Some(drivers) = self.detector.detect() {
            self.cache.save(&drivers);
            self.drivers = drivers;
            self.devices_total = self.drivers.len() as u32;
        }
        self.needs_scan = false;
        self.last_check = Some(now_iso());
        self.persist();
        CheckUpdatesResult {
            drivers: self.drivers.clone(),
            summary: self.summary(),
        }
    }

    fn apply_install(driver: &mut Driver) {
        if let Some(available) = driver.available.take() {
            driver.current = available;
        }
        driver.status = "current".to_string();
    }

    pub fn install_driver(&mut self, id: &str) -> Result<Driver, String> {
        let driver = self
            .drivers
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("Unknown driver: {id}"))?;
        Self::apply_install(driver);
        let cloned = driver.clone();
        self.persist();
        Ok(cloned)
    }

    pub fn install_updates(&mut self, ids: &[String]) -> Result<Vec<Driver>, String> {
        let mut updated = Vec::new();
        for id in ids {
            let driver = self
                .drivers
                .iter_mut()
                .find(|d| &d.id == id)
                .ok_or_else(|| format!("Unknown driver: {id}"))?;
            Self::apply_install(driver);
            updated.push(driver.clone());
        }
        self.last_check = Some(now_iso());
        self.persist();
        Ok(updated)
    }

    pub fn restore_points(&self) -> Vec<RestorePoint> {
        self.restore_points.clone()
    }

    pub fn create_restore_point(&mut self, name: LocalizedText) -> RestorePoint {
        let point = RestorePoint {
            id: format!(
                "rp-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            ),
            name,
            kind: "manual".to_string(),
            created_at: now_iso(),
            size_mb: 380
                + (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() % 120)
                    .unwrap_or(0)) as u32,
        };
        self.restore_points.insert(0, point.clone());
        self.persist();
        point
    }

    pub fn delete_restore_point(&mut self, id: &str) -> Result<(), String> {
        let before = self.restore_points.len();
        self.restore_points.retain(|p| p.id != id);
        if self.restore_points.len() == before {
            return Err(format!("Unknown restore point: {id}"));
        }
        self.persist();
        Ok(())
    }

    pub fn restore_from_point(&self, id: &str) -> Result<(), String> {
        if self.restore_points.iter().any(|p| p.id == id) {
            Ok(())
        } else {
            Err(format!("Unknown restore point: {id}"))
        }
    }

    pub fn save_settings(&mut self, mut settings: Settings) -> Settings {
        settings.backup_path = self.backup_dir.to_string_lossy().into_owned();
        self.settings = settings;
        self.persist();
        self.settings.clone()
    }
}
