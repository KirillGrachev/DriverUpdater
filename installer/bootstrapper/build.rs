use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.compile().expect("embed bootstrapper manifest");
    }

    println!("cargo:rerun-if-env-changed=DRIVER_UPDATER_BUNDLE_DIR");
    println!("cargo:rerun-if-env-changed=DRIVER_UPDATER_PAYLOAD_NAME");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let mut generated = String::new();

    if let Some(bundle_dir) = env::var_os("DRIVER_UPDATER_BUNDLE_DIR") {
        let root = PathBuf::from(bundle_dir);
        let payload_name =
            env::var("DRIVER_UPDATER_PAYLOAD_NAME").unwrap_or_else(|_| "payload.exe".to_string());
        let assets = [
            ("SETUP_UI", root.join("driver-updater-setup-ui.exe")),
            (
                "WEBVIEW2",
                root.join("webview2")
                    .join("MicrosoftEdgeWebView2RuntimeInstallerX64.exe"),
            ),
            ("PAYLOAD", root.join("payload").join(payload_name.as_str())),
        ];
        for (name, path) in &assets {
            println!("cargo:rerun-if-changed={}", path.display());
            if !path.is_file() {
                panic!("setup bundle asset is missing: {}", path.display());
            }
            generated.push_str(&format!(
                "pub const {name}: &[u8] = include_bytes!({:?});\n",
                path
            ));
        }
        generated.push_str(&format!(
            "pub const PAYLOAD_NAME: &str = {:?};\n",
            payload_name
        ));
    } else {
        generated.push_str("pub const SETUP_UI: &[u8] = &[];\n");
        generated.push_str("pub const WEBVIEW2: &[u8] = &[];\n");
        generated.push_str("pub const PAYLOAD: &[u8] = &[];\n");
        generated.push_str("pub const PAYLOAD_NAME: &str = \"payload.exe\";\n");
    }

    fs::write(out_dir.join("bundle_assets.rs"), generated).expect("write bundle assets");
}
