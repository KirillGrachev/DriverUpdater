//! Windows Update driver discovery and installation.
//!
//! <p>Windows Update is the only supported update source for now. It is an
//! honest source: every candidate comes from Microsoft's update service and is
//! installed through the Windows Update Agent. A future vendor database can
//! implement the same [`DriverUpdateProvider`] trait.</p>

#[cfg(target_os = "windows")]
use std::time::Duration;

use serde::Deserialize;

use crate::models::Driver;
#[cfg(target_os = "windows")]
use crate::process;

/// A driver update returned by Windows Update.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverUpdate {
    pub update_id: String,
    pub title: String,
    #[serde(default)]
    pub hardware_ids: Vec<String>,
    #[serde(default)]
    pub driver_date: String,
    #[serde(default)]
    pub max_download_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOutcome {
    #[cfg(target_os = "windows")]
    pub result_code: u32,
    #[serde(default)]
    pub reboot_required: bool,
    #[cfg(target_os = "windows")]
    #[serde(default)]
    pub h_result: i64,
}

/// Finds and installs driver updates.
pub trait DriverUpdateProvider: Send + Sync {
    fn find_updates(&self) -> Result<Vec<DriverUpdate>, String>;
    fn install_update(&self, update_id: &str) -> Result<InstallOutcome, String>;
}

/// Windows Update Agent implementation.
#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct WindowsUpdateProvider;

#[cfg(target_os = "windows")]
impl DriverUpdateProvider for WindowsUpdateProvider {
    fn find_updates(&self) -> Result<Vec<DriverUpdate>, String> {
        match find_updates_native() {
            Ok(updates) => Ok(updates),
            Err(error) => {
                log::warn!(
                    "native Windows Update search failed, falling back to PowerShell: {error}"
                );
                find_updates_powershell()
            }
        }
    }

