#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let Ok(current) = std::env::current_exe() else {
        return;
    };
    let Some(directory) = current.parent() else {
        return;
    };
    let candidates: [PathBuf; 2] = [
        directory.join("Driver Updater.exe"),
        directory.join("driver-updater.exe"),
    ];
    if let Some(executable) = candidates.iter().find(|path| path.is_file()) {
        let _ = Command::new(executable).spawn();
    }
}
