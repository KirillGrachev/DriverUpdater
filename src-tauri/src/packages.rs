/*!
 * Package installation through the Windows Package Manager (winget).
 *
 * <p>Only package ids listed in [`ALLOWED_PACKAGES`] can be queried or
 * installed. Arguments are passed as separate process arguments, never through
 * a shell, and versions are checked before they reach winget.</p>
 */

use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use serde::Serialize;

use crate::process;

/**
 * Result of a package installation.
 */
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageInstallResult {
    pub cancel_ignored: bool,
}

/**
 * winget packages exposed by the components page.
 */
pub const ALLOWED_PACKAGES: &[&str] = &[
    "Microsoft.VisualStudio.2022.Community",
    "Microsoft.VisualStudio.2022.Professional",
    "Microsoft.VisualStudio.2022.Enterprise",
    "Microsoft.VisualStudio.2022.BuildTools",
    "Microsoft.VisualStudioCode",
    "Microsoft.DotNet.Framework.DeveloperPack_4",
    "Microsoft.WindowsFeature.NetFx3",
    "Microsoft.DotNet.DesktopRuntime.6",
    "Microsoft.DotNet.DesktopRuntime.8",
    "Microsoft.DotNet.DesktopRuntime.9",
    "Microsoft.DotNet.DesktopRuntime.10",
    "Microsoft.DotNet.AspNetCore.8",
    "Microsoft.DotNet.AspNetCore.9",
    "Microsoft.DotNet.AspNetCore.10",
    "Microsoft.DotNet.SDK.10",
    "Microsoft.DirectX",
    "Microsoft.EdgeWebView2Runtime",
    "Microsoft.XNARedist",
    "Microsoft.VCRedist.2015+.x64",
    "Microsoft.VCRedist.2015+.x86",
    "Microsoft.VCRedist.2015+.arm64",
    "KhronosGroup.VulkanRT",
    "KhronosGroup.VulkanSDK",
    "CreativeTechnology.OpenAL",
    "Nvidia.PhysX",
    "Nvidia.CUDA",
    "Python.Python.3.10",
    "Python.Python.3.11",
    "Python.Python.3.12",
    "Python.Python.3.13",
    "Python.Python.3.14",
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
    "PHP.PHP.8.5",
    "RubyInstallerTeam.Ruby.3.4",
    "Google.DartSDK",
    "StrawberryPerl.StrawberryPerl",
    "RProject.R",
    "Oven-sh.Bun",
    "DenoLand.Deno",
    "Swift.Toolchain",
    "LLVM.LLVM",
    "Git.Git",
    "Kitware.CMake",
    "7zip.7zip",
    "Microsoft.WindowsSDK.10.0.26100",
    "Docker.DockerDesktop",
    "Ninja-build.Ninja",
    "Microsoft.PowerShell",
    "Microsoft.WSL",
    "GitHub.cli",
    "Microsoft.WindowsTerminal",
    "NASM.NASM",
    "PostgreSQL.PostgreSQL.17",
    "Oracle.MySQL",
    "SQLite.SQLite",
    "Chocolatey.Chocolatey",
];

/**
 * Windows Package Manager implementation.
 */
#[derive(Default)]
pub struct WingetPackageManager;

