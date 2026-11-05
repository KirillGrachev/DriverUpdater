//! Tauri command layer.
//!
//! <p>Commands are thin async wrappers around the repository; validation and
//! IPC input checks stay at this boundary.</p>

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, State};

use crate::backup::default_backup_provider;
use crate::detection::{detect_system_info, DriverDetector, NativeDetector};
use crate::models::{
    CheckUpdatesResult, Driver, InitialData, InstallFailure, InstallReport, RestorePoint, Settings,
    SystemInfo,
};
use crate::repository::DriverRepository;
use crate::restore::default_restore_provider;
use crate::windows_update::default_update_provider;

pub struct AppRepository(pub Arc<Mutex<DriverRepository>>);

/// Package installs that the UI can cancel by package id.
#[derive(Default)]
pub struct PackageJobs(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallProgress {
    id: String,
    current: usize,
    total: usize,
    stage: String,
}

fn emit_install_progress(app: &AppHandle, id: &str, current: usize, total: usize, stage: &str) {
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            id: id.to_string(),
            current,
            total,
            stage: stage.to_string(),
        },
    );
}

/// Tracks long-running operations that must not be interrupted by closing the window.
#[derive(Default)]
pub struct BusyState(pub Arc<AtomicUsize>);

struct BusyGuard(Arc<AtomicUsize>);

impl BusyGuard {
    fn acquire(state: &Arc<AtomicUsize>) -> Self {
        state.fetch_add(1, Ordering::SeqCst);
        Self(Arc::clone(state))
    }
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/**
 * Runs a repository operation on the blocking pool.
 *
 * <p>PowerShell/CIM scans and disk writes are blocking calls; running them on
 * Tauri's async executor would stall unrelated IPC commands. The repository
 * handle is cloned into the closure so the command future stays `Send`.</p>
 *
 * @param state managed application state
 * @param operation repository work to execute under the mutex
 * @returns operation result or a serialized error
 */
async fn run_repository_handle<T, F>(
    repository: Arc<Mutex<DriverRepository>>,
    operation: F,
) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut DriverRepository) -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let mut repository = repository.lock().map_err(|error| error.to_string())?;
        operation(&mut repository)
    })
    .await
    .map_err(|error| error.to_string())?
}

