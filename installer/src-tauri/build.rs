fn main() {
    println!("cargo:rerun-if-env-changed=DRIVER_UPDATER_PAYLOAD_SHA256");
    tauri_build::build()
}
