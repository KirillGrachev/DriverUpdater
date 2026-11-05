//! Real Windows System Restore integration.
//!
//! <p>Restore points are created, listed, restored and removed through the
//! Windows System Restore WMI provider and `srclient.dll`. These operations
//! require an elevated process; the caller surfaces the resulting error.</p>

#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(target_os = "windows")]
use serde::Deserialize;

use crate::models::{LocalizedText, RestorePoint};
#[cfg(target_os = "windows")]
use crate::process;

/// System Restore operations exposed to the repository.
pub trait RestorePointProvider: Send + Sync {
    fn list(&self) -> Result<Vec<RestorePoint>, String>;
    fn create(&self, name: &LocalizedText) -> Result<RestorePoint, String>;
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
                name: LocalizedText {
                    ru: point.description.clone(),
                    en: point.description,
                },
                kind: kind_for(point.restore_point_type).to_string(),
                created_at: point.created_at,
                size_mb: 0,
            })
            .collect())
    }

    fn create(&self, name: &LocalizedText) -> Result<RestorePoint, String> {
        let description = if name.en.trim().is_empty() {
            name.ru.trim()
        } else {
            name.en.trim()
        };
        let description = escape_ps(description);
        let script = format!(
            r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$result = Invoke-CimMethod -Namespace root/default -ClassName SystemRestore -MethodName CreateRestorePoint -Arguments @{{ Description='{description}'; EventType=1; RestorePointType=10 }}
if ($result.ReturnValue -ne 0) {{ throw "CreateRestorePoint failed with $($result.ReturnValue)" }}
$point = Get-CimInstance -Namespace root/default -ClassName SystemRestore | Sort-Object SequenceNumber -Descending | Select-Object -First 1
if (-not $point) {{ throw 'restore point was not created' }}
[pscustomobject]@{{
    sequenceNumber = [int]$point.SequenceNumber
    description = [string]$point.Description
    createdAt = if ($point.CreationTime) {{ $point.CreationTime.ToUniversalTime().ToString('o') }} else {{ '' }}
    restorePointType = [int]$point.RestorePointType
}} | ConvertTo-Json -Compress
"#
        );
        let output = process::run_powershell(&script, Duration::from_secs(120))?;
        if !output.success {
            return Err(command_error(&output));
        }
        let point: Point =
            serde_json::from_str(&output.stdout).map_err(|error| error.to_string())?;
        Ok(RestorePoint {
            id: format!("rp-{}", point.sequence_number),
            name: LocalizedText {
                ru: point.description.clone(),
                en: point.description,
            },
            kind: kind_for(point.restore_point_type).to_string(),
            created_at: point.created_at,
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
        let sequence = sequence_number(id)?;
        let script = format!(
            r#"
$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @"
using System.Runtime.InteropServices;
public static class DriverUpdaterRestore {{
    [DllImport("srclient.dll")] public static extern int SRRemoveRestorePoint(int sequence);
}}
"@
$result = [DriverUpdaterRestore]::SRRemoveRestorePoint({sequence})
if ($result -ne 0) {{ throw "SRRemoveRestorePoint failed with $result" }}
"#
        );
        let output = process::run_powershell(&script, Duration::from_secs(120))?;
        if !output.success {
            return Err(command_error(&output));
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

    fn create(&self, _name: &LocalizedText) -> Result<RestorePoint, String> {
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
fn escape_ps(value: &str) -> String {
    value.replace('\'', "''").replace(['\r', '\n'], " ")
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