    fn install_update(&self, update_id: &str) -> Result<InstallOutcome, String> {
        match install_update_native(update_id) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                log::warn!(
                    "native Windows Update install failed, falling back to PowerShell: {error}"
                );
                install_update_powershell(update_id)
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn find_updates_powershell() -> Result<Vec<DriverUpdate>, String> {
    let script = r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$session = New-Object -ComObject Microsoft.Update.Session
$searcher = $session.CreateUpdateSearcher()
$result = $searcher.Search("IsInstalled=0 AND Type='Driver'")
$items = @()
foreach ($update in $result.Updates) {
    $items += [pscustomobject]@{
        updateId = $update.Identity.UpdateID
        title = $update.Title
        hardwareIds = @($update.DriverHardwareID)
        driverModel = $update.DriverModel
        driverManufacturer = $update.DriverManufacturer
        driverDate = if ($update.DriverVerDate) { $update.DriverVerDate.ToString('o') } else { '' }
        maxDownloadSize = [uint64]$update.MaxDownloadSize
    }
}
$payload = [pscustomobject]@{ updates = @($items) }
ConvertTo-Json -InputObject $payload -Compress -Depth 4
"#;
    let output = process::run_powershell(script, Duration::from_secs(180))?;
    if !output.success {
        return Err(command_error(&output));
    }
    let parsed: UpdateEnvelope =
        serde_json::from_str(&output.stdout).map_err(|error| error.to_string())?;
    Ok(parsed.updates)
}

#[cfg(target_os = "windows")]
fn find_updates_native() -> Result<Vec<DriverUpdate>, String> {
    use windows::core::{IUnknown, Interface, BSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::System::UpdateAgent::{
        IUpdateSession, IWindowsDriverUpdate, UpdateSession,
    };

    struct ComGuard(bool);

    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe {
                    CoUninitialize();
                }
            }
        }
    }

    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .is_ok();
    let _guard = ComGuard(initialized);
    let result = (|| -> windows::core::Result<Vec<DriverUpdate>> {
        let session: IUpdateSession =
            unsafe { CoCreateInstance(&UpdateSession, None::<&IUnknown>, CLSCTX_INPROC_SERVER) }?;
        let searcher = unsafe { session.CreateUpdateSearcher() }?;
        let found = unsafe { searcher.Search(&BSTR::from("IsInstalled=0 AND Type='Driver'")) }?;
        let updates = unsafe { found.Updates()? };
        let count = unsafe { updates.Count()? };
        let mut drivers = Vec::new();
        for index in 0..count {
            let update = unsafe { updates.get_Item(index)? };
            let driver_update: IWindowsDriverUpdate = update.cast()?;
            let update_id = unsafe { update.Identity()?.UpdateID()?.to_string() };
            let title = unsafe { update.Title()?.to_string() };
            let hardware = unsafe { driver_update.DriverHardwareID()?.to_string() };
            let hardware_ids = hardware
                .split([';', ','])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect();
            let date = unsafe { driver_update.DriverVerDate() }.unwrap_or(0.0);
            let size = unsafe { driver_update.MaxDownloadSize() }.unwrap_or_default();
            drivers.push(DriverUpdate {
                update_id,
                title,
                hardware_ids,
                driver_date: ole_date_to_iso(date),
                max_download_size: decimal_to_u64(size),
            });
        }
        Ok(drivers)
    })();
    result.map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn ole_date_to_iso(value: f64) -> String {
    if !(value.is_finite() && value > 0.0) {
        return String::new();
    }
    let seconds = ((value - 25_569.0) * 86_400.0).max(0.0) as u64;
    let days = seconds / 86_400;
    let time = seconds % 86_400;
    let (year, month, day) = crate::detection::days_to_civil(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    )
}

#[cfg(target_os = "windows")]
fn decimal_to_u64(value: windows::Win32::Foundation::DECIMAL) -> u64 {
    let (scale, sign) = unsafe {
        (
            value.Anonymous1.Anonymous.scale,
            value.Anonymous1.Anonymous.sign,
        )
    };
    if sign != 0 {
        return 0;
    }
    let raw = unsafe { value.Anonymous2.Lo64 };
    let divisor = 10u64.saturating_pow(scale as u32);
    raw.checked_div(divisor).unwrap_or(raw)
}

#[cfg(target_os = "windows")]
fn install_update_native(update_id: &str) -> Result<InstallOutcome, String> {
    use windows::core::{IUnknown, BSTR};
    use windows::Win32::Foundation::E_FAIL;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::System::UpdateAgent::{
        orcSucceeded, orcSucceededWithErrors, IUpdate, IUpdateCollection, IUpdateSession,
        UpdateCollection, UpdateSession,
    };

    if !valid_update_id(update_id) {
        return Err("invalid Windows Update identity".to_string());
    }
    struct ComGuard(bool);
    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe {
                    CoUninitialize();
                }
            }
        }
    }
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .is_ok();
    let _guard = ComGuard(initialized);
    let result = (|| -> windows::core::Result<InstallOutcome> {
        let session: IUpdateSession =
            unsafe { CoCreateInstance(&UpdateSession, None::<&IUnknown>, CLSCTX_INPROC_SERVER) }?;
        let searcher = unsafe { session.CreateUpdateSearcher() }?;
        let found = unsafe { searcher.Search(&BSTR::from("IsInstalled=0 AND Type='Driver'")) }?;
        let updates = unsafe { found.Updates()? };
        let count = unsafe { updates.Count()? };
        let mut target: Option<IUpdate> = None;
        for index in 0..count {
            let update = unsafe { updates.get_Item(index)? };
            let id = unsafe { update.Identity()?.UpdateID()?.to_string() };
            if id.eq_ignore_ascii_case(update_id) {
                target = Some(update);
                break;
            }
        }
        let target = target.ok_or_else(|| {
            windows::core::Error::new(E_FAIL, "this driver update is no longer available")
        })?;
        let download: IUpdateCollection = unsafe {
            CoCreateInstance(&UpdateCollection, None::<&IUnknown>, CLSCTX_INPROC_SERVER)
        }?;
        unsafe { download.Add(&target)? };
        let downloader = unsafe { session.CreateUpdateDownloader() }?;
        unsafe { downloader.SetUpdates(&download)? };
        let download_result = unsafe { downloader.Download() }?;
        let download_code = unsafe { download_result.ResultCode() }?.0;
        if download_code != orcSucceeded.0 && download_code != orcSucceededWithErrors.0 {
            return Err(windows::core::Error::new(
                E_FAIL,
                format!("download failed with code {download_code}"),
            ));
        }
        let install: IUpdateCollection = unsafe {
            CoCreateInstance(&UpdateCollection, None::<&IUnknown>, CLSCTX_INPROC_SERVER)
        }?;
        unsafe { install.Add(&target)? };
        let installer = unsafe { session.CreateUpdateInstaller() }?;
        unsafe { installer.SetUpdates(&install)? };
        let installed = unsafe { installer.Install() }?;
        let result_code = unsafe { installed.ResultCode() }?.0 as u32;
        let reboot_required = unsafe { installed.RebootRequired() }?.as_bool();
        let h_result = unsafe { installed.HResult() }? as i64;
        if result_code != 2 && result_code != 3 {
            return Err(windows::core::Error::new(
                E_FAIL,
                format!("Windows Update returned result code {result_code}"),
            ));
        }
        Ok(InstallOutcome {
            result_code,
            reboot_required,
            h_result,
        })
    })();
    result.map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn install_update_powershell(update_id: &str) -> Result<InstallOutcome, String> {
    if !valid_update_id(update_id) {
        return Err("invalid Windows Update identity".to_string());
    }
    let script = format!(
        r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$session = New-Object -ComObject Microsoft.Update.Session
$searcher = $session.CreateUpdateSearcher()
$result = $searcher.Search("IsInstalled=0 AND Type='Driver'")
$target = $null
foreach ($update in $result.Updates) {{
    if ($update.Identity.UpdateID -eq '{update_id}') {{ $target = $update; break }}
}}
if (-not $target) {{ throw 'this driver update is no longer available' }}
$downloadColl = New-Object -ComObject Microsoft.Update.UpdateColl
[void]$downloadColl.Add($target)
$downloader = $session.CreateUpdateDownloader()
$downloader.Updates = $downloadColl
$download = $downloader.Download()
if ($download.ResultCode -notin 2,3) {{ throw "download failed with code $($download.ResultCode)" }}
$installColl = New-Object -ComObject Microsoft.Update.UpdateColl
[void]$installColl.Add($target)
$installer = $session.CreateUpdateInstaller()
$installer.Updates = $installColl
$install = $installer.Install()
[pscustomobject]@{{
    resultCode = [uint32]$install.ResultCode
    rebootRequired = [bool]$install.RebootRequired
    hResult = [int64]$install.HResult
}} | ConvertTo-Json -Compress
"#
    );
    let output = process::run_powershell(&script, Duration::from_secs(30 * 60))?;
    if !output.success {
        return Err(command_error(&output));
    }
    let outcome: InstallOutcome =
        serde_json::from_str(&output.stdout).map_err(|error| error.to_string())?;
    if outcome.result_code == 2 || outcome.result_code == 3 {
        Ok(outcome)
    } else {
        Err(format!(
            "Windows Update returned result code {} (HRESULT 0x{:08X})",
            outcome.result_code, outcome.h_result
        ))
    }
}

