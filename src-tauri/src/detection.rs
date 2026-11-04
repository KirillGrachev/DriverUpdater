//! Windows detection adapters and parsing helpers.
//!
//! <p>All Windows-specific CIM, registry and process probing lives here. The
//! repository depends on the traits below, not on PowerShell, which keeps the
//! service testable and the native boundary explicit.</p>

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::models::{Driver, InstalledComponent, LocalizedText, SystemInfo};

/// Detects installed device drivers.
pub trait DriverDetector: Send + Sync {
    fn detect(&self) -> Option<Vec<Driver>>;
}

/// Detects installed runtimes and developer tools.
pub trait ComponentDetector: Send + Sync {
    fn detect(&self) -> Vec<InstalledComponent>;
}

/// Reads basic operating-system and host information.
pub trait SystemInfoProvider: Send + Sync {
    fn info(&self) -> SystemInfo;
}

/// Native Windows implementation of all detection ports.
#[derive(Default)]
pub struct NativeDetector;

impl DriverDetector for NativeDetector {
    fn detect(&self) -> Option<Vec<Driver>> {
        detect_installed_drivers()
    }
}

impl ComponentDetector for NativeDetector {
    fn detect(&self) -> Vec<InstalledComponent> {
        detect_installed_components()
    }
}

impl SystemInfoProvider for NativeDetector {
    fn info(&self) -> SystemInfo {
        detect_system_info()
    }
}

pub(crate) fn days_to_civil(z: u64) -> (i64, u32, u32) {
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
