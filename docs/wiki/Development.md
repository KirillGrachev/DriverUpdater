# Development

## Setup

```powershell
git clone https://github.com/KirillGrachev/DriverUpdater.git
cd DriverUpdater
npm install
npm run tauri dev
```

Browser-only UI development:

```powershell
npm run dev
```

## Style

- four spaces, no tabs;
- Prettier and `cargo fmt`;
- strict TypeScript, no `any` at IPC boundaries;
- Conventional Commits.

```powershell
npm run format
npm run format:check
npm run typecheck
```

## Tests

```powershell
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

CI runs Prettier, TypeScript, frontend tests, Vite build, `cargo fmt`,
`cargo check` and Rust tests on Linux and Windows.

## Translations

Locale JSON files live in `src/i18n/locales/`. Russian and English are the
reference dictionaries. Missing keys fall back to English, then Russian.

Add a locale to `LOCALE_ORDER` in `src/i18n/index.ts` when it is not listed.

## Release

```powershell
npm run tauri build -- --bundles nsis
```

The installer is written to `src-tauri/target/release/bundle/nsis/`.

Publish the installer on GitHub Releases. The project version lives in
`src-tauri/tauri.conf.json`, `Cargo.toml` and `package.json`; keep them in sync.
