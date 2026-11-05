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
    detect_installed_drivers_native().or_else(detect_installed_drivers_powershell)
}

#[cfg(target_os = "windows")]
fn detect_installed_drivers_powershell() -> Option<Vec<Driver>> {
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

#[cfg(target_os = "windows")]
fn detect_installed_drivers_native() -> Option<Vec<Driver>> {
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW, DIGCF_ALLCLASSES,
        DIGCF_PRESENT, SPDRP_CLASS, SPDRP_DEVICEDESC, SPDRP_DRIVER, SPDRP_FRIENDLYNAME,
        SPDRP_HARDWAREID, SPDRP_MFG, SP_DEVINFO_DATA,
    };

    struct DeviceSet(windows_sys::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO);

    impl Drop for DeviceSet {
        fn drop(&mut self) {
            unsafe {
                SetupDiDestroyDeviceInfoList(self.0);
            }
        }
    }

    fn property(
        set: windows_sys::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        info: &SP_DEVINFO_DATA,
        property: u32,
        multiple: bool,
    ) -> Option<Vec<String>> {
        let mut buffer = vec![0u8; 8192];
        let mut required = 0u32;
        let ok = unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                set,
                info,
                property,
                null_mut(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                &mut required,
            )
        };
        if ok == 0 {
            return None;
        }
        let size = (required as usize).min(buffer.len());
        let wide: Vec<u16> = buffer[..size]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        let values: Vec<String> = wide
            .split(|unit| *unit == 0)
            .filter(|part| !part.is_empty())
            .filter_map(|part| String::from_utf16(part).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        if values.is_empty() {
            return None;
        }
        if multiple {
            Some(values)
        } else {
            Some(vec![values.join(" ")])
        }
    }

    fn driver_details(driver_key: &str) -> (String, String) {
        use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let Ok(key) = hklm.open_subkey_with_flags(
            format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{driver_key}"),
            KEY_READ,
        ) else {
            return (String::new(), String::new());
        };
        let version = key
            .get_value::<String, _>("DriverVersion")
            .unwrap_or_default();
        let date = key.get_value::<String, _>("DriverDate").unwrap_or_default();
        (version.trim().to_string(), date.trim().to_string())
    }

    let set = unsafe {
        SetupDiGetClassDevsW(null(), null(), null_mut(), DIGCF_PRESENT | DIGCF_ALLCLASSES)
    };
    if set == 0 || set == -1 {
        return None;
    }
    let set = DeviceSet(set);
    let mut seen = HashSet::new();
    let mut drivers = Vec::new();
    let mut index = 0u32;
    loop {
        let mut info = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        if unsafe { SetupDiEnumDeviceInfo(set.0, index, &mut info) } == 0 {
            break;
        }
        index += 1;

        let mut instance = vec![0u16; 1024];
        if unsafe {
            SetupDiGetDeviceInstanceIdW(
                set.0,
                &info,
                instance.as_mut_ptr(),
                instance.len() as u32,
                null_mut(),
            )
        } == 0
        {
            continue;
        }
        let device_id = String::from_utf16_lossy(&instance)
            .trim_end_matches('\0')
            .to_string();

        let name = property(set.0, &info, SPDRP_FRIENDLYNAME, false)
            .and_then(|values| values.into_iter().next())
            .or_else(|| {
                property(set.0, &info, SPDRP_DEVICEDESC, false).and_then(|v| v.into_iter().next())
            })
            .unwrap_or_default();
        let driver_key = property(set.0, &info, SPDRP_DRIVER, false)
            .and_then(|values| values.into_iter().next())
            .unwrap_or_default();
        let (version, released) = driver_details(&driver_key);
        if name.is_empty() || version.is_empty() {
            #[cfg(test)]
            eprintln!("native skip name/version: {device_id} name={name:?} version={version:?}");
            continue;
        }
        let hardware_ids = property(set.0, &info, SPDRP_HARDWAREID, true).unwrap_or_default();
        if is_ignored_device(&device_id, &hardware_ids) {
            #[cfg(test)]
            eprintln!("native skip ignored: {device_id}");
            continue;
        }
        let stable_key = device_id.to_ascii_lowercase();
        if !seen.insert(stable_key.clone()) {
            continue;
        }
        let class = property(set.0, &info, SPDRP_CLASS, false)
            .and_then(|values| values.into_iter().next())
            .unwrap_or_default()
            .to_ascii_uppercase();
        let manufacturer = property(set.0, &info, SPDRP_MFG, false)
            .and_then(|values| values.into_iter().next())
            .unwrap_or_default();
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
        drivers.push(Driver {
            id,
            name: name.clone(),
            vendor: if manufacturer.is_empty() {
                "Unknown".to_string()
            } else {
                manufacturer
            },
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
            device: name,
            released: normalize_driver_date(&released),
            device_id,
            hardware_ids,
            inf_name: String::new(),
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
fn component_id_for(display_name: &str, version: &str, publisher: &str) -> Option<&'static str> {
    let name = display_name.to_lowercase();
    let publisher = publisher.to_lowercase();
    let publisher_matches = |expected: &str| publisher.is_empty() || publisher.contains(expected);
    let parts: Vec<u32> = version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u32>().ok())
        .collect();
    let major = parts.first().copied().unwrap_or(0);
    let minor = parts.get(1).copied().unwrap_or(0);

    if name.contains("visual studio code") && publisher_matches("microsoft") {
        return Some("vs-code");
    }
    if (name.contains("buildtools") || name.contains("build tools"))
        && publisher_matches("microsoft")
    {
        return Some("vs-buildtools-2022");
    }
    if (name.contains("windows software development kit") || name.contains("windows sdk"))
        && publisher_matches("microsoft")
    {
        return Some("windows-sdk");
    }
    if name.contains("vulkan sdk") {
        return Some("vulkan-sdk");
    }
    if name.contains("vulkan") {
        return Some("vulkan-runtime");
    }
    if name.contains("webview2") && publisher_matches("microsoft") {
        return Some("webview2");
    }
    if name.contains("xna") && publisher_matches("microsoft") {
        return Some("xna");
    }
    if name.contains("physx") && publisher_matches("nvidia") {
        return Some("physx");
    }
    if name.contains("cuda") && publisher_matches("nvidia") {
        return Some("cuda");
    }
    if name.contains("openal") {
        return Some("openal");
    }
    if name == "ruby" || name.starts_with("ruby ") {
        return Some("ruby");
    }
    if name.contains("dart sdk") {
        return Some("dart");
    }
    if name.contains("strawberry perl") || name == "perl" || name.starts_with("perl ") {
        return Some("perl");
    }
    if name.starts_with("r for windows") || name == "r" {
        return Some("r");
    }
    if name == "bun" || name.starts_with("bun ") {
        return Some("bun");
    }
    if name == "deno" || name.starts_with("deno ") {
        return Some("deno");
    }
    if name == "swift" || name.starts_with("swift ") {
        return Some("swift");
    }
    if name.contains("llvm") {
        return Some("llvm");
    }
    if name.contains("docker desktop") {
        return Some("docker");
    }
    if name == "ninja" || name.starts_with("ninja ") {
        return Some("ninja");
    }
    if name.contains("powershell 7") {
        return Some("powershell");
    }
    if name.contains("windows subsystem for linux") {
        return Some("wsl");
    }
    if name.contains("github cli") {
        return Some("github-cli");
    }
    if name.contains("windows terminal") {
        return Some("windows-terminal");
    }
    if name == "nasm" || name.starts_with("nasm ") {
        return Some("nasm");
    }
    if name.starts_with("postgresql") {
        return Some("postgresql");
    }
    if name.starts_with("mysql") {
        return Some("mysql");
    }
    if name.starts_with("sqlite") {
        return Some("sqlite");
    }
    if name.starts_with("chocolatey") {
        return Some("chocolatey");
    }
    if name.contains("asp.net core") {
        return match major {
            8 => Some("aspnetcore-8"),
            9 => Some("aspnetcore-9"),
            10 => Some("aspnetcore-10"),
            _ => None,
        };
    }
    if name.contains("dotnet sdk") || name.contains(".net sdk") {
        return match major {
            10 => Some("dotnet-sdk-10"),
            _ => None,
        };
    }

    if name.contains("7-zip") && publisher_matches("igor pavlov") {
        Some("seven-zip")
    } else if name.contains("python")
        && !name.contains("launcher")
        && publisher_matches("python software foundation")
    {
        match (major, minor) {
            (3, 10) => Some("python-3-10"),
            (3, 11) => Some("python-3-11"),
            (3, 12) => Some("python-3-12"),
            (3, 13) => Some("python-3-13"),
            (3, 14) => Some("python-3-14"),
            _ => None,
        }
    } else if name.contains("node.js") && publisher_matches("openjs") {
        match major {
            20 => Some("nodejs-20"),
            22 => Some("nodejs-22"),
            _ => None,
        }
    } else if (name.contains("openjdk") || name.contains("adoptium") || name.contains("temurin"))
        && (publisher.is_empty() || publisher.contains("eclipse") || publisher.contains("adoptium"))
    {
        match major {
            17 => Some("openjdk-17"),
            21 => Some("openjdk-21"),
            25 => Some("openjdk-25"),
            _ => None,
        }
    } else if (name.contains("desktop runtime") || name.contains("windows desktop runtime"))
        && publisher_matches("microsoft")
    {
        match major {
            6 => Some("dotnet-desktop-6"),
            8 => Some("dotnet-desktop-8"),
            9 => Some("dotnet-desktop-9"),
            10 => Some("dotnet-desktop-10"),
            _ => None,
        }
    } else if name.contains("go programming language") || name == "go" {
        Some("go")
    } else if name.contains("rustup") || (name.contains("rust") && name.contains("toolchain")) {
        Some("rust")
    } else if name.contains("php ") && publisher_matches("php group") {
        match (major, minor) {
            (8, 2) => Some("php-8-2"),
            (8, 3) => Some("php-8-3"),
            (8, 4) => Some("php-8-4"),
            (8, 5) => Some("php-8-5"),
            _ => None,
        }
    } else if name == "git"
        || name.starts_with("git version")
        || (name.starts_with("git ") && publisher_matches("git development community"))
    {
        Some("git")
    } else if name.contains("cmake") && publisher_matches("kitware") {
        Some("cmake")
    } else if name.contains("visual c++")
        && name.contains("redistributable")
        && publisher_matches("microsoft")
    {
        if name.contains("arm64") {
            Some("vcredist-arm64")
        } else if name.contains("x86") {
            Some("vcredist-x86")
        } else {
            Some("vcredist-x64")
        }
    } else if name.contains(".net") && name.contains("framework") && name.contains("3.5") {
        Some("dotnet-framework-3-5")
    } else if name.contains(".net") && name.contains("framework") {
        Some("dotnet-framework")
    } else if name.contains("visual studio") && publisher_matches("microsoft") {
        if name.contains("community") {
            Some("visual-studio-community")
        } else if name.contains("professional") {
            Some("visual-studio-professional")
        } else if name.contains("enterprise") {
            Some("visual-studio-enterprise")
        } else {
            None
        }
    } else if name.contains("directx") && publisher_matches("microsoft") {
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

/// Reads the current driver version for one device after an install.
#[cfg(target_os = "windows")]
pub fn detect_driver_version(device_id: &str) -> Option<String> {
    if device_id.is_empty() {
        return None;
    }
    if let Some(version) = registry_driver_version(device_id) {
        return Some(version);
    }
    let escaped = device_id.replace('\'', "''");
    let script = format!(
        r#"$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$item = Get-CimInstance Win32_PnPSignedDriver -Filter "DeviceID='{escaped}'" | Select-Object -First 1
if ($item) {{ $item.DriverVersion }}"#
    );
    let value = run_powershell(&script, Duration::from_secs(30))?;
    let version = value.trim();
    valid_driver_version(version).then(|| version.to_string())
}

#[cfg(target_os = "windows")]
fn registry_driver_version(device_id: &str) -> Option<String> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let enum_key = hklm
        .open_subkey_with_flags(
            format!("SYSTEM\\CurrentControlSet\\Enum\\{device_id}"),
            KEY_READ,
        )
        .ok()?;
    if let Some(version) = version_from_device_key(&hklm, &enum_key) {
        return Some(version);
    }
    for instance in enum_key.enum_keys().flatten() {
        let Some(device) = enum_key.open_subkey_with_flags(&instance, KEY_READ).ok() else {
            continue;
        };
        if let Some(version) = version_from_device_key(&hklm, &device) {
            return Some(version);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn version_from_device_key(hklm: &winreg::RegKey, device: &winreg::RegKey) -> Option<String> {
    use winreg::enums::KEY_READ;

    let driver: String = device.get_value("Driver").ok()?;
    let class_key = hklm
        .open_subkey_with_flags(
            format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{driver}"),
            KEY_READ,
        )
        .ok()?;
    let version: String = class_key.get_value("DriverVersion").ok()?;
    let version = version.trim();
    valid_driver_version(version).then(|| version.to_string())
}

#[cfg(target_os = "windows")]
fn valid_driver_version(version: &str) -> bool {
    !version.is_empty()
        && version.chars().next().is_some_and(|c| c.is_ascii_digit())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-_+".contains(c))
}

#[cfg(not(target_os = "windows"))]
pub fn detect_driver_version(_device_id: &str) -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
pub fn detect_installed_components() -> Vec<InstalledComponent> {
    let script = r#"$ErrorActionPreference='SilentlyContinue'; [Console]::OutputEncoding=[Text.Encoding]::UTF8; $rows = @(Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' | Where-Object { $_.DisplayName } | Select-Object DisplayName,DisplayVersion,Publisher,InstallLocation,DisplayIcon); $netfx = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\NET Framework Setup\NDP\v3.5' -ErrorAction SilentlyContinue; if ($netfx -and $netfx.Install -eq 1) { $rows += [pscustomobject]@{ DisplayName='.NET Framework 3.5'; DisplayVersion='3.5 SP1'; InstallLocation=''; DisplayIcon='' } }; ConvertTo-Json -InputObject $rows -Compress"#;
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
        let publisher = row
            .get("Publisher")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let Some(id) = component_id_for(display_name, &version, publisher) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn normalizes_dotnet_dates() {
        assert_eq!(normalize_driver_date("/Date(1735689600000)/"), "2025-01-01");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalizes_slash_dates() {
        assert_eq!(normalize_driver_date("10/15/2024"), "2024-10-15");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalizes_compact_dates() {
        assert_eq!(normalize_driver_date("20240704"), "2024-07-04");
    }

    #[cfg(target_os = "windows")]
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

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_known_components() {
        assert_eq!(
            component_id_for(
                "Python 3.12.4 (64-bit)",
                "3.12.4",
                "Python Software Foundation"
            ),
            Some("python-3-12")
        );
        assert_eq!(
            component_id_for("7-Zip 24.09 (x64)", "24.09", "Igor Pavlov"),
            Some("seven-zip")
        );
        assert_eq!(
            component_id_for(
                "Microsoft Visual C++ 2015-2022 Redistributable (x64)",
                "14.44.35211",
                "Microsoft Corporation"
            ),
            Some("vcredist-x64")
        );
        assert_eq!(
            component_id_for(
                "Microsoft .NET Desktop Runtime 10.0.14",
                "10.0.14",
                "Microsoft Corporation"
            ),
            Some("dotnet-desktop-10")
        );
        assert_eq!(
            component_id_for(
                "Microsoft ASP.NET Core 10.0.14 - Shared Framework",
                "10.0.14",
                "Microsoft Corporation"
            ),
            Some("aspnetcore-10")
        );
        assert_eq!(
            component_id_for(
                "Python 3.14.10 (64-bit)",
                "3.14.10",
                "Python Software Foundation"
            ),
            Some("python-3-14")
        );
        assert_eq!(
            component_id_for("PHP 8.5.12", "8.5.12", "The PHP Group"),
            Some("php-8-5")
        );
        assert_eq!(
            component_id_for(
                "Microsoft Edge WebView2 Runtime",
                "156.0.4314.58",
                "Microsoft Corporation"
            ),
            Some("webview2")
        );
        assert_eq!(
            component_id_for("Vulkan SDK 1.4.363.0", "1.4.363.0", "Khronos Group"),
            Some("vulkan-sdk")
        );
        assert_eq!(
            component_id_for(
                "Vulkan Run Time Libraries 1.4.363.0",
                "1.4.363.0",
                "Khronos Group"
            ),
            Some("vulkan-runtime")
        );
        assert_eq!(
            component_id_for(
                "Microsoft Visual C++ 2022 Redistributable (Arm64)",
                "14.51.36247.0",
                "Microsoft Corporation"
            ),
            Some("vcredist-arm64")
        );
        assert_eq!(
            component_id_for(
                "Microsoft Visual Studio Code (User)",
                "1.143.1",
                "Microsoft Corporation"
            ),
            Some("vs-code")
        );
        assert_eq!(
            component_id_for(
                "Visual Studio Build Tools 2022",
                "17.14.44",
                "Microsoft Corporation"
            ),
            Some("vs-buildtools-2022")
        );
        assert_eq!(
            component_id_for("Docker Desktop", "4.98.0", "Docker Inc."),
            Some("docker")
        );
        assert_eq!(component_id_for("Unrelated Tool", "1.0.0", "Acme"), None);
        assert_eq!(component_id_for("Git LFS", "3.6.0", "Git LFS"), None);
        assert_eq!(
            component_id_for("Python Launcher", "3.12.4", "Python Software Foundation"),
            None
        );
    }

    #[cfg(target_os = "windows")]
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

#[cfg(all(test, target_os = "windows"))]
mod windows_native_tests {
    use super::*;

    #[test]
    #[ignore = "requires Windows devices"]
    fn native_inventory_returns_devices() {
        let native = detect_installed_drivers_native().expect("native inventory");
        assert!(!native.is_empty());
        let powershell = detect_installed_drivers_powershell().expect("powershell inventory");
        println!("native={} powershell={}", native.len(), powershell.len());
        println!(
            "native resource names={} resource vendors={}",
            native
                .iter()
                .filter(|driver| driver.name.starts_with('@'))
                .count(),
            native
                .iter()
                .filter(|driver| driver.vendor.starts_with('@'))
                .count()
        );
        for driver in native.iter().take(5) {
            println!("{} {} {}", driver.name, driver.current, driver.device_id);
        }
        let native_ids: std::collections::HashSet<&str> = native
            .iter()
            .map(|driver| driver.device_id.as_str())
            .collect();
        for driver in powershell
            .iter()
            .filter(|driver| !native_ids.contains(driver.device_id.as_str()))
        {
            println!(
                "powershell-only: {} {} {}",
                driver.name, driver.current, driver.device_id
            );
        }
    }
}
