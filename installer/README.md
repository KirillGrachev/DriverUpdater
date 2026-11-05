# Driver Updater setup wizard

This is the single user-facing installer for Driver Updater. It is a small
Tauri/WebView2 application with the branded six-step flow:

1. welcome;
2. language;
3. appearance;
4. installation folder;
5. progress;
6. completion.

The actual installation is delegated to the existing signed MSI or NSIS
payload. The payload is kept next to `DriverUpdaterSetup.exe` in the `payload`
directory. This keeps one visual experience while MSI and NSIS remain available
for enterprise deployment and architecture-specific builds.

## Build

Build the payload first:

```powershell
npm run tauri build -- --bundles msi,nsis
```

Then build the wizard for the same Rust target and stage the payload:

```powershell
./scripts/build-setup.ps1 -Target x86_64-pc-windows-msvc
```

The result is written to `artifacts/setup/<arch>/` as a folder containing the
wizard executable and its payload, plus a ZIP archive.

The back end searches for a payload in this order:

- `./payload`;
- `./resources/payload`;
- `../payload`;
- `DRIVER_UPDATER_PAYLOAD`.

The setup writes the selected language and theme to
`%LOCALAPPDATA%\Driver Updater\installer.json`. The main application consumes
that file on its next launch and removes it.
