//! Mock driver repository.
//!
//! TODO(real-hardware): replace with the actual detection backend.
//! The repository is intentionally hidden behind plain methods so the
//! command layer (`commands.rs`) does not care where the data comes from.
//!
//! Fixtures are shared with the webview: `shared/fixtures/*.json` is the
//! single source of test data for both halves of the app.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{
    CheckUpdatesResult, Driver, InitialData, LocalizedText, RestorePoint, Settings, Summary,
    SystemInfo,
};

const FIXTURE_DRIVERS: &str = include_str!("../../shared/fixtures/drivers.json");
const FIXTURE_RESTORE_POINTS: &str = include_str!("../../shared/fixtures/restore_points.json");
const FIXTURE_SETTINGS: &str = include_str!("../../shared/fixtures/settings.json");
const FIXTURE_META: &str = include_str!("../../shared/fixtures/meta.json");

#[derive(serde::Serialize, serde::Deserialize)]
struct Meta {
    #[serde(rename = "devicesTotal")]
    devices_total: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedState {
    drivers: Vec<Driver>,
    restore_points: Vec<RestorePoint>,
    settings: Settings,
    last_check: Option<String>,
}

pub struct MockRepository {
    drivers: Vec<Driver>,
    restore_points: Vec<RestorePoint>,
    settings: Settings,
    last_check: Option<String>,
    devices_total: u32,
    state_file: Option<PathBuf>,
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC3339-ish UTC stamp; enough for test data.
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (hours, mins, sec) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let civil = days_to_civil(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        civil.0, civil.1, civil.2, hours, mins, sec
    )
}

/// Days since 1970-01-01 -> (year, month, day); Howard Hinnant's algorithm.
fn days_to_civil(z: u64) -> (i64, u32, u32) {
    let z = z as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

impl MockRepository {
    pub fn new(state_file: Option<PathBuf>) -> Self {
        let drivers: Vec<Driver> =
            serde_json::from_str(FIXTURE_DRIVERS).expect("drivers fixture must be valid");
        let restore_points: Vec<RestorePoint> =
            serde_json::from_str(FIXTURE_RESTORE_POINTS).expect("restore points fixture must be valid");
        let settings: Settings =
            serde_json::from_str(FIXTURE_SETTINGS).expect("settings fixture must be valid");
        let meta: Meta = serde_json::from_str(FIXTURE_META).expect("meta fixture must be valid");

        let mut repo = Self {
            drivers,
            restore_points,
            settings,
            last_check: Some(now_iso()),
            devices_total: meta.devices_total,
            state_file,
        };

        if let Some(path) = repo.state_file.clone() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<PersistedState>(&raw) {
                    repo.drivers = state.drivers;
                    repo.restore_points = state.restore_points;
                    repo.settings = state.settings;
                    repo.last_check = state.last_check;
                }
            }
        }
        repo
    }

    fn persist(&self) {
        let Some(path) = self.state_file.clone() else {
            return;
        };
        let state = PersistedState {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            last_check: self.last_check.clone(),
        };
        if let Ok(raw) = serde_json::to_string_pretty(&state) {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(path, raw);
        }
    }

    pub fn summary(&self) -> Summary {
        let updates_available = self
            .drivers
            .iter()
            .filter(|d| d.status == "update")
            .count() as u32;
        Summary {
            devices_total: self.devices_total,
            up_to_date: self.devices_total.saturating_sub(updates_available),
            updates_available,
            last_check: self.last_check.clone(),
        }
    }

    pub fn snapshot(&self) -> InitialData {
        InitialData {
            drivers: self.drivers.clone(),
            restore_points: self.restore_points.clone(),
            settings: self.settings.clone(),
            summary: self.summary(),
            system_info: detect_system_info(),
        }
    }

    pub fn check_updates(&mut self) -> CheckUpdatesResult {
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
            id: format!("rp-{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)),
            name,
            kind: "manual".to_string(),
            created_at: now_iso(),
            size_mb: 380 + (SystemTime::now()
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

    pub fn save_settings(&mut self, settings: Settings) -> Settings {
        self.settings = settings;
        self.persist();
        self.settings.clone()
    }
}

pub fn detect_system_info() -> SystemInfo {
    let os = detect_os();
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64".to_string(),
        other => other.to_string(),
    };
    let platform = if cfg!(target_os = "windows") {
        "Win32".to_string()
    } else {
        std::env::consts::OS.to_string()
    };
    SystemInfo { os, arch, platform }
}

fn detect_os() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(raw) = fs::read_to_string("/etc/os-release") {
            for line in raw.lines() {
                if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                    return value.trim_matches('"').to_string();
                }
            }
        }
        return "Linux".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        return "Windows 11 (10.0.26100)".to_string(); // TODO(real-hardware): read the real NT version
    }
    #[cfg(target_os = "macos")]
    {
        return "macOS".to_string();
    }
    #[allow(unreachable_code)]
    std::env::consts::OS.to_string()
}
