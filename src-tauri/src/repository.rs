//! Fixture-backed repository with real Windows detection.
//!
//! <p>The repository is hidden behind plain methods so the command layer
//! ({@code commands.rs}) does not care where the data comes from. Fixtures are
//! shared with the webview: {@code shared/fixtures/*.json} is the single source
//! of test data for both halves of the app.</p>

use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::models::{
    CheckUpdatesResult, Driver, InitialData, InstalledComponent, LocalizedText, RestorePoint,
    Settings, Summary, SystemInfo,
};

const FIXTURE_DRIVERS: &str = include_str!("../../shared/fixtures/drivers.json");
const FIXTURE_RESTORE_POINTS: &str = include_str!("../../shared/fixtures/restore_points.json");
const FIXTURE_SETTINGS: &str = include_str!("../../shared/fixtures/settings.json");

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedState {
    drivers: Vec<Driver>,
    restore_points: Vec<RestorePoint>,
    settings: Settings,
    last_check: Option<String>,
}

pub struct DriverRepository {
    drivers: Vec<Driver>,
    restore_points: Vec<RestorePoint>,
    settings: Settings,
    last_check: Option<String>,
    devices_total: u32,
    state_file: Option<PathBuf>,
    backup_dir: PathBuf,
    needs_scan: bool,
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

/// Converts days since 1970-01-01 to a civil date using Howard Hinnant's
/// algorithm.
///
/// @param z days since the Unix epoch
/// @return {@code (year, month, day)}
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

fn normalize_driver_date(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    if let Some(rest) = raw.strip_prefix("/Date(") {
        if let Some(end) = rest.find(')') {
            if let Ok(ms) = rest[..end].parse::<i64>() {
                let days = (ms / 1000).max(0) as u64 / 86_400;
                let (y, m, d) = days_to_civil(days);
                return format!("{y:04}-{m:02}-{d:02}");
            }
        }
    }
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if raw.contains('/') {
        let date_part = raw.split_whitespace().next().unwrap_or(raw);
        let parts: Vec<&str> = date_part.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(m), Ok(d), Ok(y)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                if (1990..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
                    return format!("{y:04}-{m:02}-{d:02}");
                }
            }
        }
    }
    if digits.len() >= 8 {
        let year = digits[0..4].parse::<i32>().ok();
        let month = digits[4..6].parse::<u32>().ok();
        let day = digits[6..8].parse::<u32>().ok();
        if let (Some(y), Some(m), Some(d)) = (year, month, day) {
            if (1990..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
                return format!("{y:04}-{m:02}-{d:02}");
            }
        }
    }
    String::new()
}

/// Runs a Windows PowerShell script with a hard timeout and no console window.
///
/// The output is drained on a separate thread so a large CIM/registry payload
/// cannot fill the pipe and deadlock the child while the timeout is pending.
///
/// @param script PowerShell command text
/// @param timeout maximum wall-clock time for the child process
/// @returns UTF-8 stdout on success, or `None` on timeout/failure
#[cfg(target_os = "windows")]
fn run_powershell(script: &str, timeout: Duration) -> Option<String> {
    use std::io::Read;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use wait_timeout::ChildExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut child = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer).ok().map(|_| buffer)
    });

    let status = child.wait_timeout(timeout).ok().flatten();
    let Some(status) = status else {
        let _ = child.kill();
        let _ = child.wait();
        let _ = reader.join();
        return None;
    };

    let bytes = reader.join().ok().flatten()?;
    if !status.success() {
        return None;
    }
    String::from_utf8(bytes).ok()
}

