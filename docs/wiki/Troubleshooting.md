# Troubleshooting

## The window is blank or does not appear

The native window is hidden until the first React commit. If the webview fails
before that point, restart the app. A boot error screen is shown when the backend
or a locale chunk fails after the shell has started.

## winget is not available

Install **App Installer** from the Microsoft Store and reopen the app. The
Components page reports a winget error in a toast; it does not mark the package
as installed.

## A component installation failed

- Check the network connection.
- Approve the UAC prompt if Windows shows one.
- Make sure the selected version still exists in winget.
- Try the same package once more; some installers require a retry after a
  previous partially completed install.

## Visual Studio takes a very long time

Visual Studio is several gigabytes. The app allows up to 45 minutes for a winget
install. Keep the machine awake and connected; the progress is managed by winget,
not by the app.

## The driver scan is slow

The first scan queries `Win32_PnPSignedDriver` and writes an encrypted cache.
Later launches read the cache. Use **Check for updates** to force a fresh scan.
The query is killed after 20 seconds and falls back to cached data.

## Settings do not survive a restart

Settings are written through a serialized queue to DPAPI-sealed storage. Check
that the app data directory is writable by the current Windows user. On startup,
legacy plaintext state is migrated once and rewritten.

## Enabling debug logs

The desktop build has devtools disabled by design. Reproduce the issue with a
clear description, the exact steps and the Windows version, then open a bug
report. For security issues use the private advisory link in
[Security and Privacy](https://github.com/KirillGrachev/DriverUpdater/wiki/Security-and-Privacy).
