#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::process::Stdio;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallationSpace {
    required_bytes: u64,
    available_bytes: u64,
}

#[tauri::command]
fn installation_space(path: String) -> InstallationSpace {
    let payload_bytes = find_payload()
        .ok()
        .and_then(|payload| std::fs::metadata(payload).ok())
        .map(|metadata| metadata.len())
        .unwrap_or(120 * 1024 * 1024);
    InstallationSpace {
        required_bytes: payload_bytes.saturating_add(64 * 1024 * 1024),
        available_bytes: available_bytes(&path).unwrap_or(0),
    }
}

#[cfg(target_os = "windows")]
fn available_bytes(path: &str) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let mut existing = PathBuf::from(path);
    while !existing.exists() {
        if !existing.pop() {
            return None;
        }
    }
    let wide: Vec<u16> = existing
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut free = 0u64;
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, null_mut(), null_mut()) };
    (ok != 0).then_some(free)
}

#[cfg(not(target_os = "windows"))]
fn available_bytes(_path: &str) -> Option<u64> {
    None
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
    verify_payload(&payload)?;
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
        emit(&app_handle, 94, "Registering the Windows service...");
        install_service(&install_path)?;
        emit(&app_handle, 97, "Writing application preferences...");
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

fn verify_payload(payload: &Path) -> Result<(), String> {
    let Some(expected) = option_env!("DRIVER_UPDATER_PAYLOAD_SHA256") else {
        return Ok(());
    };
    let bytes = std::fs::read(payload)
        .map_err(|error| format!("Could not read the installer payload: {error}"))?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected.to_ascii_lowercase() {
        return Err(
            "The installer payload failed its integrity check. Download the setup again."
                .to_string(),
        );
    }
    Ok(())
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

#[cfg(target_os = "windows")]
static SERVICE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/service.bin"));

#[cfg(target_os = "windows")]
fn system_sc() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("sc.exe")
}

#[cfg(target_os = "windows")]
fn system_command(program: &Path) -> Command {
    let mut command = Command::new(program);
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
    command
}

#[cfg(target_os = "windows")]
fn current_user_sid() -> Result<String, String> {
    let whoami = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("whoami.exe");
    let output = system_command(&whoami)
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .map_err(|error| format!("Could not resolve the current user SID: {error}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let sid = text
        .trim()
        .rsplit(',')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .to_string();
    if !sid.starts_with("S-1-") {
        return Err("Could not resolve the current user SID.".to_string());
    }
    Ok(sid)
}

#[cfg(target_os = "windows")]
fn protect_service_token(token: &Path) -> Result<(), String> {
    let icacls = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("icacls.exe");
    let sid = current_user_sid()?;
    let status = system_command(&icacls)
        .arg(token)
        .args([
            "/inheritance:r",
            "/grant:r",
            &format!("*{sid}:R"),
            "/grant:r",
            "*S-1-5-18:F",
            "/grant:r",
            "*S-1-5-32-544:F",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("Could not protect the service token: {error}"))?;
    if !status.success() {
        return Err("Could not restrict access to the service token.".to_string());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_service(install_path: &Path) -> Result<(), String> {
    const SERVICE_NAME: &str = "DriverUpdaterService";
    if SERVICE_BYTES.is_empty() {
        return Err("The Windows service binary is missing from this setup build.".to_string());
    }
    std::fs::create_dir_all(install_path).map_err(|error| error.to_string())?;
    let service_path = install_path.join("driver-updater-service.exe");
    let sc = system_sc();

    let _ = system_command(&sc)
        .args(["stop", SERVICE_NAME])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    std::thread::sleep(std::time::Duration::from_secs(2));
    for _ in 0..4 {
        let deleted = system_command(&sc)
            .args(["delete", SERVICE_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if deleted {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    std::thread::sleep(std::time::Duration::from_millis(500));

    std::fs::write(&service_path, SERVICE_BYTES)
        .map_err(|error| format!("Could not install the Windows service: {error}"))?;

    let token_dir = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "PROGRAMDATA is not available.".to_string())?
        .join("Driver Updater");
    std::fs::create_dir_all(&token_dir).map_err(|error| error.to_string())?;
    let mut token_bytes = [0u8; 32];
    getrandom::fill(&mut token_bytes).map_err(|error| error.to_string())?;
    let token: String = token_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let token_path = token_dir.join("service.token");
    std::fs::write(&token_path, token.as_bytes())
        .map_err(|error| format!("Could not write the service token: {error}"))?;
    protect_service_token(&token_path)?;

    let bin_path = format!("\"{}\"", service_path.display());
    let status = system_command(&sc)
        .args([
            "create",
            SERVICE_NAME,
            "binPath=",
            &bin_path,
            "start=",
            "auto",
            "obj=",
            "LocalSystem",
        ])
        .status()
        .map_err(|error| format!("Could not register the Windows service: {error}"))?;
    if !status.success() {
        return Err(format!(
            "sc.exe create failed with code {}.",
            status.code().unwrap_or(-1)
        ));
    }
    let status = system_command(&sc)
        .args(["start", SERVICE_NAME])
        .status()
        .map_err(|error| format!("Could not start the Windows service: {error}"))?;
    if !status.success() {
        return Err(format!(
            "sc.exe start failed with code {}.",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn install_service(_install_path: &Path) -> Result<(), String> {
    Ok(())
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
            installation_space,
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
