# Driver Updater Wiki

Driver Updater is a native Windows driver manager built with Tauri 2, Rust and
React. It inventories installed drivers and software, keeps restore points,
installs selected component versions through winget and ships in 35 languages.

## Start here

- [[Getting Started]] - install the app and run the first scan.
- [[Architecture]] - how the React shell, Rust IPC and encrypted storage fit together.
- [[Components]] - versioned runtimes, Visual Studio, .NET and winget behaviour.
- [[Security and Privacy]] - DPAPI storage, threat model and reporting.
- [[Development]] - local setup, formatting, tests and releases.
- [[FAQ]] - short answers to common questions.
- [[Troubleshooting]] - fixes for winget, scanning and startup issues.

## Current status

Driver and software inventory is read from the real Windows system. Driver
update installation still uses the simulated repository layer. Component
installation is real and goes through Windows Package Manager (winget).

The latest installer is available on the
[Releases](https://github.com/KirillGrachev/DriverUpdater/releases) page.

## Links

- [Source code](https://github.com/KirillGrachev/DriverUpdater)
- [Issue tracker](https://github.com/KirillGrachev/DriverUpdater/issues)
- [Contributing guide](https://github.com/KirillGrachev/DriverUpdater/blob/main/CONTRIBUTING.md)
- [Security policy](https://github.com/KirillGrachev/DriverUpdater/blob/main/SECURITY.md)
