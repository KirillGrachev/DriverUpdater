# Third-party notices

Driver Updater is MIT licensed. The following third-party components are
included or required at runtime.

## Fonts

**Montserrat** is distributed under the SIL Open Font License 1.1. The license
text is included at [src/assets/fonts/OFL.txt](src/assets/fonts/OFL.txt).

## Flags

**flag-icons** is distributed under the MIT license:
https://github.com/lipis/flag-icons

## Application framework

- **Tauri** and **tauri-plugin-opener** are distributed under the Apache-2.0 /
  MIT licenses.
- **React** and **React DOM** are distributed under the MIT license.
- **Rust** dependencies are listed with their licenses in
  `src-tauri/Cargo.lock` and the crates.io metadata.
- **Vite**, **TypeScript**, **Vitest** and **Prettier** are development
  dependencies with their own licenses.

## Microsoft runtime

The Windows installer bundles the **Microsoft Edge WebView2 Runtime** offline
installer. Use of the WebView2 Runtime is governed by Microsoft's terms:
https://developer.microsoft.com/microsoft-edge/webview2/

The app links the Microsoft Visual C++ runtime statically through the MSVC
toolchain. No Microsoft binaries are committed to this repository.

## Assets

The application logo, About illustration and UI icons are project assets. Do not
redistribute them as part of another product without permission.