/// Non-Windows placeholder so the application can still build and run in a
/// browser preview without compiling Windows Update code.
#[cfg(not(target_os = "windows"))]
#[derive(Default)]
pub struct UnsupportedUpdateProvider;

#[cfg(not(target_os = "windows"))]
impl DriverUpdateProvider for UnsupportedUpdateProvider {
    fn find_updates(&self) -> Result<Vec<DriverUpdate>, String> {
        Err("Windows Update is available only on Windows".to_string())
    }

    fn install_update(&self, _update_id: &str) -> Result<InstallOutcome, String> {
        Err("Windows Update is available only on Windows".to_string())
    }
}

/// Returns the provider for the current target.
pub fn default_update_provider() -> Box<dyn DriverUpdateProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::<WindowsUpdateProvider>::default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::<UnsupportedUpdateProvider>::default()
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
struct UpdateEnvelope {
    #[serde(default)]
    updates: Vec<DriverUpdate>,
}

#[cfg(target_os = "windows")]
pub fn valid_update_id(value: &str) -> bool {
    let length = value.len();
    length == 36
        && value.chars().enumerate().all(|(index, character)| {
            if index == 8 || index == 13 || index == 18 || index == 23 {
                character == '-'
            } else {
                character.is_ascii_hexdigit()
            }
        })
}

