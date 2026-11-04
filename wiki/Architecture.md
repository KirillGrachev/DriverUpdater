# Architecture

## Overview

```text
React + Vite webview
        |
        | typed invoke()
        v
Tauri commands (src-tauri/src/commands.rs)
        |
        | spawn_blocking
        v
DriverRepository service
   |          |             |
detection   encrypted     restore points
traits      storage
   |          |
PowerShell   DPAPI files
/ CIM
```

The webview never calls PowerShell or the filesystem directly. Every native
operation goes through a typed command and validates its input at the IPC
boundary.

## Rust modules

| Module | Responsibility |
|---|---|
| `commands.rs` | Tauri commands, input validation and blocking-pool dispatch |
| `repository.rs` | Application service: drivers, settings, restore points, component ids |
| `detection.rs` | `DriverDetector`, `ComponentDetector`, `SystemInfoProvider` and the native adapter |
| `storage.rs` | Encrypted state and driver-cache persistence |
| `security.rs` | Windows DPAPI seal/open |
| `packages.rs` | winget package allowlist, version lookup and installation |
| `process.rs` | Timed child processes with deadlock-safe stdout/stderr capture |
| `models.rs` | Serde models mirroring `src/ipc/types.ts` |

The repository receives detector traits through its constructor. Production uses
`NativeDetector`; tests can inject a fake without touching PowerShell.

## Frontend

- `src/App.tsx` shell, lazy routes, boot and error states.
- `src/store/app.tsx` routing, IPC calls, theme, locale, toasts, settings queue.
- `src/ipc` the `Backend` contract and the Tauri/mock implementations.
- `src/pages` one component per screen.
- `src/styles` design tokens and page CSS.

Secondary pages are lazy-loaded. Home is eager so the first screen paints
without a route chunk request.

## Async and blocking work

PowerShell/CIM calls and winget are blocking. Commands hand repository work to
`tauri::async_runtime::spawn_blocking`; the webview's async executor stays free
for other IPC calls. Driver and software scans have a 20-second timeout.
Component installation has a 45-minute timeout because Visual Studio and the
.NET runtimes are large.

## Persistence

The repository stores settings, drivers, restore points and component ids in
one document. The driver cache is a separate document. Both are encrypted before
they reach disk; see [[Security and Privacy]].