async fn run_repository<T, F>(state: State<'_, AppRepository>, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut DriverRepository) -> Result<T, String> + Send + 'static,
{
    run_repository_handle(Arc::clone(&state.0), operation).await
}

async fn detect_drivers() -> Option<Vec<Driver>> {
    tauri::async_runtime::spawn_blocking(|| NativeDetector.detect())
        .await
        .ok()
        .flatten()
}

#[tauri::command]
pub async fn get_initial_data(state: State<'_, AppRepository>) -> Result<InitialData, String> {
    let repository = Arc::clone(&state.0);
    let needs_scan = run_repository_handle(Arc::clone(&repository), |repository| {
        Ok(repository.needs_scan())
    })
    .await?;
    if needs_scan {
        let drivers = detect_drivers().await;
        let _ = run_repository_handle(Arc::clone(&repository), move |repository| {
            repository.apply_initial_scan(drivers)
        })
        .await;
    }
    run_repository_handle(repository, |repository| Ok(repository.snapshot())).await
}

#[tauri::command]
pub async fn scan_inventory(state: State<'_, AppRepository>) -> Result<InitialData, String> {
    let repository = Arc::clone(&state.0);
    let drivers = detect_drivers().await;
    run_repository_handle(repository, move |repository| {
        repository.apply_initial_scan(drivers)?;
        Ok(repository.snapshot())
    })
    .await
}

#[tauri::command]
pub async fn check_updates(state: State<'_, AppRepository>) -> Result<CheckUpdatesResult, String> {
    let repository = Arc::clone(&state.0);
    let drivers_task = tauri::async_runtime::spawn_blocking(|| NativeDetector.detect());
    let updates_task =
        tauri::async_runtime::spawn_blocking(|| default_update_provider().find_updates());
    let drivers = drivers_task.await.ok().flatten();
    let updates = updates_task.await.map_err(|error| error.to_string())?;
    run_repository_handle(repository, move |repository| {
        repository.apply_check_result(drivers, updates)
    })
    .await
}

struct InstallRequest {
    id: String,
    create_backup: bool,
    restore_point_name: Option<String>,
    create_restore: bool,
    allow_missing_restore: bool,
    current: usize,
    total: usize,
}

async fn install_one(
    repository: Arc<Mutex<DriverRepository>>,
    busy: Arc<AtomicUsize>,
    app: AppHandle,
    request: InstallRequest,
) -> Result<Driver, String> {
    let InstallRequest {
        id,
        create_backup,
        restore_point_name,
        create_restore,
        allow_missing_restore,
        current,
        total,
    } = request;
    let _busy = BusyGuard::acquire(&busy);
    log::info!("preparing driver install for {id}");
    let plan = run_repository_handle(Arc::clone(&repository), {
        let id = id.clone();
        move |repository| repository.prepare_install(&id)
    })
    .await?;

    if create_restore && plan.restore_before_install {
        emit_install_progress(&app, &id, current, total, "restore");
        let name = restore_point_name
            .filter(|name| !name.trim().is_empty() && name.chars().count() <= 200)
            .ok_or_else(|| "E_RESTORE_POINT: a restore point name is required".to_string())?;
        let created =
            tauri::async_runtime::spawn_blocking(move || default_restore_provider().create(&name))
                .await
                .map_err(|error| error.to_string())?;
        match created {
            Ok(_) => {
                let points =
                    tauri::async_runtime::spawn_blocking(|| default_restore_provider().list())
                        .await
                        .map_err(|error| error.to_string())?
                        .map_err(|error| format!("E_RESTORE_POINT: {error}"))?;
                run_repository_handle(Arc::clone(&repository), move |repository| {
                    repository.set_restore_points(points);
                    Ok(())
                })
                .await?;
            }
            Err(error) if allow_missing_restore => {
                log::warn!("restore point was skipped after an error: {error}");
            }
            Err(error) => return Err(format!("E_RESTORE_POINT: {error}")),
        }
    }

    if create_backup && plan.keep_backups {
        emit_install_progress(&app, &id, current, total, "backup");
        let backup_dir = plan.backup_dir.clone();
        tauri::async_runtime::spawn_blocking(move || default_backup_provider(backup_dir).create())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| format!("driver backup was not created: {error}"))?;
    }

    emit_install_progress(&app, &id, current, total, "install");
    let update_id = plan.update_id.clone();
    log::info!("installing Windows Update package {update_id}");
    let logged_update_id = update_id.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        default_update_provider().install_update(&update_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    log::info!(
        "Windows Update package {logged_update_id} finished, reboot={}",
        outcome.reboot_required
    );
    run_repository_handle(repository, move |repository| {
        repository.finish_install(&id, outcome)
    })
    .await
}

#[tauri::command]
pub async fn install_driver(
    state: State<'_, AppRepository>,
    id: String,
    restore_point_name: String,
    allow_missing_restore: bool,
    busy: State<'_, BusyState>,
    app: AppHandle,
) -> Result<Driver, String> {
    install_one(
        Arc::clone(&state.0),
        Arc::clone(&busy.0),
        app,
        InstallRequest {
            id,
            create_backup: true,
            restore_point_name: Some(restore_point_name),
            create_restore: true,
            allow_missing_restore,
            current: 1,
            total: 1,
        },
    )
    .await
}

#[tauri::command]
pub async fn install_updates(
    state: State<'_, AppRepository>,
    ids: Vec<String>,
    restore_point_name: String,
    allow_missing_restore: bool,
    busy: State<'_, BusyState>,
    app: AppHandle,
) -> Result<InstallReport, String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let repository = Arc::clone(&state.0);
    let (restore_before_install, keep_backups, backup_dir) =
        run_repository_handle(Arc::clone(&repository), |repository| {
            let (keep_backups, backup_dir) = repository.backup_policy();
            Ok((repository.restore_policy(), keep_backups, backup_dir))
        })
        .await?;
    if restore_before_install {
        emit_install_progress(&app, "", 0, 0, "restore");
        if restore_point_name.trim().is_empty() || restore_point_name.chars().count() > 200 {
            return Err("E_RESTORE_POINT: a restore point name is required".to_string());
        }
        let name = restore_point_name;
        let created =
            tauri::async_runtime::spawn_blocking(move || default_restore_provider().create(&name))
                .await
                .map_err(|error| error.to_string())?;
        match created {
            Ok(_) => {}
            Err(error) if allow_missing_restore => {
                log::warn!("restore point was skipped after an error: {error}");
            }
            Err(error) => return Err(format!("E_RESTORE_POINT: {error}")),
        }
    }
    if keep_backups {
        emit_install_progress(&app, "", 0, 0, "backup");
        tauri::async_runtime::spawn_blocking(move || default_backup_provider(backup_dir).create())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| format!("driver backup was not created: {error}"))?;
    }
    let mut report = InstallReport::default();
    let total = ids.len();
    for (index, id) in ids.into_iter().enumerate() {
        match install_one(
            Arc::clone(&repository),
            Arc::clone(&busy.0),
            app.clone(),
            InstallRequest {
                id: id.clone(),
                create_backup: false,
                restore_point_name: None,
                create_restore: false,
                allow_missing_restore: true,
                current: index + 1,
                total,
            },
        )
        .await
        {
            Ok(driver) => report.succeeded.push(driver),
            Err(message) => report.failed.push(InstallFailure { id, message }),
        }
    }
    Ok(report)
}

#[tauri::command]
pub async fn list_restore_points(
    state: State<'_, AppRepository>,
) -> Result<Vec<RestorePoint>, String> {
    let points = tauri::async_runtime::spawn_blocking(|| default_restore_provider().list())
        .await
        .map_err(|error| error.to_string())??;
    run_repository_handle(Arc::clone(&state.0), move |repository| {
        let result = points.clone();
        repository.set_restore_points(points);
        Ok(result)
    })
    .await
}