#[cfg(target_os = "windows")]
fn detect_installed_drivers() -> Option<Vec<Driver>> {
    let script = r#"[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceName -and $_.DriverVersion } | Select-Object DeviceName,DeviceClass,Manufacturer,DriverVersion,DriverDate,DeviceID | ConvertTo-Json -Compress"#;
    let stdout = run_powershell(script, Duration::from_secs(20))?;
    let value: serde_json::Value = serde_json::from_str(&stdout).ok()?;
    let rows: Vec<&serde_json::Value> = match &value {
        serde_json::Value::Array(items) => items.iter().collect(),
        serde_json::Value::Object(_) => vec![&value],
        _ => return None,
    };

    fn text(row: &serde_json::Value, key: &str) -> String {
        row.get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    let mut seen = HashSet::new();
    let mut drivers = Vec::new();
    for row in rows {
        let name = text(row, "DeviceName");
        let version = text(row, "DriverVersion");
        if name.is_empty() || version.is_empty() {
            continue;
        }
        let manufacturer = text(row, "Manufacturer");
        let dedupe_key = format!("{name}|{version}|{manufacturer}");
        if !seen.insert(dedupe_key.clone()) {
            continue;
        }

        let class = text(row, "DeviceClass").to_ascii_uppercase();
        let (category, kind, color) = match class.as_str() {
            "DISPLAY" => ("video", "gpu", "#76b900"),
            "MEDIA" => ("audio", "audio", "#64748b"),
            "NET" => ("network", "network", "#2563eb"),
            "HIDCLASS" | "KEYBOARD" | "MOUSE" => ("input", "input", "#8b5cf6"),
            _ => ("chipset", "chip", "#f59e0b"),
        };

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        dedupe_key.hash(&mut hasher);
        let id = format!("pnp-{:016x}", hasher.finish());
        let vendor = if manufacturer.is_empty() {
            "Unknown".to_string()
        } else {
            manufacturer
        };

        drivers.push(Driver {
            id,
            name: name.clone(),
            vendor,
            category: category.to_string(),
            icon: crate::models::DriverIcon {
                kind: kind.to_string(),
                color: color.to_string(),
            },
            current: version,
            available: None,
            status: "current".to_string(),
            size_mb: 0,
            recommended: false,
            device: name.clone(),
            released: normalize_driver_date(&text(row, "DriverDate")),
            description: LocalizedText {
                ru: format!("Драйвер устройства {name}, установленный в системе."),
                en: format!("Installed device driver for {name}."),
            },
        });
    }

    if drivers.is_empty() {
        None
    } else {
        drivers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Some(drivers)
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_installed_drivers() -> Option<Vec<Driver>> {
    None
}

fn driver_cache_path(state_file: Option<&std::path::Path>) -> Option<PathBuf> {
    state_file
        .and_then(|path| path.parent())
        .map(|directory| directory.join("installed-drivers.json"))
}

fn read_cached_drivers(state_file: Option<&std::path::Path>) -> Option<Vec<Driver>> {
    let path = driver_cache_path(state_file)?;
    let raw = fs::read_to_string(path).ok()?;
    let drivers: Vec<Driver> = serde_json::from_str(&raw).ok()?;
    (!drivers.is_empty()).then_some(drivers)
}

fn write_cached_drivers(state_file: Option<&std::path::Path>, drivers: &[Driver]) {
    let Some(path) = driver_cache_path(state_file) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(drivers) {
        let _ = fs::write(path, raw);
    }
}

impl DriverRepository {
    pub fn new(state_file: Option<PathBuf>, backup_dir: PathBuf) -> Self {
        let cached = read_cached_drivers(state_file.as_deref());
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
            state_file,
            backup_dir,
            needs_scan,
        };

        if let Some(path) = repo.state_file.clone() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<PersistedState>(&raw) {
                    // A real hardware scan always wins over old test fixtures.
                    if cached.is_none() {
                        repo.drivers = state.drivers;
                    }
                    repo.restore_points = state.restore_points;
                    repo.settings = state.settings;
                    repo.settings.backup_path = repo.backup_dir.to_string_lossy().into_owned();
                    repo.last_check = state.last_check;
                }
            }
        }
        repo.devices_total = repo.drivers.len() as u32;
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
            // Write-then-rename so a crash cannot leave a truncated state file.
            let tmp = path.with_extension("json.tmp");
            if fs::write(&tmp, &raw).is_ok() {
                if fs::rename(&tmp, &path).is_err() {
                    let _ = fs::write(&path, raw);
                    let _ = fs::remove_file(&tmp);
                }
            }
        }
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

    /// Scans Windows once when no fresh driver cache exists.
    ///
    /// The setup hook only builds the in-memory repository; the actual CIM
    /// query runs in the blocking pool on the first IPC request.
    pub fn ensure_fresh(&mut self) {
        if !self.needs_scan {
            return;
        }
        self.needs_scan = false;
        let Some(drivers) = detect_installed_drivers() else {
            return;
        };
        write_cached_drivers(self.state_file.as_deref(), &drivers);
        self.drivers = drivers;
        self.devices_total = self.drivers.len() as u32;
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
        // Rescan the machine so newly connected devices and updated drivers
        // are reflected without restarting the app.
        if let Some(drivers) = detect_installed_drivers() {
            write_cached_drivers(self.state_file.as_deref(), &drivers);
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

fn component_id_for(display_name: &str) -> Option<&'static str> {
    let name = display_name.to_lowercase();
    if name.contains("7-zip") {
        Some("seven-zip-24")
    } else if name.contains("python") {
        Some("python-3-12")
    } else if name.contains("node.js") {
        Some("nodejs-22")
    } else if name.contains("openjdk") || name.contains("adoptium") || name.contains("temurin") {
        Some("openjdk-21")
    } else if name.contains("go programming language") || name == "go" {
        Some("go-1-23")
    } else if name.contains("rustup") || (name.contains("rust") && name.contains("toolchain")) {
        Some("rust-1-81")
    } else if name.contains("php ") {
        Some("php-8-3")
    } else if name == "git" || name.starts_with("git version") || name.starts_with("git ") {
        Some("git-2-46")
    } else if name.contains("cmake") {
        Some("cmake-3-30")
    } else if name.contains("visual c++") && name.contains("redistributable") {
        Some("vcredist-2015-2022")
    } else if name.contains(".net") && name.contains("framework") {
        Some("dotnet-framework-4-8")
    } else if name.contains("windows desktop runtime") || name.contains("microsoft .net runtime") {
        Some("dotnet-desktop-8")
    } else if name.contains("directx") {
        Some("directx-end-user")
    } else {
        None
    }
}

fn extract_version(text: &str) -> Option<String> {
    let mut best = None;
    let mut current = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            let parts: Vec<&str> = current.split('.').filter(|part| !part.is_empty()).collect();
            if parts.len() >= 2 {
                best = Some(current.clone());
            }
            current.clear();
        }
    }
    best
}

