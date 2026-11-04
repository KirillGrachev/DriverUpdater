# FAQ

## Are driver updates real?

Driver and software inventory is real. Driver update installation is still a
simulated repository flow. Component installation through winget is real.

## Why does a driver show "No data" for its size?

`Win32_PnPSignedDriver` does not expose the installed package size. Showing
`0 MB` would be misleading, so the UI reports that the size is unknown.

## Why is the installer about 210 MB?

It embeds the full offline WebView2 runtime. This lets a clean Windows machine
install without downloading WebView2 separately.

## Does the app need administrator rights?

The app runs per-user. Winget may request UAC for machine-wide packages such as
Visual Studio or the .NET runtimes. The installer may also request elevation
depending on the selected install scope.

## Where is data stored?

Under the current user's app data directory. Settings, driver data, restore
points and component ids are DPAPI-sealed. See [Security and Privacy](https://github.com/KirillGrachev/DriverUpdater/wiki/Security-and-Privacy).

## How do I change the language?

Settings -> Interface language. The choice is saved immediately. The app ships
35 locales and falls back to English for missing keys.

## My driver scan is empty or takes too long

The scan uses PowerShell/CIM and has a 20-second timeout. Make sure PowerShell
is available, then run a new scan from Home or Drivers. A failed scan falls back
to the cached/fixture data instead of blocking the UI.

## Can I install a version that winget does not list?

No. The version selector is populated from winget, and installation validates
the version string. Versions removed from the winget source are not installable
through the app.

## Does OpenGL have a separate installer?

No. OpenGL support is provided by the GPU driver. Update the graphics driver
instead.
