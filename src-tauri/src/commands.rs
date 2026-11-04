//! Tauri command layer. Every command is async and simulates a small amount
//! of work so the UI loading states behave like they will with real hardware.

use std::sync::Mutex;
use std::time::Duration;

use tauri::State;

use crate::mock::{detect_system_info, MockRepository};
use crate::models::{
    CheckUpdatesResult, Driver, InitialData, LocalizedText, RestorePoint, Settings, Summary,
    SystemInfo,
};

pub struct AppRepository(pub Mutex<MockRepository>);

const LATENCY: Duration = Duration::from_millis(180);
const SCAN_LATENCY: Duration = Duration::from_millis(900);
const INSTALL_LATENCY: Duration = Duration::from_millis(1300);

#[tauri::command]
pub async fn get_initial_data(state: State<'_, AppRepository>) -> Result<InitialData, String> {
    tokio::time::sleep(LATENCY).await;
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.snapshot())
}

#[tauri::command]
pub async fn check_updates(state: State<'_, AppRepository>) -> Result<CheckUpdatesResult, String> {
    tokio::time::sleep(SCAN_LATENCY).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.check_updates())
}

#[tauri::command]
pub async fn install_driver(
    state: State<'_, AppRepository>,
    id: String,
) -> Result<Driver, String> {
    tokio::time::sleep(INSTALL_LATENCY).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.install_driver(&id)
}

#[tauri::command]
pub async fn install_updates(
    state: State<'_, AppRepository>,
    ids: Vec<String>,
) -> Result<Vec<Driver>, String> {
    tokio::time::sleep(INSTALL_LATENCY).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.install_updates(&ids)
}

#[tauri::command]
pub async fn list_restore_points(
    state: State<'_, AppRepository>,
) -> Result<Vec<RestorePoint>, String> {
    tokio::time::sleep(LATENCY).await;
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.restore_points())
}

#[tauri::command]
pub async fn create_restore_point(
    state: State<'_, AppRepository>,
    name: LocalizedText,
) -> Result<RestorePoint, String> {
    tokio::time::sleep(Duration::from_millis(700)).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.create_restore_point(name))
}

#[tauri::command]
pub async fn delete_restore_point(
    state: State<'_, AppRepository>,
    id: String,
) -> Result<(), String> {
    tokio::time::sleep(LATENCY).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.delete_restore_point(&id)
}

#[tauri::command]
pub async fn restore_from_point(
    state: State<'_, AppRepository>,
    id: String,
) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.restore_from_point(&id)
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppRepository>,
    settings: Settings,
) -> Result<Settings, String> {
    tokio::time::sleep(LATENCY).await;
    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.save_settings(settings))
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
    let updates_available = drivers.iter().filter(|d| d.status == "update").count() as u32;
    Ok(Summary {
        devices_total: repo.snapshot().summary.devices_total,
        up_to_date: repo.snapshot().summary.devices_total.saturating_sub(updates_available),
        updates_available,
        last_check: repo.snapshot().summary.last_check,
    })
}
