use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=DRIVER_UPDATER_PAYLOAD_SHA256");
    println!("cargo:rerun-if-env-changed=DRIVER_UPDATER_SERVICE_BIN");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let embedded = out_dir.join("service.bin");
    match std::env::var_os("DRIVER_UPDATER_SERVICE_BIN") {
        Some(source) => {
            println!(
                "cargo:rerun-if-changed={}",
                PathBuf::from(&source).display()
            );
            std::fs::copy(&source, &embedded).expect("copy service binary");
        }
        None => {
            // Development builds can still be checked without producing a
            // service artifact; installation will reject the empty payload.
            std::fs::write(&embedded, []).expect("write empty service placeholder");
        }
    }

    tauri_build::build()
}
