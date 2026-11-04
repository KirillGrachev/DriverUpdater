//! Tauri command layer.
//!
//! <p>Commands are thin async wrappers around the repository; validation and
//! IPC input checks stay at this boundary.</p>

use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use tauri::State;

use crate::models::{
    CheckUpdatesResult, Driver, InitialData, LocalizedText, RestorePoint, Settings, Summary,
    SystemInfo,
};
use crate::repository::{detect_system_info, DriverRepository};

pub struct AppRepository(pub Mutex<DriverRepository>);

#[tauri::command]
pub async fn get_initial_data(state: State<'_, AppRepository>) -> Result<InitialData, String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.snapshot())
}

#[tauri::command]
pub async fn check_updates(state: State<'_, AppRepository>) -> Result<CheckUpdatesResult, String> {
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.check_updates())
}

#[tauri::command]
pub async fn install_driver(state: State<'_, AppRepository>, id: String) -> Result<Driver, String> {
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.install_driver(&id)
}

#[tauri::command]
pub async fn install_updates(
    state: State<'_, AppRepository>,
    ids: Vec<String>,
) -> Result<Vec<Driver>, String> {
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.install_updates(&ids)
}

#[tauri::command]
pub async fn list_restore_points(
    state: State<'_, AppRepository>,
) -> Result<Vec<RestorePoint>, String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.restore_points())
}

#[tauri::command]
pub async fn create_restore_point(
    state: State<'_, AppRepository>,
    name: LocalizedText,
) -> Result<RestorePoint, String> {
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.create_restore_point(name))
}

#[tauri::command]
pub async fn delete_restore_point(
    state: State<'_, AppRepository>,
    id: String,
) -> Result<(), String> {
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.delete_restore_point(&id)
}

#[tauri::command]
pub async fn restore_from_point(state: State<'_, AppRepository>, id: String) -> Result<(), String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.restore_from_point(&id)
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppRepository>,
    settings: Settings,
) -> Result<Settings, String> {
    validate_settings(&settings)?;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.save_settings(settings))
}

/// Rejects values the UI never produces, so a compromised or buggy webview
/// cannot persist arbitrary state or drive code paths with unexpected strings.
///
/// @param settings settings document received over IPC
/// @return validation error, or {@code Ok} when every field is accepted
fn validate_settings(settings: &Settings) -> Result<(), String> {
    const THEMES: [&str; 3] = ["light", "dark", "system"];
    const INTERVALS: [&str; 3] = ["hourly", "daily", "weekly"];
    const PERIODS: [&str; 3] = ["daily", "weekly", "monthly"];
    if !THEMES.contains(&settings.theme.as_str()) {
        return Err("invalid theme".to_string());
    }
    if !INTERVALS.contains(&settings.check_interval.as_str()) {
        return Err("invalid check interval".to_string());
    }
    if !PERIODS.contains(&settings.restore_point_period.as_str()) {
        return Err("invalid restore point period".to_string());
    }
    if settings.language.len() < 2
        || settings.language.len() > 8
        || !settings.language.chars().all(|c| c.is_ascii_alphabetic())
    {
        return Err("invalid language".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn open_backup_folder(state: State<'_, AppRepository>) -> Result<(), String> {
    let dir = {
        let repo = state.0.lock().map_err(|e| e.to_string())?;
        repo.backup_dir().to_path_buf()
    };
    open_folder(&dir)
}

fn open_folder(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("explorer");
        cmd.arg(path);
        cmd
    } else if cfg!(target_os = "macos") {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        cmd
    } else {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    };
    command.spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<bool, String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only http(s) urls are allowed".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        Ok(open_in_running_browser(&url))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
        Ok(false)
    }
}

/// Prefers an already running browser over launching the default one.
///
/// @param url HTTP(S) URL to open
/// @return {@code true} when the URL was handed to a running browser process
#[cfg(target_os = "windows")]
fn open_in_running_browser(url: &str) -> bool {
    const BROWSERS: [&str; 8] = [
        "chrome.exe",
        "msedge.exe",
        "brave.exe",
        "vivaldi.exe",
        "opera.exe",
        "yandex.exe",
        "browser.exe",
        "firefox.exe",
    ];

    let system = sysinfo::System::new_all();
    for browser in BROWSERS {
        let process = system
            .processes()
            .values()
            .find(|process| process.name().eq_ignore_ascii_case(browser));
        let Some(process) = process else {
            continue;
        };
        let Some(executable) = process.exe() else {
            continue;
        };
        if Command::new(executable).arg(url).spawn().is_ok() {
            return true;
        }
    }
    false
}

#[tauri::command]
pub async fn detect_installed_components() -> Result<Vec<crate::models::InstalledComponent>, String>
{
    Ok(crate::repository::detect_installed_components())
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(detect_system_info())
}

#[tauri::command]
pub async fn compute_summary(
    state: State<'_, AppRepository>,
    drivers: Vec<Driver>,
) -> Result<Summary, String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    let base = repo.summary();
    let updates_available = drivers.iter().filter(|d| d.status == "update").count() as u32;
    Ok(Summary {
        devices_total: base.devices_total,
        up_to_date: base.devices_total.saturating_sub(updates_available),
        updates_available,
        last_check: base.last_check,
    })
}
