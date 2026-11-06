#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager, Window};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallRequest {
    install_path: String,
    language: String,
    theme: String,
    desktop_shortcut: bool,
}

#[derive(Clone, Serialize)]
struct InstallProgress {
    percent: u32,
    stage: String,
}

#[tauri::command]
fn minimize_setup(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize(window: Window) {
    let _ = if window.is_maximized().unwrap_or(false) {
        window.unmaximize()
    } else {
        window.maximize()
    };
}

#[tauri::command]
fn close_setup(window: Window) {
    let _ = window.close();
}

#[tauri::command]
async fn choose_directory() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choose installation folder")
            .pick_folder()
            .map(|path| path.to_string_lossy().into_owned())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
async fn install(app: AppHandle, request: InstallRequest) -> Result<(), String> {
    let install_path = PathBuf::from(request.install_path.trim());
    if install_path.as_os_str().is_empty() {
        return Err("Choose an installation folder.".to_string());
    }
    let payload = find_payload()?;
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        emit(&app_handle, 8, "Preparing installation files...");
        let log = std::env::temp_dir().join("driver-updater-setup.log");
        let mut command = Command::new(payload_command(&payload));
        if payload.extension().and_then(|value| value.to_str()) == Some("msi") {
            command.args([
                "/i",
                &payload.to_string_lossy(),
                "/qn",
                "/norestart",
                "/l*v",
                &log.to_string_lossy(),
                &format!("INSTALLDIR={}", install_path.to_string_lossy()),
            ]);
        } else {
            command.arg(&payload).arg("/S");
            if !request.desktop_shortcut {
                command.arg("/NS");
            }
            command.arg(format!("/D={}", install_path.to_string_lossy()));
        }
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        emit(&app_handle, 24, "Starting the Windows installer...");
        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start the installer: {error}"))?;
        emit(&app_handle, 52, "Installing Driver Updater...");
        let status = child
            .wait()
            .map_err(|error| format!("Could not wait for the installer: {error}"))?;
        if !status.success() {
            return Err(format!(
                "The Windows installer exited with code {}. See {}",
                status.code().unwrap_or(-1),
                log.display()
            ));
        }
        emit(&app_handle, 94, "Writing application preferences...");
        write_first_run(&request)?;
        emit(&app_handle, 100, "Finishing...");
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn finish_setup(app: AppHandle, launch: bool) {
    if launch {
        let candidates = [
            PathBuf::from(r"C:\Program Files\Driver Updater\Driver Updater.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Driver Updater\Driver Updater.exe"),
        ];
        if let Some(executable) = candidates.iter().find(|path| path.exists()) {
            let _ = Command::new(executable).spawn();
        }
    }
    app.exit(0);
}

fn emit(app: &AppHandle, percent: u32, stage: &str) {
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            percent,
            stage: stage.to_string(),
        },
    );
}

fn payload_command(payload: &Path) -> PathBuf {
    if payload.extension().and_then(|value| value.to_str()) == Some("msi") {
        PathBuf::from("msiexec.exe")
    } else {
        payload.to_path_buf()
    }
}

fn find_payload() -> Result<PathBuf, String> {
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let parent = current
        .parent()
        .ok_or_else(|| "The setup executable has no parent directory.".to_string())?;
    let candidates = [
        parent.join("payload").join("payload.exe"),
        parent.join("payload").join("payload.msi"),
        parent.join("resources").join("payload").join("payload.exe"),
        parent.join("resources").join("payload").join("payload.msi"),
    ];
    for candidate in candidates {
        if candidate.is_file() && candidate != current {
            return Ok(candidate);
        }
    }
    Err("Installer payload not found next to the setup executable.".to_string())
}

fn write_first_run(request: &InstallRequest) -> Result<(), String> {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "LOCALAPPDATA is not available.".to_string())?;
    let directory = local.join("Driver Updater");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create the application data folder: {error}"))?;
    let payload = serde_json::json!({
        "language": request.language,
        "theme": request.theme,
        "desktopShortcut": request.desktop_shortcut,
        "installPath": request.install_path,
    });
    let path = directory.join("installer.json");
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("Could not write installer preferences: {error}"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            minimize_setup,
            toggle_maximize,
            close_setup,
            choose_directory,
            install,
            finish_setup
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("setup") {
                let _ = window.set_focus();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the setup wizard");
}