#[tauri::command]
pub async fn create_restore_point(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
    name: String,
) -> Result<RestorePoint, String> {
    let _busy = BusyGuard::acquire(&busy.0);
    if name.trim().is_empty() || name.chars().count() > 200 {
        return Err("invalid restore point name".to_string());
    }
    let point =
        tauri::async_runtime::spawn_blocking(move || default_restore_provider().create(&name))
            .await
            .map_err(|error| error.to_string())??;
    let points = tauri::async_runtime::spawn_blocking(|| default_restore_provider().list())
        .await
        .map_err(|error| error.to_string())??;
    run_repository_handle(Arc::clone(&state.0), move |repository| {
        repository.set_restore_points(points);
        Ok(point)
    })
    .await
}

#[tauri::command]
pub async fn delete_restore_point(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
    id: String,
) -> Result<(), String> {
    let _busy = BusyGuard::acquire(&busy.0);
    tauri::async_runtime::spawn_blocking(move || default_restore_provider().delete(&id))
        .await
        .map_err(|error| error.to_string())??;
    let points = tauri::async_runtime::spawn_blocking(|| default_restore_provider().list())
        .await
        .map_err(|error| error.to_string())??;
    run_repository_handle(Arc::clone(&state.0), move |repository| {
        repository.set_restore_points(points);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn restore_from_point(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
    id: String,
) -> Result<(), String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let _ = state;
    tauri::async_runtime::spawn_blocking(move || default_restore_provider().restore(&id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppRepository>,
    settings: Settings,
) -> Result<Settings, String> {
    validate_settings(&settings)?;
    run_repository(state, move |repository| repository.save_settings(settings)).await
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

async fn backup_root(state: &State<'_, AppRepository>) -> Result<PathBuf, String> {
    run_repository_handle(Arc::clone(&state.0), |repository| {
        Ok(repository.backup_dir().to_path_buf())
    })
    .await
}

#[tauri::command]
pub async fn open_backup_folder(state: State<'_, AppRepository>) -> Result<(), String> {
    let folder = backup_root(&state).await?;
    tauri::async_runtime::spawn_blocking(move || open_folder(&folder))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(target_os = "windows")]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new(crate::process::windows_executable("explorer.exe"))
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn detect_installed_components(
    state: State<'_, AppRepository>,
) -> Result<Vec<crate::models::InstalledComponent>, String> {
    run_repository(state, |repository| Ok(repository.detect_components())).await
}

#[tauri::command]
pub async fn get_installed_components(
    state: State<'_, AppRepository>,
) -> Result<Vec<String>, String> {
    run_repository(state, |repository| Ok(repository.installed_components())).await
}

#[tauri::command]
pub async fn save_installed_components(
    state: State<'_, AppRepository>,
    ids: Vec<String>,
) -> Result<Vec<String>, String> {
    run_repository(state, move |repository| {
        repository.save_installed_components(ids)
    })
    .await
}

#[tauri::command]
pub async fn list_driver_backups(
    state: State<'_, AppRepository>,
) -> Result<Vec<crate::backup::DriverBackup>, String> {
    let root = backup_root(&state).await?;
    tauri::async_runtime::spawn_blocking(move || default_backup_provider(root).list())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn create_driver_backup(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
) -> Result<crate::backup::DriverBackup, String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let root = backup_root(&state).await?;
    tauri::async_runtime::spawn_blocking(move || default_backup_provider(root).create())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn restore_driver_backup(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
    id: String,
) -> Result<(), String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let root = backup_root(&state).await?;
    tauri::async_runtime::spawn_blocking(move || default_backup_provider(root).restore(&id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn delete_driver_backup(
    state: State<'_, AppRepository>,
    busy: State<'_, BusyState>,
    id: String,
) -> Result<(), String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let root = backup_root(&state).await?;
    tauri::async_runtime::spawn_blocking(move || default_backup_provider(root).delete(&id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    tauri::async_runtime::spawn_blocking(detect_system_info)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_package_versions(package_id: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::packages::WingetPackageManager.versions(&package_id)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn install_package(
    state: State<'_, PackageJobs>,
    busy: State<'_, BusyState>,
    package_id: String,
    version: String,
) -> Result<(), String> {
    let _busy = BusyGuard::acquire(&busy.0);
    let cancel = Arc::new(AtomicBool::new(false));
    let job_key = package_id.clone();
    {
        let mut jobs = state
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if jobs.contains_key(&job_key) {
            return Err("package is already installing".to_string());
        }
        jobs.insert(job_key.clone(), Arc::clone(&cancel));
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::packages::WingetPackageManager.install_cancellable(&package_id, &version, &cancel)
    })
    .await
    .map_err(|error| error.to_string());
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&job_key);
    result?
}

#[tauri::command]
pub async fn cancel_package_install(
    state: State<'_, PackageJobs>,
    package_id: String,
) -> Result<bool, String> {
    let job = state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&package_id)
        .cloned();
    if let Some(flag) = job {
        flag.store(true, Ordering::SeqCst);
        Ok(true)
    } else {
        Ok(false)
    }
}
