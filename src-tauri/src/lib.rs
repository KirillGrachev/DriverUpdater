mod backup;
mod commands;
mod detection;
mod error;
mod models;
mod packages;
#[cfg(target_os = "windows")]
mod privileged;
mod process;
mod repository;
mod restore;
mod security;
pub mod service;
mod service_client;
pub mod service_contract;
#[cfg(target_os = "windows")]
mod service_token;
mod storage;
mod windows_update;

use std::path::{Path, PathBuf};

use commands::AppRepository;
use repository::DriverRepository;
use tauri::{Emitter, Manager};

/**
 * Gives the backups folder the app icon in Explorer instead of the generic
 * folder glyph. Windows reads this from {@code desktop.ini} and expects the
 * folder itself to be marked read-only.
 *
 * @param dir backups directory created during application setup
 */
#[cfg(target_os = "windows")]
fn apply_backup_folder_icon(dir: &std::path::Path) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let ini = format!("[.ShellClassInfo]\r\nIconResource={},0\r\n", exe.display());
    let mut bytes = vec![0xFF, 0xFE];
    for unit in ini.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }

    let ini_path = dir.join("desktop.ini");
    if std::fs::write(&ini_path, bytes).is_err() {
        return;
    }

    let _ = std::process::Command::new(crate::process::system_executable("attrib.exe"))
        .creation_flags(CREATE_NO_WINDOW)
        .args(["+r", dir.to_string_lossy().as_ref()])
        .status();
    let _ = std::process::Command::new(crate::process::system_executable("attrib.exe"))
        .creation_flags(CREATE_NO_WINDOW)
        .args(["+h", "+s", ini_path.to_string_lossy().as_ref()])
        .status();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn migrate_legacy_appdata(target: &Path) {
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return;
    };
    let legacy = std::path::PathBuf::from(appdata).join("dev.driverupdater.desktop");
    if !legacy.is_dir() || legacy == target {
        return;
    }
    let _ = std::fs::create_dir_all(target);
    for name in ["state.json", "mock-state.json", "installed-drivers.json"] {
        let source = legacy.join(name);
        let destination = target.join(name);
        if source.is_file() && !destination.exists() {
            let _ = std::fs::copy(source, destination);
        }
    }
}

const APP_IDENTIFIER: &str = "com.github.kirillgrachev.driverupdater";

fn migrate_legacy_backups(local_app_data: &Path) -> PathBuf {
    let target = local_app_data.join("Driver Updater").join("Backups");
    let sources = [
        local_app_data.join(APP_IDENTIFIER).join("Backups"),
        local_app_data.join("DriverUpdater").join("Backups"),
    ];
    let has_content = |path: &Path| {
        std::fs::read_dir(path)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    !entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case("desktop.ini")
                })
            })
            .unwrap_or(false)
    };
    if target.exists() {
        if has_content(&target) || sources.iter().all(|source| !source.exists()) {
            return target;
        }
        let _ = std::fs::remove_dir_all(&target);
    }
    for source in sources {
        if !source.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::rename(&source, &target).is_err() {
            log::warn!("could not move the legacy backup folder");
            return source;
        }
        rewrite_backup_paths(&target, &source, &target);
        return target;
    }
    target
}

fn rewrite_backup_paths(root: &Path, old_root: &Path, new_root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let metadata = entry.path().join("backup.json");
        let Ok(raw) = std::fs::read_to_string(&metadata) else {
            continue;
        };
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let Some(path) = value
            .get("path")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };
        let old_text = old_root.to_string_lossy();
        let new_text = new_root.to_string_lossy();
        let replacement = path.replace(old_text.as_ref(), new_text.as_ref());
        if replacement != path {
            value["path"] = serde_json::Value::String(replacement);
            if let Ok(updated) = serde_json::to_string_pretty(&value) {
                let _ = std::fs::write(&metadata, updated);
            }
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            |app, _arguments, _cwd| {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            },
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("driver-updater".to_string()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let busy = window
                    .app_handle()
                    .state::<commands::BusyState>()
                    .0
                    .load(std::sync::atomic::Ordering::SeqCst);
                if busy > 0 {
                    api.prevent_close();
                    let _ = window.emit("close-blocked", ());
                }
            }
        })
        .setup(|app| {
            log::info!("Driver Updater {} starting", env!("CARGO_PKG_VERSION"));
            let state_dir = app.path().app_data_dir().ok();
            if let Some(directory) = &state_dir {
                migrate_legacy_appdata(directory);
                let _ = std::fs::create_dir_all(directory);
            }
            let backup_dir = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .map(|base| migrate_legacy_backups(&base))
                .or_else(|| {
                    state_dir
                        .as_ref()
                        .map(|directory| directory.join("Backups"))
                })
                .unwrap_or_else(|| std::env::temp_dir().join("DriverUpdater").join("Backups"));
            let _ = std::fs::create_dir_all(&backup_dir);
            #[cfg(target_os = "windows")]
            apply_backup_folder_icon(&backup_dir);
            let state_file = state_dir.map(|directory| {
                let current = directory.join("state.json");
                let legacy = directory.join("mock-state.json");
                if !current.exists() && legacy.exists() {
                    let _ = std::fs::rename(&legacy, &current);
                }
                if current.exists() || !legacy.exists() {
                    current
                } else {
                    legacy
                }
            });
            let repo = DriverRepository::new(state_file, backup_dir);
            app.manage(AppRepository(std::sync::Arc::new(std::sync::Mutex::new(
                repo,
            ))));
            app.manage(commands::PackageJobs::default());
            app.manage(commands::BusyState::default());

            /*
             * Keep the taskbar/window icon in sync with the bundled app icon.
             */
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::new_owned(
                    include_bytes!("../icons/icon_256.rgba").to_vec(),
                    256,
                    256,
                );
                let _ = window.set_icon(icon);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_data,
            commands::scan_inventory,
            commands::check_updates,
            commands::install_driver,
            commands::install_updates,
            commands::list_restore_points,
            commands::create_restore_point,
            commands::delete_restore_point,
            commands::restore_from_point,
            commands::save_settings,
            commands::open_backup_folder,
            commands::detect_installed_components,
            commands::get_installed_components,
            commands::save_installed_components,
            commands::list_driver_backups,
            commands::create_driver_backup,
            commands::restore_driver_backup,
            commands::delete_driver_backup,
            commands::list_package_versions,
            commands::install_package,
            commands::cancel_package_install,
            commands::get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
