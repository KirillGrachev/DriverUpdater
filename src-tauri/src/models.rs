//! Shared serde models — mirror `src/ipc/types.ts` on the webview side.
//! All structs serialize to camelCase to match the TypeScript contracts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedText {
    pub ru: String,
    pub en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverIcon {
    pub kind: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Driver {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub category: String,
    pub icon: DriverIcon,
    pub current: String,
    pub available: Option<String>,
    pub status: String,
    pub size_mb: u32,
    pub recommended: bool,
    pub device: String,
    pub released: String,
    pub description: LocalizedText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub id: String,
    pub name: LocalizedText,
    pub kind: String,
    pub created_at: String,
    pub size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub notifications: bool,
    pub auto_check: bool,
    pub check_interval: String,
    pub restore_point_before_install: bool,
    pub keep_backups: bool,
    pub backup_path: String,
    pub telemetry: bool,
    pub auto_restore_points: bool,
    pub restore_point_period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub devices_total: u32,
    pub up_to_date: u32,
    pub updates_available: u32,
    pub last_check: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialData {
    pub drivers: Vec<Driver>,
    pub restore_points: Vec<RestorePoint>,
    pub settings: Settings,
    pub summary: Summary,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckUpdatesResult {
    pub drivers: Vec<Driver>,
    pub summary: Summary,
}
