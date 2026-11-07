#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        if std::env::args().any(|argument| argument == "--console") {
            driver_updater_lib::service::serve_pipe(std::sync::Arc::new(
                std::sync::atomic::AtomicBool::new(false),
            ));
            return;
        }
        driver_updater_lib::service::run_service();
    }
}
