param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

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

Get-ChildItem $release -Filter "*.pdb" -File | ForEach-Object {
    Copy-Item $_.FullName $symbols
}

$portableNote = @"
Driver Updater portable build ($arch)

This build needs Microsoft Edge WebView2 Runtime. It is already present on
current Windows 10 and Windows 11 systems; otherwise install it from
https://developer.microsoft.com/microsoft-edge/webview2/.

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
