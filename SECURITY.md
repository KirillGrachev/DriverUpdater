# Security Policy

Driver Updater reads hardware and software inventory from Windows, runs
PowerShell/CIM queries and can open folders and links. Treat device names,
registry values, file paths and IPC payloads as untrusted input.

## Supported versions

| Version | Supported |
|---|---|
| `main` (development) | Yes |
| Latest release | Yes |
| Older releases | No |

Security fixes land on `main` and in the next release. If you are unsure whether
a version is affected, include the commit or release tag in the report.

## Reporting a vulnerability

Do not open a public issue containing a working exploit, secrets, personal
paths or a malicious attachment.

Use GitHub's private vulnerability reporting on the repository:

https://github.com/KirillGrachev/DriverUpdater/security/advisories/new

If private reporting is unavailable, contact the maintainer through the GitHub
profile linked in the repository and ask for a private transfer method before
sharing sensitive details.

Please include:

- affected version, commit or installer;
- Windows version and architecture;
- the component involved: UI, Tauri command, driver scan, software detection,
  settings persistence, URL opener or installer;
- impact and required attacker access;
- exact reproduction steps;
- expected and actual behavior;
- logs with credentials, usernames and personal paths removed;
- whether the issue reproduces from a clean checkout.

## Scope

Reports are welcome for:

- command injection or argument injection through driver, software or path data;
- path traversal or symlink handling in backup and restore operations;
- unsafe deserialization of persisted settings or IPC payloads;
- bypass of the external URL allowlist or the browser opener;
- privilege boundary issues in the installer, WebView2 bootstrap or uninstaller;
- WebView2 or Tauri capability/CSP misconfiguration;
- denial of service through oversized registry values or scan output;
- dependency or release artifact integrity.

General bugs without a security impact belong in the normal issue tracker.

## Current hardening

- External links are restricted to `http(s)` and routed through the opener
  plugin; the Rust command validates the scheme again.
- Settings received over IPC are validated against the allowed theme, interval
  and restore-point period values.
- Persisted settings, driver inventory, restore points and component state are
  sealed with Windows DPAPI (current-user scope) and application entropy.
- The webview has devtools, drag-and-drop, zoom hotkeys and browser shortcuts
  disabled, and a restrictive CSP.
- The installer embeds the WebView2 offline installer and links the MSVC runtime
  statically, so a failed prerequisite install does not leave a broken app.

## At-rest data limits

DPAPI protects files copied to another account, another machine, or an offline
disk image. It does not protect against code already executing as the same
logged-in Windows user, because that user is allowed to call DPAPI. The app does
not claim protection against a compromised administrator session.

The browser preview stores fixture state in `localStorage`; the desktop build
does not use that path for settings, driver data, restore points or components.

## Disclosure

Please give the maintainers reasonable time to investigate and ship a fix
before publishing details. Credit is given in the release notes when requested.
