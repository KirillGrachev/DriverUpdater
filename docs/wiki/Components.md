# Components

## What can be installed

The Components page exposes package families with selectable versions:

- Visual Studio Community, Professional and Enterprise 2022;
- .NET Framework 3.5 (Windows optional feature) and the 4.x Developer Pack;
- .NET Desktop Runtime 6, 8 and 9;
- DirectX End-User Runtime;
- Visual C++ Redistributable x64 and x86;
- Python 3.10, 3.11, 3.12 and 3.13;
- Node.js 20 and 22;
- OpenJDK 17, 21 and 25;
- Go, Rust, PHP 8.2/8.3/8.4, Git, CMake and 7-Zip.

## Versions

Opening a version selector runs:

```powershell
winget show --id <package> --exact --versions
```

The list is requested when the Components page first loads, with four bounded
workers, so the selectors do not wait for an individual dropdown. The bundled
list is a fallback while winget responds; it is not a claim that those versions
are still available.

## Installation

The selected version is installed with:

```powershell
winget install --id <package> --exact --version <version> --silent `
  --accept-package-agreements --accept-source-agreements --disable-interactivity
```

Winget may show a UAC prompt. Installing Visual Studio or a .NET runtime can take
several minutes. The app shows an installing state and reports the winget error
in a toast when installation fails.

## OpenGL

OpenGL is an API implemented by the graphics driver, not a separate Windows
package. The page explains that it is driver-managed and does not offer a fake
installer. Update the GPU driver through the vendor or through the Drivers page.

## Security boundary

Only package ids in the Rust allowlist are accepted. Versions are restricted to
letters, digits, `.`, `-`, `_` and `+`. The webview cannot ask winget to install
an arbitrary package.
