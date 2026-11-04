# Contributing to Driver Updater

Thanks for taking the time to improve the project. This guide covers the local
setup, the architecture boundary and the conventions used in the repository.

## Before you start

- Read [README.md](README.md) for the product and architecture overview.
- For a security issue, follow [SECURITY.md](SECURITY.md) instead of opening a
  public issue.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Development setup

Requirements:

- Windows 10/11 x64;
- Node.js 20+ and npm;
- Rust stable with the MSVC toolchain;
- Microsoft C++ Build Tools and the Windows SDK;
- Tauri prerequisites: https://v2.tauri.app/start/prerequisites/

```powershell
git clone https://github.com/KirillGrachev/DriverUpdater.git
cd DriverUpdater
npm install
npm run tauri dev
```

The browser-only preview is useful for UI work:

```powershell
npm run dev
```

It uses `MockBackend` and the shared fixtures, so Windows APIs are not required.

## Project layout

| Path | Responsibility |
|---|---|
| `src/pages` | One component per screen |
| `src/components` | Titlebar, bottom dock, UI kit, icons, overlays |
| `src/store/app.tsx` | Routing, IPC state, settings queue, theme, toasts |
| `src/ipc` | `Backend` contract and the Tauri/mock implementations |
| `src/i18n` | Reference dictionaries and lazy locale bundles |
| `src/styles` | Design tokens and page/component CSS |
| `src-tauri/src/commands.rs` | Typed IPC boundary and input validation |
| `src-tauri/src/repository.rs` | Application service and in-memory state |
| `src-tauri/src/detection.rs` | Detector traits and native Windows adapters |
| `src-tauri/src/storage.rs` | Encrypted state and cache persistence |
| `src-tauri/src/security.rs` | DPAPI sealing for files at rest |
| `shared/fixtures` | Fixtures shared by Rust and TypeScript tests |

Keep the boundary intact: React must not call PowerShell directly, and the Rust
commands must not know about React components.

## Checks

Run these before opening a pull request:

```powershell
npm run typecheck
npm test
npm run build

cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Formatting:

```powershell
npm run format
npm run format:check
```

`npm run format` applies Prettier to TypeScript, TSX, CSS, HTML and the root
configs, then runs `cargo fmt`.

## Code style

- **Indentation:** four spaces, no tabs. This matches the maintainer's Java
  projects, including BirthdayBot.
- **TypeScript:** strict mode. Avoid `any`; prefer discriminated unions and
  `unknown` plus narrowing when data crosses the IPC boundary.
- **Comments:** JSDoc/Javadoc block comments for declarations and non-obvious
  branches. In TypeScript use `/** ... */` with `@param` and `@returns` where
  they add information. In Rust use `///` and `//!`, the native Javadoc
  equivalents; short inline notes stay `//`.
- **Rust:** run `cargo fmt`; keep error messages lowercase and actionable.
- **CSS:** design tokens from `src/styles/tokens.css`; no hard-coded colors in
  page CSS unless the value is a one-off asset color.
- **No browser behaviour:** no context menu, text selection, browser shortcuts,
  devtools, drag-and-drop or permanent scrollbar in the desktop shell.

## Tests

- Unit tests live next to the code (`*.test.ts`, Rust `#[cfg(test)]`).
- New behavior should have a regression test where practical.
- UI-only changes should still be checked in both themes and at the minimum
  window size (1024x720).
- For changes to Windows detection, include the Windows version and the command
  used to verify the result.

## Translations

1. Copy a JSON file from `src/i18n/locales/`.
2. Keep the key structure and translate the values.
3. Do not translate product names, file extensions or placeholder names.
4. Add the locale to `LOCALE_ORDER` in `src/i18n/index.ts` if it is new.
5. Missing keys fall back to English, then Russian.

Russian and English are the reference dictionaries. Changes to a key should be
made in both reference files first.

## Commit messages

Use Conventional Commits:

```text
feat(ui): add the components page
fix(core): detect software installed outside C:\
docs: rewrite the README with app screenshots
chore(style): apply four-space formatting
```

Keep the subject in the imperative mood, under ~72 characters, and explain the
why in the body when the change is not obvious. Avoid committing generated
artifacts (`dist`, `node_modules`, `src-tauri/target`).

## Pull requests

A pull request should contain:

- a short description of the user-visible change;
- the commands used to verify it;
- screenshots for visual changes in both light and dark themes;
- a linked issue when one exists;
- no unrelated formatting churn.

The PR template repeats this checklist.

## Documentation

- Keep the README screenshots in `docs/screenshots/` and regenerate them from
  the real app when the UI changes.
- Do not claim a feature is complete while its backend is still simulated; mark
  the limitation explicitly.
- Use relative links so the repository renders correctly on GitHub and in a
  local markdown viewer.

## Questions

Open a discussion or a draft pull request with the context and the exact command
that failed. For security reports, use the private channel described in
[SECURITY.md](SECURITY.md).
