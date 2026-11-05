//! Driver package backups through `pnputil`.
//!
//! <p>A backup exports every third-party driver package into a timestamped
//! directory. Restoring adds the exported INF files back to the driver store.
//! Export and restore require administrator rights.</p>

#[cfg(target_os = "windows")]
use std::fs;
#[cfg(target_os = "windows")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use crate::process;
use serde::{Deserialize, Serialize};

/// Metadata for one exported driver package set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverBackup {
    pub id: String,
    pub created_at: String,
    pub path: String,
    pub size_mb: u64,
    pub package_count: u32,
}

pub trait DriverBackupProvider: Send + Sync {
    fn list(&self) -> Result<Vec<DriverBackup>, String>;
    fn create(&self) -> Result<DriverBackup, String>;
    fn restore(&self, id: &str) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// Native `pnputil` implementation rooted at the application backup folder.
#[cfg(target_os = "windows")]
pub struct WindowsDriverBackups {
    root: PathBuf,
}

#[cfg(target_os = "windows")]
impl WindowsDriverBackups {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn backup_path(&self, id: &str) -> Result<PathBuf, String> {
        if !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
            || id.is_empty()
        {
            return Err("invalid driver backup id".to_string());
        }
        let path = self.root.join(id);
        if path.parent() != Some(self.root.as_path()) {
            return Err("invalid driver backup path".to_string());
        }
        Ok(path)
    }

    fn read_metadata(&self, directory: &Path) -> Option<DriverBackup> {
        let raw = fs::read_to_string(directory.join("backup.json")).ok()?;
        let mut backup: DriverBackup = serde_json::from_str(&raw).ok()?;
        if backup.id.is_empty() {
            backup.id = directory.file_name()?.to_string_lossy().into_owned();
        }
        Some(backup)
    }
}

#[cfg(target_os = "windows")]
impl DriverBackupProvider for WindowsDriverBackups {
    fn list(&self) -> Result<Vec<DriverBackup>, String> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut backups = Vec::new();
        let entries = fs::read_dir(&self.root).map_err(|error| error.to_string())?;
        for entry in entries.flatten() {
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            if let Some(backup) = self.read_metadata(&entry.path()) {
                backups.push(backup);
            }
        }
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    fn create(&self) -> Result<DriverBackup, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_secs())
            .unwrap_or(0);
        let id = format!("backup-{stamp}");
        let directory = self.backup_path(&id)?;
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

        let mut command = std::process::Command::new(process::system_executable("pnputil.exe"));
        command.args(["/export-driver", "*", &directory.to_string_lossy()]);
        let output = process::run(command, Duration::from_secs(15 * 60))?;
        if !output.success {
            let _ = fs::remove_dir_all(&directory);
            return Err(command_error(&output));
        }

        let (size_mb, package_count) = directory_stats(&directory)?;
        let backup = DriverBackup {
            id,
            created_at: now_iso(),
            path: directory.to_string_lossy().into_owned(),
            size_mb,
            package_count,
        };
        let metadata = serde_json::to_vec_pretty(&backup).map_err(|error| error.to_string())?;
        fs::write(directory.join("backup.json"), metadata).map_err(|error| error.to_string())?;
        Ok(backup)
    }

    fn restore(&self, id: &str) -> Result<(), String> {
        let directory = self.backup_path(id)?;
        if !directory.exists() {
            return Err("driver backup not found".to_string());
        }
        let mut inf_files = Vec::new();
        collect_inf_files(&directory, &mut inf_files)?;
        if inf_files.is_empty() {
            return Err("driver backup contains no INF packages".to_string());
        }
        let total = inf_files.len();
        let mut failures: Vec<String> = Vec::new();
        for inf in inf_files {
            let mut command = std::process::Command::new(process::system_executable("pnputil.exe"));
            command.args(["/add-driver", &inf.to_string_lossy(), "/install"]);
            match process::run(command, Duration::from_secs(10 * 60)) {
                Ok(output) if output.success => {}
                Ok(output) => {
                    failures.push(format!("{}: {}", inf.display(), command_error(&output)))
                }
                Err(error) => failures.push(format!("{}: {error}", inf.display())),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "{} of {} driver packages failed; first error: {}",
                failures.len(),
                total,
                failures[0]
            ))
        }
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let directory = self.backup_path(id)?;
        if !directory.exists() {
            return Err("driver backup not found".to_string());
        }
        fs::remove_dir_all(directory).map_err(|error| error.to_string())
    }
}

/// Non-Windows placeholder.
#[cfg(not(target_os = "windows"))]
#[derive(Default)]
pub struct UnsupportedDriverBackups;

#[cfg(not(target_os = "windows"))]
impl DriverBackupProvider for UnsupportedDriverBackups {
    fn list(&self) -> Result<Vec<DriverBackup>, String> {
        Err("driver backups are available only on Windows".to_string())
    }

    fn create(&self) -> Result<DriverBackup, String> {
        Err("driver backups are available only on Windows".to_string())
    }

    fn restore(&self, _id: &str) -> Result<(), String> {
        Err("driver backups are available only on Windows".to_string())
    }

    fn delete(&self, _id: &str) -> Result<(), String> {
        Err("driver backups are available only on Windows".to_string())
    }
}

/// Returns the provider rooted at the application backup directory.
pub fn default_backup_provider(root: std::path::PathBuf) -> Box<dyn DriverBackupProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsDriverBackups::new(root))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = root;
        Box::<UnsupportedDriverBackups>::default()
    }
}

#[cfg(target_os = "windows")]
fn collect_inf_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_inf_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("inf"))
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn directory_stats(directory: &Path) -> Result<(u64, u32), String> {
    let mut bytes = 0_u64;
    let mut packages = 0_u32;
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let (nested_bytes, nested_packages) = directory_stats(&path)?;
            bytes += nested_bytes;
            packages += nested_packages;
        } else {
            if let Ok(metadata) = entry.metadata() {
                bytes += metadata.len();
            }
            if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("inf"))
            {
                packages += 1;
            }
        }
    }
    Ok((bytes / (1024 * 1024), packages))
}

#[cfg(target_os = "windows")]
fn now_iso() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
    let days = seconds / 86_400;
    let remainder = seconds % 86_400;
    let (year, month, day) = crate::detection::days_to_civil(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        remainder / 3600,
        (remainder % 3600) / 60,
        remainder % 60
    )
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
        .unwrap_or("pnputil failed")
        .chars()
        .take(300)
        .collect()
}