/// Applies matching Windows Update candidates to the detected driver list.
pub fn merge_updates(drivers: &mut [Driver], updates: &[DriverUpdate]) -> usize {
    let mut applied = 0;
    for update in updates {
        let version = version_from_title(&update.title);
        let hardware_ids: Vec<String> = update
            .hardware_ids
            .iter()
            .flat_map(|value| {
                value.split(|character: char| character.is_whitespace() || character == ',')
            })
            .map(normalize_hardware_id)
            .filter(|value| !value.is_empty())
            .collect();
        for driver in drivers.iter_mut() {
            let driver_hardware: Vec<String> = driver
                .hardware_ids
                .iter()
                .map(|value| normalize_hardware_id(value))
                .collect();
            let hardware_match = hardware_ids.iter().any(|candidate| {
                driver_hardware
                    .iter()
                    .any(|installed| installed == candidate)
            });
            if !hardware_match || version.is_empty() {
                continue;
            }
            if !is_newer_version(&version, &driver.current) {
                continue;
            }
            driver.available = Some(version.clone());
            driver.status = "update".to_string();
            driver.recommended = true;
            driver.update_id = Some(update.update_id.clone());
            driver.size_mb = (update.max_download_size / (1024 * 1024)) as u32;
            driver.reboot_required = false;
            if !update.driver_date.is_empty() && driver.released.is_empty() {
                driver.released = update.driver_date.chars().take(10).collect();
            }
            applied += 1;
        }
    }
    applied
}

fn normalize_hardware_id(value: &str) -> String {
    value.trim().trim_matches('"').to_ascii_uppercase()
}

fn is_newer_version(candidate: &str, installed: &str) -> bool {
    let numbers = |value: &str| -> Vec<u64> {
        value
            .split(|character: char| !character.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<u64>().ok())
            .collect()
    };
    let candidate = numbers(candidate);
    let installed = numbers(installed);
    if candidate.is_empty() {
        return false;
    }
    for (left, right) in candidate.iter().zip(installed.iter()) {
        if left != right {
            return left > right;
        }
    }
    candidate.len() > installed.len()
}

fn version_from_title(title: &str) -> String {
    let Some(close) = title.rfind(')') else {
        return String::new();
    };
    let Some(open) = title[..close].rfind('(') else {
        return String::new();
    };
    let candidate = title[open + 1..close]
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V');
    if !candidate.is_empty()
        && candidate.chars().next().is_some_and(|c| c.is_ascii_digit())
        && candidate
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_+".contains(character))
    {
        candidate.to_string()
    } else {
        String::new()
    }
}

