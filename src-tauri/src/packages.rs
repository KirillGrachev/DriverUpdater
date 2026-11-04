//! Package installation through the Windows Package Manager (winget).
//!
//! <p>Only package ids listed in [`ALLOWED_PACKAGES`] can be queried or
//! installed. Arguments are passed as separate process arguments, never through
//! a shell, and versions are checked before they reach winget.</p>

use std::process::Command;
use std::time::Duration;

use crate::process;

/// winget packages exposed by the components page.
pub const ALLOWED_PACKAGES: &[&str] = &[
    "Microsoft.VisualStudio.2022.Community",
    "Microsoft.VisualStudio.2022.Professional",
    "Microsoft.VisualStudio.2022.Enterprise",
    "Microsoft.DotNet.Framework.DeveloperPack_4",
    "Microsoft.WindowsFeature.NetFx3",
    "Microsoft.DotNet.DesktopRuntime.6",
    "Microsoft.DotNet.DesktopRuntime.8",
    "Microsoft.DotNet.DesktopRuntime.9",
    "Microsoft.DirectX",
    "Microsoft.VCRedist.2015+.x64",
    "Microsoft.VCRedist.2015+.x86",
    "Python.Python.3.10",
    "Python.Python.3.11",
    "Python.Python.3.12",
    "Python.Python.3.13",
    "OpenJS.NodeJS.20",
    "OpenJS.NodeJS.22",
    "OpenJS.NodeJS.24",
    "EclipseAdoptium.Temurin.17.JDK",
    "EclipseAdoptium.Temurin.21.JDK",
    "EclipseAdoptium.Temurin.25.JDK",
    "GoLang.Go",
    "Rustlang.Rustup",
    "PHP.PHP.8.2",
    "PHP.PHP.8.3",
    "PHP.PHP.8.4",
    "Git.Git",
    "Kitware.CMake",
    "7zip.7zip",
];

/// Package manager port used by the IPC layer.
pub trait PackageManager: Send + Sync {
    fn versions(&self, package_id: &str) -> Result<Vec<String>, String>;
    fn install(&self, package_id: &str, version: &str) -> Result<(), String>;
}

/// Windows Package Manager implementation.
#[derive(Default)]
pub struct WingetPackageManager;

impl PackageManager for WingetPackageManager {
    fn versions(&self, package_id: &str) -> Result<Vec<String>, String> {
        validate_package_id(package_id)?;
        if package_id == "Microsoft.WindowsFeature.NetFx3" {
            return Ok(vec!["3.5 SP1".to_string()]);
        }
        let mut command = Command::new("winget");
        command.args([
            "show",
            "--id",
            package_id,
            "--exact",
            "--versions",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]);
        let output = process::run(command, Duration::from_secs(25))?;
        if !output.success {
            return Err(command_error(&output));
        }
        Ok(parse_versions(&output.stdout))
    }

    fn install(&self, package_id: &str, version: &str) -> Result<(), String> {
        validate_package_id(package_id)?;
        validate_version(version)?;

        if package_id == "Microsoft.WindowsFeature.NetFx3" {
            return install_netfx3();
        }

        let mut command = Command::new("winget");
        command.args([
            "install",
            "--id",
            package_id,
            "--exact",
            "--version",
            version,
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]);
        let output = process::run(command, Duration::from_secs(45 * 60))?;
        if output.success
            || output.stdout.contains("already installed")
            || output.stderr.contains("already installed")
        {
            return Ok(());
        }
        Err(command_error(&output))
    }
}

/// Extracts version rows from the tabular `winget show --versions` output.
pub fn parse_versions(output: &str) -> Vec<String> {
    let mut versions = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.eq_ignore_ascii_case("version") || line.starts_with('-') {
            continue;
        }
        let valid = line
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_digit())
            && line
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || ".-_+".contains(character));
        if valid && !versions.iter().any(|existing| existing == line) {
            versions.push(line.to_string());
        }
    }
    versions
}

fn validate_package_id(package_id: &str) -> Result<(), String> {
    if ALLOWED_PACKAGES.contains(&package_id) {
        Ok(())
    } else {
        Err(format!("package is not allowed: {package_id}"))
    }
}

fn validate_version(version: &str) -> Result<(), String> {
    let valid = !version.is_empty()
        && version.len() <= 64
        && version
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_+".contains(character));
    if valid {
        Ok(())
    } else {
        Err("invalid package version".to_string())
    }
}

fn command_error(output: &process::ProcessOutput) -> String {
    let details = if !output.stderr.trim().is_empty() {
        output.stderr.trim()
    } else {
        output.stdout.trim()
    };
    let last_line = details.lines().last().unwrap_or("winget failed");
    last_line.chars().take(300).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_table() {
        let output =
            "Found Python 3.12 [Python.Python.3.12]\nVersion\n-------\n3.12.10\n3.12.9\n3.12.9\n";
        assert_eq!(parse_versions(output), vec!["3.12.10", "3.12.9"]);
    }

    #[test]
    fn rejects_unknown_packages_and_versions() {
        assert!(validate_package_id("Evil.Package").is_err());
        assert!(validate_package_id("Python.Python.3.12").is_ok());
        assert!(validate_version("3.12.10; rm -rf /").is_err());
        assert!(validate_version("3.12.10").is_ok());
    }
}

#[cfg(target_os = "windows")]
fn install_netfx3() -> Result<(), String> {
    let mut command = Command::new("dism.exe");
    command.args([
        "/Online",
        "/Enable-Feature",
        "/FeatureName:NetFx3",
        "/All",
        "/NoRestart",
    ]);
    let output = process::run(command, Duration::from_secs(20 * 60))?;
    if output.success || output.code == Some(3010) {
        return Ok(());
    }
    if output.code == Some(740) {
        return Err("administrator rights are required to enable .NET Framework 3.5".to_string());
    }
    Err(command_error(&output))
}

#[cfg(not(target_os = "windows"))]
fn install_netfx3() -> Result<(), String> {
    Err(".NET Framework 3.5 is available only on Windows".to_string())
}
