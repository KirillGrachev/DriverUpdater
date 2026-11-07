/*!
 * Versioned IPC contract between the medium-integrity UI and the elevated
 * Driver Updater service.
 *
 * <p>The contract is deliberately an allowlist: callers cannot send arbitrary
 * commands or executable paths. Every request carries the service token from
 * ProgramData and exactly one typed operation.</p>
 */

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Named pipe used by the UI.
pub const SERVICE_PIPE: &str = r"\\.\pipe\driver-updater-service";
/// Service name registered with the Windows Service Control Manager.
pub const SERVICE_NAME: &str = "DriverUpdaterService";
/// Current IPC contract version.
pub const SERVICE_PROTOCOL_VERSION: u8 = 1;
/// Maximum serialized request size accepted by the service.
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// One typed operation the service is allowed to perform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "op", content = "payload", rename_all = "camelCase")]
pub enum ServiceOperation {
    Ping,
    FindUpdates,
    InstallDriverUpdate { update_id: String },
    CreateRestorePoint { name: String },
    ListRestorePoints,
    DeleteRestorePoint { id: String },
    RestoreRestorePoint { id: String },
    CreateDriverBackup { path: String },
    ListDriverBackups { path: String },
    RestoreDriverBackup { path: String, id: String },
    DeleteDriverBackup { path: String, id: String },
    InstallPackage { package_id: String, version: String },
}

/// Request envelope written by the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequest {
    pub version: u8,
    pub token: String,
    pub operation: ServiceOperation,
}

/// Response envelope written by the service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Builds a successful response.
pub fn success(result: Value) -> ServiceResponse {
    ServiceResponse {
        ok: true,
        result: Some(result),
        error: None,
    }
}

/// Builds a failed response without leaking internals as structured data.
pub fn failure(message: impl Into<String>) -> ServiceResponse {
    ServiceResponse {
        ok: false,
        result: None,
        error: Some(message.into()),
    }
}

/// Validates the envelope before dispatch.
pub fn validate_request(request: &ServiceRequest, expected_token: &str) -> Result<(), String> {
    if request.version != SERVICE_PROTOCOL_VERSION {
        return Err("unsupported service protocol version".to_string());
    }
    if expected_token.is_empty() || request.token != expected_token {
        return Err("service authentication failed".to_string());
    }
    Ok(())
}

/// Allows only the per-user or machine-wide Driver Updater backup roots.
pub fn is_allowed_backup_root(path: &str) -> bool {
    let normalized = path.replace('/', "\\").to_lowercase();
    let normalized = normalized.trim_end_matches('\\');
    (normalized == "c:\\programdata\\driver updater\\backups"
        || (normalized.starts_with("c:\\users\\")
            && normalized.ends_with("\\appdata\\local\\driver updater\\backups")))
        && !normalized.contains("\\..\\")
}

/// Checks that a backup path stays inside the configured application root.
pub fn validate_backup_path(path: &str, allowed_root: &str) -> Result<(), String> {
    let root = std::path::Path::new(allowed_root);
    let candidate = std::path::Path::new(path);
    if !candidate.is_absolute() || !candidate.starts_with(root) {
        return Err("backup path is outside the allowed folder".to_string());
    }
    if candidate
        .components()
        .any(|component| component == std::path::Component::ParentDir)
    {
        return Err("backup path contains a parent-directory segment".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(operation: ServiceOperation) -> ServiceRequest {
        ServiceRequest {
            version: SERVICE_PROTOCOL_VERSION,
            token: "secret".to_string(),
            operation,
        }
    }

    #[test]
    fn round_trips_a_typed_request() {
        let original = request(ServiceOperation::InstallDriverUpdate {
            update_id: "abc".to_string(),
        });
        let encoded = serde_json::to_string(&original).expect("serialize");
        let decoded: ServiceRequest = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, original);
    }

    #[test]
    fn rejects_a_wrong_token_and_version() {
        let mut bad = request(ServiceOperation::Ping);
        bad.token = "nope".to_string();
        assert!(validate_request(&bad, "secret").is_err());
        bad.token = "secret".to_string();
        bad.version = 99;
        assert!(validate_request(&bad, "secret").is_err());
        assert!(validate_request(&request(ServiceOperation::Ping), "secret").is_ok());
    }

    #[test]
    fn accepts_only_application_backup_roots() {
        assert!(is_allowed_backup_root(
            r"C:\ProgramData\Driver Updater\Backups"
        ));
        assert!(is_allowed_backup_root(
            r"C:\Users\alice\AppData\Local\Driver Updater\Backups"
        ));
        assert!(!is_allowed_backup_root(r"D:\Driver Backups"));
        assert!(!is_allowed_backup_root(
            r"C:\Users\alice\AppData\Local\Driver Updater\Backups\..\Windows"
        ));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn rejects_backup_paths_outside_the_root() {
        assert!(validate_backup_path(
            r"C:\ProgramData\Driver Updater\Backups\a",
            r"C:\ProgramData\Driver Updater\Backups"
        )
        .is_ok());
        assert!(
            validate_backup_path(r"C:\Windows\Temp", r"C:\ProgramData\Driver Updater\Backups")
                .is_err()
        );
        assert!(
            validate_backup_path(r"relative\path", r"C:\ProgramData\Driver Updater\Backups")
                .is_err()
        );
    }
}
