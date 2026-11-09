#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

const WEBVIEW2_CLIENT: &str =
    r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

fn main() {
    if let Err(error) = run() {
        show_error(&error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let directory = env::current_exe()
        .map_err(|error| format!("Не удалось определить папку установщика: {error}"))?
        .parent()
        .ok_or_else(|| "Не удалось определить папку установщика".to_string())?
        .to_path_buf();

    if !webview2_installed() {
        install_webview2(&directory)?;
    }

    let ui = directory.join("driver-updater-setup-ui.exe");
    if !ui.is_file() {
        return Err(format!("Не найден установщик:\n{}", ui.display()));
    }

    let status = Command::new(&ui)
        .args(env::args().skip(1))
        .current_dir(&directory)
        .status()
        .map_err(|error| format!("Не удалось запустить установщик: {error}"))?;

    if !status.success() {
        return Err(format!(
            "Установщик завершился с кодом {}",
            status.code().unwrap_or(-1)
        ));
    }
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
        env::var_os("ProgramFiles(x86)").map(PathBuf::from),
        env::var_os("ProgramFiles").map(PathBuf::from),
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
            "На этом компьютере отсутствует WebView2 Runtime, а offline-установщик не найден:\n{}",
            installer.display()
        ));
    }

    let status = Command::new(&installer)
        .args(["/silent", "/install"])
        .status()
        .map_err(|error| format!("Не удалось запустить установку WebView2 Runtime: {error}"))?;

    if !status.success() {
        return Err(format!(
            "Установка WebView2 Runtime завершилась с кодом {}.\nУстановите Microsoft Edge WebView2 Runtime вручную и повторите установку.",
            status.code().unwrap_or(-1)
        ));
    }

    if !webview2_installed() {
        return Err(
            "WebView2 Runtime установлен, но Windows его не видит. Перезагрузите компьютер и повторите установку."
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn show_error(message: &str) {
    let title: Vec<u16> = "Driver Updater Setup\0".encode_utf16().collect();
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
