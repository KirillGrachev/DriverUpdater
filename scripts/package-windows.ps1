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

$webview2Dir = Join-Path $portable 'webview2'
$webview2Name = switch ($arch) {
    'x64' { 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe' }
    'x86' { 'MicrosoftEdgeWebView2RuntimeInstallerX86.exe' }
    'arm64' { 'MicrosoftEdgeWebview2Setup.exe' }
    default { throw "Unsupported WebView2 architecture: $arch" }
}
$webview2Url = switch ($arch) {
    'x64' { 'https://go.microsoft.com/fwlink/?linkid=2124701' }
    'x86' { 'https://go.microsoft.com/fwlink/?linkid=2099617' }
    'arm64' { 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' }
    default { throw "Unsupported WebView2 architecture: $arch" }
}
$webview2Installer = Join-Path $root "installer/webview2/$webview2Name"
$webview2MinimumBytes = if ($arch -eq 'arm64') { 1MB } else { 100MB }
if (-not (Test-Path $webview2Installer) -or (Get-Item $webview2Installer).Length -lt $webview2MinimumBytes) {
    New-Item -ItemType Directory -Path (Split-Path $webview2Installer) -Force | Out-Null
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if (-not $curl) { throw 'curl.exe is required to download the WebView2 installer' }
    & $curl.Source --location --fail --retry 3 --output $webview2Installer $webview2Url
    if ($LASTEXITCODE -ne 0) { throw "WebView2 Runtime download failed with exit code $LASTEXITCODE" }
}
New-Item -ItemType Directory -Path $webview2Dir -Force | Out-Null
Copy-Item $webview2Installer (Join-Path $webview2Dir $webview2Name) -Force

$portableProject = Join-Path $root 'portable'
$launcher = Join-Path $portableProject "target/$Target/release/DriverUpdaterPortable.exe"
$cargoOut = Join-Path $env:TEMP "driver-updater-portable-$arch.out.log"
$cargoErr = Join-Path $env:TEMP "driver-updater-portable-$arch.err.log"
$previousRustFlags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = "$previousRustFlags -C target-feature=+crt-static".Trim()
    $cargoProcess = Start-Process cargo -ArgumentList @(
        'build',
        '--release',
        '--manifest-path',
        (Join-Path $portableProject 'Cargo.toml'),
        '--target',
        $Target
    ) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $cargoOut -RedirectStandardError $cargoErr
} finally {
    $env:RUSTFLAGS = $previousRustFlags
}
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

The launcher checks Microsoft Edge WebView2 Runtime and installs the bundled
offline runtime when it is missing, so this build works on clean Windows 10
LTSC as well as normal Windows 10/11 systems.

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
