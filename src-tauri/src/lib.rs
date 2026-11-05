mod backup;
mod commands;
mod detection;
mod models;
mod packages;
mod process;
mod repository;
mod restore;
mod security;
mod storage;
mod windows_update;

use commands::AppRepository;
use repository::DriverRepository;
use tauri::{Emitter, Manager};

/// Gives the backups folder the app icon in Explorer instead of the generic
/// folder glyph. Windows reads this from {@code desktop.ini} and expects the
/// folder itself to be marked read-only.
///
/// @param dir backups directory created during application setup
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
                let _ = std::fs::create_dir_all(directory);
            }
            let backup_dir = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .map(|base| base.join("DriverUpdater").join("Backups"))
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

            // Keep the taskbar/window icon in sync with the bundled app icon.
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