#[cfg(target_os = "windows")]
pub fn detect_installed_components() -> Vec<InstalledComponent> {
    let script = r#"$ErrorActionPreference='SilentlyContinue'; [Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' | Where-Object { $_.DisplayName } | Select-Object DisplayName,DisplayVersion,InstallLocation,DisplayIcon | ConvertTo-Json -Compress"#;
    let Some(stdout) = run_powershell(script, Duration::from_secs(20)) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) else {
        return Vec::new();
    };
    let rows: Vec<&serde_json::Value> = match &value {
        serde_json::Value::Array(items) => items.iter().collect(),
        serde_json::Value::Object(_) => vec![&value],
        _ => return Vec::new(),
    };

    let mut seen = HashSet::new();
    let mut components = Vec::new();
    for row in rows {
        let display_name = row
            .get("DisplayName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let Some(id) = component_id_for(display_name) else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let version = extract_version(display_name)
            .or_else(|| {
                row.get("DisplayVersion")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
            })
            .unwrap_or_default();
        let install_location = row
            .get("InstallLocation")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let display_icon = row
            .get("DisplayIcon")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let path = if !install_location.is_empty() {
            install_location.to_string()
        } else {
            display_icon
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        };
        components.push(InstalledComponent {
            id: id.to_string(),
            version,
            path,
        });
    }
    components
}

#[cfg(not(target_os = "windows"))]
pub fn detect_installed_components() -> Vec<InstalledComponent> {
    Vec::new()
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
    SystemInfo {
        os,
        arch,
        platform,
        hostname: std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "Unknown".to_string()),
    }
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
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

        if let Ok(output) = std::process::Command::new("cmd")
            .args(["/C", "ver"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let raw = String::from_utf8_lossy(&output.stdout);
            if let (Some(start), Some(end)) = (raw.find("Version "), raw.find(']')) {
                let version = raw[start + "Version ".len()..end].trim();
                let build = version
                    .split('.')
                    .nth(2)
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(0);
                let edition = if build >= 22_000 {
                    "Windows 11"
                } else if version.starts_with("10.0") {
                    "Windows 10"
                } else if version.starts_with("6.3") {
                    "Windows 8.1"
                } else {
                    "Windows"
                };
                return format!("{edition} ({version})");
            }
        }
        return "Windows".to_string();
    }
    #[cfg(target_os = "macos")]
    {
        return "macOS".to_string();
    }
    #[allow(unreachable_code)]
    std::env::consts::OS.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dotnet_dates() {
        assert_eq!(normalize_driver_date("/Date(1735689600000)/"), "2025-01-01");
    }

    #[test]
    fn normalizes_slash_dates() {
        assert_eq!(normalize_driver_date("10/15/2024"), "2024-10-15");
    }

    #[test]
    fn normalizes_compact_dates() {
        assert_eq!(normalize_driver_date("20240704"), "2024-07-04");
    }

    #[test]
    fn rejects_invalid_dates() {
        assert_eq!(normalize_driver_date("not a date"), "");
    }

    #[test]
    fn maps_known_components() {
        assert_eq!(
            component_id_for("Python 3.12.4 (64-bit)"),
            Some("python-3-12")
        );
        assert_eq!(component_id_for("7-Zip 24.09 (x64)"), Some("seven-zip-24"));
        assert_eq!(
            component_id_for("Microsoft Visual C++ 2015-2022 Redistributable"),
            Some("vcredist-2015-2022")
        );
        assert_eq!(component_id_for("Unrelated Tool"), None);
    }

    #[test]
    fn extracts_trailing_versions() {
        assert_eq!(
            extract_version("Python 3.12.4 (64-bit)").as_deref(),
            Some("3.12.4")
        );
        assert_eq!(
            extract_version("7-Zip 24.09 (x64)").as_deref(),
            Some("24.09")
        );
        assert_eq!(extract_version("no version here"), None);
    }

    #[test]
    fn converts_unix_epoch_to_civil_date() {
        assert_eq!(days_to_civil(0), (1970, 1, 1));
    }
}
