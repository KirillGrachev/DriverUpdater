/*!
 * Location and accessors for the local service authentication token.
 */

use std::path::PathBuf;

/// Returns the token path shared by the installer, service, and UI.
pub fn path() -> Result<PathBuf, String> {
    let program_data = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "PROGRAMDATA is not available".to_string())?;
    Ok(program_data.join("Driver Updater").join("service.token"))
}

/// Reads the token written by the setup wizard.
pub fn read() -> Result<String, String> {
    if let Ok(token) = std::env::var("DRIVER_UPDATER_SERVICE_TOKEN") {
        if token.len() >= 32 {
            return Ok(token);
        }
    }
    let path = path()?;
    let token = std::fs::read_to_string(&path)
        .map_err(|error| format!("could not read the service token: {error}"))?;
    let token = token.trim().to_string();
    if token.len() < 32 {
        return Err("service token is too short".to_string());
    }
    Ok(token)
}

/// Root allowed for driver backup operations.
pub fn backup_root() -> String {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\ProgramData"));
    local
        .join("Driver Updater")
        .join("Backups")
        .to_string_lossy()
        .to_string()
}
