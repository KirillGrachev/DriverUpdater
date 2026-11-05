# Developer scripts

These files are used by the release workflow and by maintainers. They are not
bundled into the Windows installer and an end user never needs them.

- `build-setup.ps1` builds the unified WebView2 setup wizard for one Windows
  target, stages the matching MSI or NSIS payload next to it, and creates the
  `artifacts/setup/<arch>` archive.
- `package-windows.ps1` builds the portable and symbol archives after Tauri
  creates the MSI and NSIS bundles.
- `smoke-msi.ps1` extracts and validates the MSI payload without installing the
  running application.
- `smoke-nsis.ps1` silently installs and uninstalls the NSIS payload to verify
  the complete installer lifecycle.
- `generate_icons.py` regenerates the application icon set from the supplied
  branding asset.
- `check-version.mjs` verifies that the package, Cargo and Tauri versions match.
- `check-locales.mjs` rejects new missing translation keys against
  `shared/locales-baseline.json`.

The installer contains only the compiled application and its frontend assets.
