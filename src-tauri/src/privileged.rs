/*!
 * Privileged operations executed inside the service process.
 *
 * <p>These functions never accept executable paths or raw commands. The
 * service dispatch layer validates the typed request and the caller token
 * before reaching this module.</p>
 */

use serde_json::{json, Value};

use crate::backup::default_backup_provider;
use crate::packages::WingetPackageManager;
use crate::restore::default_restore_provider;
use crate::windows_update::default_update_provider;

/// Searches Windows Update for driver packages.
pub fn find_updates() -> Result<Value, String> {
    serde_json::to_value(default_update_provider().find_updates()?)
        .map_err(|error| error.to_string())
}

/// Installs one Windows Update driver package.
pub fn install_update(update_id: &str) -> Result<Value, String> {
    if update_id.trim().is_empty() {
        return Err("update id is required".to_string());
    }
    let outcome = default_update_provider().install_update(update_id)?;
    Ok(json!({ "rebootRequired": outcome.reboot_required }))
}

/// Creates a restore point and returns it.
pub fn create_restore_point(name: &str) -> Result<Value, String> {
    if name.trim().is_empty() || name.chars().count() > 63 {
        return Err("restore point name must contain 1-63 characters".to_string());
    }
    let point = default_restore_provider().create(name)?;
    serde_json::to_value(point).map_err(|error| error.to_string())
}

/// Lists restore points.
pub fn list_restore_points() -> Result<Value, String> {
    serde_json::to_value(default_restore_provider().list()?).map_err(|error| error.to_string())
}

/// Deletes a restore point.
pub fn delete_restore_point(id: &str) -> Result<Value, String> {
    default_restore_provider().delete(id)?;
    Ok(Value::Null)
}

/// Starts a system restore.
pub fn restore_restore_point(id: &str) -> Result<Value, String> {
    default_restore_provider().restore(id)?;
    Ok(Value::Null)
}

/// Lists driver package backups under an already validated root.
pub fn list_driver_backups(root: &str) -> Result<Value, String> {
    serde_json::to_value(default_backup_provider(std::path::PathBuf::from(root)).list()?)
        .map_err(|error| error.to_string())
}

/// Creates a driver package backup under an already validated root.
pub fn create_driver_backup(root: &str) -> Result<Value, String> {
    let backup = default_backup_provider(std::path::PathBuf::from(root)).create()?;
    serde_json::to_value(backup).map_err(|error| error.to_string())
}

/// Restores a driver package backup.
pub fn restore_driver_backup(root: &str, id: &str) -> Result<Value, String> {
    default_backup_provider(std::path::PathBuf::from(root)).restore(id)?;
    Ok(Value::Null)
}

/// Deletes a driver package backup.
pub fn delete_driver_backup(root: &str, id: &str) -> Result<Value, String> {
    default_backup_provider(std::path::PathBuf::from(root)).delete(id)?;
    Ok(Value::Null)
}

/// Installs one allowlisted winget package.
pub fn install_package(package_id: &str, version: &str) -> Result<Value, String> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = WingetPackageManager.install_cancellable(package_id, version, &cancel)?;
    serde_json::to_value(result).map_err(|error| error.to_string())
}
