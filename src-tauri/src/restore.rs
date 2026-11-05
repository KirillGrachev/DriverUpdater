//! Real Windows System Restore integration.
//!
//! <p>Restore points are created, listed, restored and removed through the
//! Windows System Restore WMI provider and `srclient.dll`. These operations
//! require an elevated process; the caller surfaces the resulting error.</p>

#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(target_os = "windows")]
use serde::Deserialize;

use crate::models::RestorePoint;
#[cfg(target_os = "windows")]
use crate::process;

#[cfg(target_os = "windows")]
const BEGIN_SYSTEM_CHANGE: u32 = 100;
#[cfg(target_os = "windows")]
const MODIFY_SETTINGS: u32 = 12;

#[cfg(target_os = "windows")]
#[repr(C)]
struct RestorePointInfo {
    event_type: u32,
    restore_point_type: u32,
    sequence_number: i64,
    description: [u16; 256],
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct StateManagerStatus {
    status: u32,
    sequence_number: i64,
}

#[cfg(target_os = "windows")]
#[link(name = "srclient")]
extern "system" {
    fn SRSetRestorePointW(
        restore_point: *const RestorePointInfo,
        status: *mut StateManagerStatus,
    ) -> i32;
    fn SRRemoveRestorePoint(sequence_number: u32) -> u32;
}

#[cfg(target_os = "windows")]
fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
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

/// System Restore operations exposed to the repository.
pub trait RestorePointProvider: Send + Sync {
    fn list(&self) -> Result<Vec<RestorePoint>, String>;
    fn create(&self, name: &str) -> Result<RestorePoint, String>;
    fn restore(&self, id: &str) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// Native Windows implementation.
#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct WindowsRestorePoints;

#[cfg(target_os = "windows")]
impl RestorePointProvider for WindowsRestorePoints {
    fn list(&self) -> Result<Vec<RestorePoint>, String> {
        let script = r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$points = @(Get-CimInstance -Namespace root/default -ClassName SystemRestore | Sort-Object SequenceNumber -Descending)
$items = @()
foreach ($point in $points) {
    $items += [pscustomobject]@{
        sequenceNumber = [int]$point.SequenceNumber
        description = [string]$point.Description
        createdAt = if ($point.CreationTime) { $point.CreationTime.ToUniversalTime().ToString('o') } else { '' }
        restorePointType = [int]$point.RestorePointType
    }
}
$payload = [pscustomobject]@{ points = @($items) }
ConvertTo-Json -InputObject $payload -Compress -Depth 4
"#;
        let output = process::run_powershell(script, Duration::from_secs(60))?;
        if !output.success {
            return Err(command_error(&output));
        }
        let envelope: Envelope =
            serde_json::from_str(&output.stdout).map_err(|error| error.to_string())?;
        Ok(envelope
            .points
            .into_iter()
            .map(|point| RestorePoint {
                id: format!("rp-{}", point.sequence_number),
                name: point.description,
                kind: kind_for(point.restore_point_type).to_string(),
                created_at: point.created_at,
                size_mb: 0,
            })
            .collect())
    }

    fn create(&self, name: &str) -> Result<RestorePoint, String> {
        let description = name.trim();
        if description.is_empty() || description.chars().count() > 200 {
            return Err("invalid restore point name".to_string());
        }
        let mut wide: Vec<u16> = description.encode_utf16().take(255).collect();
        wide.push(0);
        let mut spec = RestorePointInfo {
            event_type: BEGIN_SYSTEM_CHANGE,
            restore_point_type: MODIFY_SETTINGS,
            sequence_number: 0,
            description: [0; 256],
        };
        for (slot, unit) in spec.description.iter_mut().zip(wide) {
            *slot = unit;
        }
        let mut status = StateManagerStatus {
            status: 1,
            sequence_number: 0,
        };
        let ok = unsafe { SRSetRestorePointW(&spec, &mut status) };
        if ok == 0 || status.status != 0 {
            return Err(format!(
                "System Restore failed with status {}",
                status.status
            ));
        }
        Ok(RestorePoint {
            id: format!("rp-{}", status.sequence_number),
            name: description.to_string(),
            kind: "manual".to_string(),
            created_at: now_iso(),
            size_mb: 0,
        })
    }

    fn restore(&self, id: &str) -> Result<(), String> {
        let sequence = sequence_number(id)?;
        let script = format!(
            r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$result = Invoke-CimMethod -Namespace root/default -ClassName SystemRestore -MethodName Restore -Arguments @{{ SequenceNumber={sequence} }}
if ($result.ReturnValue -ne 0) {{ throw "System Restore failed with $($result.ReturnValue)" }}
"#
        );
        let output = process::run_powershell(&script, Duration::from_secs(10 * 60))?;
        if !output.success {
            return Err(command_error(&output));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let sequence = u32::try_from(sequence_number(id)?)
            .map_err(|_| "invalid restore point id".to_string())?;
        let result = unsafe { SRRemoveRestorePoint(sequence) };
        if result != 0 {
            return Err(format!("SRRemoveRestorePoint failed with {result}"));
        }
        Ok(())
    }
}

/// Non-Windows placeholder.
#[cfg(not(target_os = "windows"))]
#[derive(Default)]
pub struct UnsupportedRestorePoints;

#[cfg(not(target_os = "windows"))]
impl RestorePointProvider for UnsupportedRestorePoints {
    fn list(&self) -> Result<Vec<RestorePoint>, String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn create(&self, _name: &str) -> Result<RestorePoint, String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn restore(&self, _id: &str) -> Result<(), String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn delete(&self, _id: &str) -> Result<(), String> {
        Err("System Restore is available only on Windows".to_string())
    }
}

/// Returns the provider for the current target.
pub fn default_restore_provider() -> Box<dyn RestorePointProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::<WindowsRestorePoints>::default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::<UnsupportedRestorePoints>::default()
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
struct Envelope {
    #[serde(default)]
    points: Vec<Point>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Point {
    sequence_number: i64,
    #[serde(default)]
    description: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    restore_point_type: i32,
}

#[cfg(target_os = "windows")]
fn kind_for(restore_point_type: i32) -> &'static str {
    match restore_point_type {
        10 => "auto",
        12 => "manual",
        _ => "manual",
    }
}

#[cfg(target_os = "windows")]
fn sequence_number(id: &str) -> Result<i64, String> {
    let value = id
        .strip_prefix("rp-")
        .ok_or_else(|| "invalid restore point id".to_string())?;
    value
        .parse::<i64>()
        .map_err(|_| "invalid restore point id".to_string())
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
        .unwrap_or("System Restore failed")
        .chars()
        .take(300)
        .collect()
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn formats_the_current_time_as_utc_iso() {
        let value = now_iso();
        assert_eq!(value.len(), 20);
        assert!(value.ends_with('Z'));
        assert_eq!(&value[4..5], "-");
        assert_eq!(&value[10..11], "T");
    }

    #[test]
    fn parses_only_numeric_restore_point_ids() {
        assert_eq!(sequence_number("rp-42").expect("id"), 42);
        assert!(sequence_number("42").is_err());
        assert!(sequence_number("rp-nope").is_err());
    }
}
