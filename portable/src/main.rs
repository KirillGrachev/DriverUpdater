#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

const WEBVIEW2_CLIENT: &str =
    r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

fn main() {
    if let Err(message) = run() {
        show_error(&message);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("Could not locate the portable launcher: {error}"))?;
    let directory = current
        .parent()
        .ok_or_else(|| "The portable launcher has no parent directory.".to_string())?;

    if !webview2_installed() {
        install_webview2(directory)?;
    }

    let candidates = [
        directory.join("Driver Updater.exe"),
        directory.join("driver-updater.exe"),
    ];
    let executable = candidates
        .iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            "Driver Updater.exe was not found next to the portable launcher.".to_string()
        })?;

    Command::new(executable)
        .current_dir(directory)
        .spawn()
        .map_err(|error| format!("Could not start Driver Updater: {error}"))?;
    Ok(())
}

fn webview2_installed() -> bool {
    let key = format!("HKLM\\{WEBVIEW2_CLIENT}");
    if registry_has_version(&key) {
        return true;
    }
    let key = format!("HKCU\\{WEBVIEW2_CLIENT}");
    if registry_has_version(&key) {
        return true;
    }

    for root in [
        std::env::var_os("ProgramFiles(x86)").map(PathBuf::from),
        std::env::var_os("ProgramFiles").map(PathBuf::from),
    ]
    .into_iter()
    .flatten()
    {
        if root
            .join("Microsoft")
            .join("EdgeWebView")
            .join("Application")
            .is_dir()
        {
            return true;
        }
    }
    false
}

fn registry_has_version(key: &str) -> bool {
    let status = Command::new("reg.exe")
        .args(["query", key, "/v", "pv"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    matches!(status, Ok(status) if status.success())
}

fn install_webview2(directory: &Path) -> Result<(), String> {
    let installer = directory
        .join("webview2")
        .join("MicrosoftEdgeWebView2RuntimeInstallerX64.exe");
    if !installer.is_file() {
        return Err(format!(
            "Microsoft Edge WebView2 Runtime is missing and the bundled installer is missing:\n{}",
            installer.display()
        ));
    }
    let status = Command::new(&installer)
        .args(["/silent", "/install"])
        .status()
        .map_err(|error| format!("Could not start the WebView2 Runtime installer: {error}"))?;
    if !status.success() {
        return Err(format!(
            "WebView2 Runtime installation failed with code {}.",
            status.code().unwrap_or(-1)
        ));
    }
    if !webview2_installed() {
        return Err("WebView2 Runtime installed but Windows did not detect it.".to_string());
    }
    Ok(())
}

#[cfg(windows)]
fn show_error(message: &str) {
    let title: Vec<u16> = "Driver Updater Portable\0".encode_utf16().collect();
    let message: Vec<u16> = format!("{message}\0").encode_utf16().collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn show_error(message: &str) {
    eprintln!("{message}");
}
