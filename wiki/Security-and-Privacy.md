# Security and Privacy

## Local data protection

Settings, driver inventory, restore points and component ids are stored under
the current user's app data directory and sealed with Windows DPAPI
(current-user scope) plus application-specific entropy.

The driver cache is sealed the same way.

Encrypted files:

- `mock-state.json`
- `installed-drivers.json`

A copied profile, another Windows account or an offline disk image cannot
decrypt these files with the app alone. Legacy plaintext files are migrated once
and rewritten in encrypted form.

## Threat model

DPAPI raises the cost of offline data theft. It does **not** protect against
malware running as the same logged-in Windows user: that user can ask DPAPI to
decrypt the files. Use BitLocker, a clean machine and OS-level account security
as part of the overall model.

## Network

- Driver and software inventory is local.
- Component installation downloads packages from configured winget sources.
- Anonymous telemetry is off by default and currently only records a local
  preference.
- External links open in the system browser.

## Webview hardening

- devtools, drag-and-drop, zoom hotkeys and browser shortcuts are disabled;
- context menu and text selection are suppressed outside editable fields;
- a restrictive Content Security Policy is applied;
- external URLs are restricted to http(s) and opened through the opener plugin.

## Reporting a vulnerability

Do not open a public issue with an exploit or personal data. Use private
vulnerability reporting on the repository:

https://github.com/KirillGrachev/DriverUpdater/security/advisories/new

See [SECURITY.md](https://github.com/KirillGrachev/DriverUpdater/blob/main/SECURITY.md)
for the full scope and response process.
