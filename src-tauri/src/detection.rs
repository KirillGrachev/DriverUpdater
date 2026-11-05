//! Windows detection adapters and parsing helpers.
//!
//! <p>All Windows-specific CIM, registry and process probing lives here. The
//! repository depends on the traits below, not on PowerShell, which keeps the
//! service testable and the native boundary explicit.</p>

#[cfg(target_os = "windows")]
use std::collections::HashSet;
#[cfg(target_os = "windows")]
use std::time::Duration;

use crate::models::{Driver, InstalledComponent, SystemInfo};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};

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

#[cfg(target_os = "windows")]
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

/// Runs a Windows PowerShell script with a hard timeout.
///
/// @param script PowerShell command text
/// @param timeout maximum wall-clock time for the child process
/// @returns stdout on success, or `None` on timeout/failure
#[cfg(target_os = "windows")]
fn run_powershell(script: &str, timeout: Duration) -> Option<String> {
    let output = crate::process::run_powershell(script, timeout).ok()?;
    output.success.then_some(output.stdout)
}

#[cfg(target_os = "windows")]
fn detect_installed_drivers() -> Option<Vec<Driver>> {
    let script = r#"[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceName -and $_.DriverVersion } | Select-Object DeviceName,DeviceClass,Manufacturer,DriverVersion,DriverDate,DeviceID,HardwareID,InfName | ConvertTo-Json -Compress"#;
    let stdout = run_powershell(script, Duration::from_secs(90))?;
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

    fn text_list(row: &serde_json::Value, key: &str) -> Vec<String> {
        match row.get(key) {
            Some(serde_json::Value::String(value)) => value
                .split(['\n', ','])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect(),
            Some(serde_json::Value::Array(values)) => values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect(),
            _ => Vec::new(),
        }
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
        let device_id = text(row, "DeviceID");
        let hardware_ids = text_list(row, "HardwareID");
        if is_ignored_device(&device_id, &hardware_ids) {
            continue;
        }
        let stable_key = if device_id.is_empty() {
            format!("{name}|{version}|{manufacturer}")
        } else {
            device_id.to_ascii_lowercase()
        };
        if !seen.insert(stable_key.clone()) {
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

        let digest = Sha256::digest(stable_key.as_bytes());
        let id = format!(
            "pnp-{}",
            digest[..8]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
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
            device_id,
            hardware_ids,
            inf_name: text(row, "InfName"),
            update_id: None,
            reboot_required: false,
        });
    }

    if drivers.is_empty() {
        None
    } else {
        drivers.sort_by_key(|driver| driver.name.to_lowercase());
        Some(drivers)
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_installed_drivers() -> Option<Vec<Driver>> {
    None
}

/// Driver Booster-style filter for devices that cannot receive a useful driver update.
pub(crate) fn is_ignored_device(device_id: &str, hardware_ids: &[String]) -> bool {
    const IGNORED: &[&str] = &[
        "ACPI\\FIXEDBUTTON",
        "ACPIAPIC",
        "GENCDROM",
        "GENDISK",
        "UMB\\UMBUS",
        "DETECTEDINTERNAL\\ACPI_HAL",
        "DETECTED\\ACPI_HAL",
    ];
    let mut values = vec![device_id.to_ascii_uppercase()];
    values.extend(hardware_ids.iter().map(|value| value.to_ascii_uppercase()));
    values
        .iter()
        .any(|value| IGNORED.iter().any(|ignored| value.starts_with(ignored)))
}

/* Component mapping starts here. */
#[cfg(target_os = "windows")]
fn component_id_for(display_name: &str, version: &str) -> Option<&'static str> {
    let name = display_name.to_lowercase();
    let parts: Vec<u32> = version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u32>().ok())
        .collect();
    let major = parts.first().copied().unwrap_or(0);
    let minor = parts.get(1).copied().unwrap_or(0);

    if name.contains("7-zip") {
        Some("seven-zip")
    } else if name.contains("python") {
        match (major, minor) {
            (3, 10) => Some("python-3-10"),
            (3, 11) => Some("python-3-11"),
            (3, 12) => Some("python-3-12"),
            (3, 13) => Some("python-3-13"),
            _ => None,
        }
    } else if name.contains("node.js") {
        match major {
            20 => Some("nodejs-20"),
            22 => Some("nodejs-22"),
            _ => None,
        }
    } else if name.contains("openjdk") || name.contains("adoptium") || name.contains("temurin") {
        match major {
            17 => Some("openjdk-17"),
            21 => Some("openjdk-21"),
            25 => Some("openjdk-25"),
            _ => None,
        }
    } else if name.contains("desktop runtime") || name.contains("windows desktop runtime") {
        match major {
            6 => Some("dotnet-desktop-6"),
            8 => Some("dotnet-desktop-8"),
            9 => Some("dotnet-desktop-9"),
            _ => None,
        }
    } else if name.contains("go programming language") || name == "go" {
        Some("go")
    } else if name.contains("rustup") || (name.contains("rust") && name.contains("toolchain")) {
        Some("rust")
    } else if name.contains("php ") {
        match (major, minor) {
            (8, 2) => Some("php-8-2"),
            (8, 3) => Some("php-8-3"),
            (8, 4) => Some("php-8-4"),
            _ => None,
        }
    } else if name == "git" || name.starts_with("git version") || name.starts_with("git ") {
        Some("git")
    } else if name.contains("cmake") {
        Some("cmake")
    } else if name.contains("visual c++") && name.contains("redistributable") {
        if name.contains("x86") {
            Some("vcredist-x86")
        } else {
            Some("vcredist-x64")
        }
    } else if name.contains(".net") && name.contains("framework") && name.contains("3.5") {
        Some("dotnet-framework-3-5")
    } else if name.contains(".net") && name.contains("framework") {
        Some("dotnet-framework")
    } else if name.contains("visual studio") {
        if name.contains("community") {
            Some("visual-studio-community")
        } else if name.contains("professional") {
            Some("visual-studio-professional")
        } else if name.contains("enterprise") {
            Some("visual-studio-enterprise")
        } else {
            None
        }
    } else if name.contains("directx") {
        Some("directx")
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
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
    let script = r#"$ErrorActionPreference='SilentlyContinue'; [Console]::OutputEncoding=[Text.Encoding]::UTF8; $rows = @(Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' | Where-Object { $_.DisplayName } | Select-Object DisplayName,DisplayVersion,InstallLocation,DisplayIcon); $netfx = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\NET Framework Setup\NDP\v3.5' -ErrorAction SilentlyContinue; if ($netfx -and $netfx.Install -eq 1) { $rows += [pscustomobject]@{ DisplayName='.NET Framework 3.5'; DisplayVersion='3.5 SP1'; InstallLocation=''; DisplayIcon='' } }; ConvertTo-Json -InputObject $rows -Compress"#;
    let Some(stdout) = run_powershell(script, Duration::from_secs(90)) else {
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
        let version = row
            .get("DisplayVersion")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .or_else(|| extract_version(display_name))
            .unwrap_or_default();
        let Some(id) = component_id_for(display_name, &version) else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
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
    #[cfg(target_os = "windows")]
    let platform = "Win32".to_string();
    #[cfg(not(target_os = "windows"))]
    let platform = std::env::consts::OS.to_string();
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
        if let Ok(raw) = std::fs::read_to_string("/etc/os-release") {
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

        if let Ok(output) = std::process::Command::new(crate::process::system_executable("cmd.exe"))
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

#[cfg(all(test, target_os = "windows"))]
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
    fn filters_firmware_pseudo_devices() {
        assert!(is_ignored_device("ACPI\\FIXEDBUTTON\\2&DABA3FF&0", &[]));
        assert!(is_ignored_device("", &["GENDISK".to_string()]));
        assert!(!is_ignored_device(
            "PCI\\VEN_10DE&DEV_2F04",
            &["PCI\\VEN_10DE&DEV_2F04".to_string()]
        ));
    }

    #[test]
    fn maps_known_components() {
        assert_eq!(
            component_id_for("Python 3.12.4 (64-bit)", "3.12.4"),
            Some("python-3-12")
        );
        assert_eq!(
            component_id_for("7-Zip 24.09 (x64)", "24.09"),
            Some("seven-zip")
        );
        assert_eq!(
            component_id_for(
                "Microsoft Visual C++ 2015-2022 Redistributable (x64)",
                "14.44.35211"
            ),
            Some("vcredist-x64")
        );
        assert_eq!(component_id_for("Unrelated Tool", "1.0.0"), None);
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
