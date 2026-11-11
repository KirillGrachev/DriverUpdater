fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_manifest(include_str!("app.manifest"));
        resource
            .compile()
            .expect("compile portable launcher resources");
    }
}
