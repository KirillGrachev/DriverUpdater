mod commands;
mod mock;
mod models;

use commands::AppRepository;
use mock::MockRepository;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir must be resolvable");
            let _ = std::fs::create_dir_all(&state_dir);
            let backup_dir = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .map(|base| base.join("DriverUpdater").join("Backups"))
                .unwrap_or_else(|| state_dir.join("Backups"));
            let _ = std::fs::create_dir_all(&backup_dir);
            let repo = MockRepository::new(Some(state_dir.join("mock-state.json")), backup_dir);
            app.manage(AppRepository(std::sync::Mutex::new(repo)));

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
            commands::check_updates,
            commands::install_driver,
            commands::install_updates,
            commands::list_restore_points,
            commands::create_restore_point,
            commands::delete_restore_point,
            commands::restore_from_point,
            commands::save_settings,
            commands::open_backup_folder,
            commands::open_external_url,
            commands::detect_installed_components,
            commands::get_system_info,
            commands::compute_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
