mod commands;
mod mock;
mod models;

use commands::AppRepository;
use mock::MockRepository;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir must be resolvable");
            let _ = std::fs::create_dir_all(&state_dir);
            let repo = MockRepository::new(Some(state_dir.join("mock-state.json")));
            app.manage(AppRepository(std::sync::Mutex::new(repo)));
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
            commands::get_system_info,
            commands::compute_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