impl WingetPackageManager {
    pub fn versions(&self, package_id: &str) -> Result<Vec<String>, String> {
        validate_package_id(package_id)?;
        if package_id == "Microsoft.WindowsFeature.NetFx3" {
            return Ok(vec!["3.5 SP1".to_string()]);
        }
        let executable = winget_executable()?;
        let mut command = Command::new(executable);
        command.args([
            "show",
            "--id",
            package_id,
            "--exact",
            "--source",
            "winget",
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
}

impl WingetPackageManager {
    /**
     * Installs a package and stops the child process when `cancel` is set.
     */
    pub fn install_cancellable(
        &self,
        package_id: &str,
        version: &str,
        cancel: &AtomicBool,
    ) -> Result<PackageInstallResult, String> {
        self.install_inner(package_id, version, Some(cancel))
    }

    fn install_inner(
        &self,
        package_id: &str,
        version: &str,
        cancel: Option<&AtomicBool>,
    ) -> Result<PackageInstallResult, String> {
        validate_package_id(package_id)?;
        validate_version(version)?;

        if package_id == "Microsoft.WindowsFeature.NetFx3" {
            return install_netfx3(cancel);
        }

        let executable = winget_executable()?;
        let mut command = Command::new(executable);
        command.args([
            "install",
            "--id",
            package_id,
            "--exact",
            "--source",
            "winget",
            "--version",
            version,
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]);
        let output =
            run_command(command, Duration::from_secs(45 * 60), cancel).map_err(|error| {
                if error == "process cancelled" {
                    "E_PACKAGE_CANCELLED".to_string()
                } else {
                    error
                }
            })?;
        if winget_success(&output) {
            return Ok(PackageInstallResult {
                cancel_ignored: output.cancel_ignored,
            });
        }
        Err(command_error(&output))
    }
}

fn run_command(
    command: Command,
    timeout: Duration,
    cancel: Option<&AtomicBool>,
) -> Result<process::ProcessOutput, String> {
    match cancel {
        Some(flag) => process::run_cancellable(command, timeout, flag, true),
        None => process::run(command, timeout),
    }
}

/**
 * Extracts version rows from the tabular `winget show --versions` output.
 */
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

fn winget_executable() -> Result<std::path::PathBuf, String> {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .map(|base| {
            base.join("Microsoft")
                .join("WindowsApps")
                .join("winget.exe")
        });
    if let Some(path) = local {
        if path.is_file() {
            return Ok(path);
        }
    }
    Err("Windows Package Manager (winget) is not installed".to_string())
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

#[cfg(target_os = "windows")]
fn winget_success(output: &process::ProcessOutput) -> bool {
    const OK: i32 = 0;
    const ALREADY_INSTALLED: i32 = 0x8A15_002B_u32 as i32;
    const REBOOT_REQUIRED: i32 = 0x8A15_0066_u32 as i32;
    output.success
        || matches!(
            output.code,
            Some(OK) | Some(ALREADY_INSTALLED) | Some(REBOOT_REQUIRED)
        )
}

#[cfg(not(target_os = "windows"))]
fn winget_success(output: &process::ProcessOutput) -> bool {
    output.success
}

#[cfg(target_os = "windows")]
fn command_error(output: &process::ProcessOutput) -> String {
    let details = if !output.stderr.trim().is_empty() {
        output.stderr.trim()
    } else {
        output.stdout.trim()
    };
    let last_line = details.lines().last().unwrap_or("winget failed");
    let suffix = output
        .code
        .map(|code| format!(" (exit code {code})"))
        .unwrap_or_default();
    format!("{last_line}{suffix}").chars().take(300).collect()
}

#[cfg(target_os = "windows")]
fn install_netfx3(cancel: Option<&AtomicBool>) -> Result<PackageInstallResult, String> {
    let mut command = Command::new(process::system_executable("dism.exe"));
    command.args([
        "/Online",
        "/Enable-Feature",
        "/FeatureName:NetFx3",
        "/All",
        "/NoRestart",
    ]);
    let output = run_command(command, Duration::from_secs(20 * 60), cancel).map_err(|error| {
        if error == "process cancelled" {
            "E_PACKAGE_CANCELLED".to_string()
        } else {
            error
        }
    })?;
    if output.success || output.code == Some(3010) {
        return Ok(PackageInstallResult {
            cancel_ignored: output.cancel_ignored,
        });
    }
    if output.code == Some(740) {
        return Err("administrator rights are required to enable .NET Framework 3.5".to_string());
    }
    Err(command_error(&output))
}

#[cfg(not(target_os = "windows"))]
fn install_netfx3(_cancel: Option<&AtomicBool>) -> Result<PackageInstallResult, String> {
    Err(".NET Framework 3.5 is available only on Windows".to_string())
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

    #[test]
    fn package_result_is_serialized_for_the_frontend() {
        let value = serde_json::to_value(PackageInstallResult {
            cancel_ignored: true,
        })
        .expect("json");
        assert_eq!(value["cancelIgnored"], true);
    }
}
