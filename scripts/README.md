# Developer scripts

These files are used by the release workflow and by maintainers. They are not
bundled into the Windows installer and an end user never needs them.

- `package-windows.ps1` builds the portable and symbol archives after Tauri
  creates the MSI and NSIS bundles.
- `generate_icons.py` regenerates the application icon set from the supplied
  branding asset.
- `publish-wiki.ps1` synchronises `docs/wiki` with the GitHub wiki.
- `check-version.mjs` verifies that the package, Cargo and Tauri versions match.

The installer contains only the compiled application and its frontend assets.