#[cfg(target_os = "windows")]
fn command_error(output: &process::ProcessOutput) -> String {
    let details = if !output.stderr.trim().is_empty() {
        output.stderr.trim()
    } else {
        output.stdout.trim()
    };
    details
        .lines()
        .last()
        .unwrap_or("Windows Update failed")
        .chars()
        .take(300)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn validates_update_identity() {
        assert!(valid_update_id("12345678-1234-1234-1234-123456789abc"));
        assert!(!valid_update_id("12345678-1234-1234-1234-123456789ab"));
        assert!(!valid_update_id("12345678; Remove-Item C:\\"));
    }

    #[test]
    fn extracts_version_from_update_title() {
        assert_eq!(version_from_title("Vendor Driver Update (1.2.3)"), "1.2.3");
        assert_eq!(version_from_title("No version"), "");
    }

    #[test]
    fn normalizes_hardware_ids() {
        assert_eq!(normalize_hardware_id("  swc\\ven_msft  "), "SWC\\VEN_MSFT");
    }

    #[test]
    fn merges_by_hardware_id() {
        let mut drivers = vec![Driver {
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
        }];
        let updates = vec![DriverUpdate {
            update_id: "12345678-1234-1234-1234-123456789abc".to_string(),
            title: "Voice Clarity (2.0.0)".to_string(),
            hardware_ids: vec!["swc\\ven_msft&audio".to_string()],
            driver_date: "2026-10-16T00:00:00Z".to_string(),
            max_download_size: 10 * 1024 * 1024,
        }];
        assert_eq!(merge_updates(&mut drivers, &updates), 1);
        assert_eq!(drivers[0].available.as_deref(), Some("2.0.0"));
        assert_eq!(drivers[0].status, "update");
    }

    #[test]
    fn ignores_update_without_matching_hardware_id() {
        let mut drivers = vec![Driver {
            id: "pnp-1".to_string(),
            name: "Realtek Audio".to_string(),
            vendor: "Realtek".to_string(),
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
            device: "Realtek Audio".to_string(),
            released: String::new(),
            device_id: "HDAUDIO\\DEVICE".to_string(),
            hardware_ids: vec!["HDAUDIO\\VEN_10EC&DEV_0897".to_string()],
            inf_name: String::new(),
            update_id: None,
            reboot_required: false,
        }];
        let updates = vec![DriverUpdate {
            update_id: "12345678-1234-1234-1234-123456789abc".to_string(),
            title: "Wrong vendor driver (2.0.0)".to_string(),
            hardware_ids: vec!["PCI\\VEN_10DE&DEV_2F04".to_string()],
            driver_date: String::new(),
            max_download_size: 0,
        }];
        assert_eq!(merge_updates(&mut drivers, &updates), 0);
        assert_eq!(drivers[0].status, "current");
    }

    #[test]
    fn ignores_update_without_a_numeric_version() {
        let mut drivers = vec![Driver {
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
        }];
        let updates = vec![DriverUpdate {
            update_id: "12345678-1234-1234-1234-123456789abc".to_string(),
            title: "Voice Clarity (beta)".to_string(),
            hardware_ids: vec!["SWC\\VEN_MSFT&AUDIO".to_string()],
            driver_date: String::new(),
            max_download_size: 0,
        }];
        assert_eq!(merge_updates(&mut drivers, &updates), 0);
    }

    #[test]
    fn ignores_older_windows_update_candidates() {
        assert!(is_newer_version("2.0.0", "1.0.0"));
        assert!(is_newer_version("1.0.4.55794", "1.0.4.7057"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }
}

#[cfg(all(test, target_os = "windows"))]
mod native_wu_tests {
    use super::*;

    #[test]
    #[ignore = "queries Windows Update"]
    fn native_update_search_returns_a_result() {
        match find_updates_native() {
            Ok(updates) => {
                println!("native Windows Update candidates={}", updates.len());
                for update in &updates {
                    println!(
                        "{} | {} | {:?} | {} | {}",
                        update.update_id,
                        update.title,
                        update.hardware_ids,
                        update.driver_date,
                        update.max_download_size
                    );
                }
            }
            Err(error) => println!("native Windows Update error={error}"),
        }
    }
}
