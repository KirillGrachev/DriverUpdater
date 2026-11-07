param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$arch = switch ($Target) {
    "x86_64-pc-windows-msvc" { "x64" }
    "aarch64-pc-windows-msvc" { "arm64" }
    "i686-pc-windows-msvc" { "x86" }
    default { throw "Unsupported Windows target: $Target" }
}

$version = $Version.TrimStart("v")
$root = Split-Path $PSScriptRoot -Parent
$release = Join-Path $root "src-tauri\target\$Target\release"
$output = Join-Path $root "artifacts\windows-$arch"
$portable = Join-Path $output "portable"
$symbols = Join-Path $output "symbols"

if (-not (Test-Path $release)) {
    throw "Release output is missing: $release"
}

if (Test-Path $output) {
    Remove-Item $output -Recurse -Force
}
New-Item -ItemType Directory -Path $portable, $symbols -Force | Out-Null

$exe = Join-Path $release "driver-updater.exe"
if (-not (Test-Path $exe)) {
    throw "Application binary is missing: $exe"
}
Copy-Item $exe $portable

$portableProject = Join-Path $root 'portable'
$launcher = Join-Path $portableProject "target/$Target/release/DriverUpdaterPortable.exe"
$cargoOut = Join-Path $env:TEMP "driver-updater-portable-$arch.out.log"
$cargoErr = Join-Path $env:TEMP "driver-updater-portable-$arch.err.log"
$cargoProcess = Start-Process cargo -ArgumentList @(
    'build',
    '--release',
    '--manifest-path',
    (Join-Path $portableProject 'Cargo.toml'),
    '--target',
    $Target
) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $cargoOut -RedirectStandardError $cargoErr
Remove-Item -LiteralPath $cargoOut, $cargoErr -Force -ErrorAction SilentlyContinue
if ($cargoProcess.ExitCode -ne 0 -or -not (Test-Path $launcher)) {
    throw "Portable launcher was not built: $launcher"
}
Copy-Item $launcher (Join-Path $portable 'DriverUpdaterPortable.exe')

Get-ChildItem $release -Filter "*.pdb" -File | ForEach-Object {
    Copy-Item $_.FullName $symbols
}

$portableNote = @"
Driver Updater portable build ($arch)

This build needs Microsoft Edge WebView2 Runtime. It is already present on
current Windows 10 and Windows 11 systems; otherwise install it from
https://developer.microsoft.com/microsoft-edge/webview2/.

Run DriverUpdaterPortable.exe. It asks for administrator rights once per
session, then starts Driver Updater elevated so driver installation and restore
points work without a service installation or a per-driver UAC prompt.

The application is portable: settings and component state stay in the user's
AppData directory. Delete the folder to remove the app; no installer or
uninstaller is involved.
"@
Set-Content -Path (Join-Path $portable "PORTABLE.txt") -Value $portableNote -Encoding UTF8

$portableZip = Join-Path $output "Driver.Updater_${version}_windows-${arch}_portable.zip"
$symbolsZip = Join-Path $output "Driver.Updater_${version}_windows-${arch}_symbols.zip"
Compress-Archive -Path (Join-Path $portable "*") -DestinationPath $portableZip -CompressionLevel Optimal
Compress-Archive -Path (Join-Path $symbols "*") -DestinationPath $symbolsZip -CompressionLevel Optimal
Remove-Item $portable, $symbols -Recurse -Force

$bundleRoots = @(
    (Join-Path $release "bundle\nsis"),
    (Join-Path $release "bundle\msi")
)
foreach ($bundleRoot in $bundleRoots) {
    if (-not (Test-Path $bundleRoot)) { continue }
    Get-ChildItem $bundleRoot -File | Where-Object { $_.Extension -in ".exe", ".msi" } | ForEach-Object {
        Copy-Item $_.FullName $output
    }
}

$checksumFile = Join-Path $output "SHA256SUMS-windows-$arch.txt"
Get-ChildItem $output -File | Where-Object { $_.Name -notlike "SHA256SUMS-*" } | Sort-Object Name | ForEach-Object {
    $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $($_.Name)"
} | Set-Content $checksumFile -Encoding ASCII

Get-ChildItem $output -File | Select-Object Name, Length
