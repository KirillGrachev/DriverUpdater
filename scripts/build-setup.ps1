param(
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$Version = '',
    [string]$BundleRoot = ''
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path -Parent $PSScriptRoot
if (-not $Version) {
    $Version = (Get-Content (Join-Path $root 'package.json') | ConvertFrom-Json).version
}
$arch = switch ($Target) {
    'x86_64-pc-windows-msvc' { 'x64' }
    'aarch64-pc-windows-msvc' { 'arm64' }
    'i686-pc-windows-msvc' { 'x86' }
    default { throw "Unsupported target: $Target" }
}
if (-not $BundleRoot) {
    $BundleRoot = Join-Path $root "src-tauri/target/$Target/release/bundle"
}
$nsisCandidates = @(Get-ChildItem $BundleRoot -Filter '*-setup.exe' -File -Recurse -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending)
$msiCandidates = @(Get-ChildItem $BundleRoot -Filter '*.msi' -File -Recurse -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending)
$cleanVersion = $Version.TrimStart('v')
$nsis = @($nsisCandidates | Where-Object { $_.Name -like "*$cleanVersion*" } | Select-Object -First 1)
$msi = @($msiCandidates | Where-Object { $_.Name -like "*$cleanVersion*" } | Select-Object -First 1)
if ($nsis.Count -eq 0 -and $msi.Count -eq 0) {
    $nsis = @($nsisCandidates | Select-Object -First 1)
    if ($nsis.Count -eq 0) {
        $msi = @($msiCandidates | Select-Object -First 1)
    }
}
if ($nsis.Count -eq 0 -and $msi.Count -eq 0) {
    throw "No MSI or NSIS payload found under $BundleRoot"
}

$setupProject = Join-Path $root 'installer/src-tauri'
$buildId = [guid]::NewGuid().ToString('N')
$stdoutLog = Join-Path $env:TEMP "driver-updater-setup-$buildId.out.log"
$stderrLog = Join-Path $env:TEMP "driver-updater-setup-$buildId.err.log"
try {
    $process = Start-Process cargo -ArgumentList @(
        'build',
        '--release',
        '--manifest-path',
        (Join-Path $setupProject 'Cargo.toml'),
        '--target',
        $Target
    ) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog
    if ($process.ExitCode -ne 0) {
        Get-Content $stdoutLog, $stderrLog -Tail 80 -ErrorAction SilentlyContinue
        throw "The setup wizard build failed with exit code $($process.ExitCode)"
    }
} finally {
    Remove-Item -LiteralPath $stdoutLog, $stderrLog -Force -ErrorAction SilentlyContinue
}
$setupExe = Join-Path $setupProject "target/$Target/release/driver-updater-setup.exe"
if (-not (Test-Path $setupExe)) {
    throw "Setup executable was not produced: $setupExe"
}

$output = Join-Path $root "artifacts/setup/$arch"
if (Test-Path $output) {
    Remove-Item -LiteralPath $output -Recurse -Force
}
$payloadDir = Join-Path $output 'payload'
New-Item -ItemType Directory -Path $payloadDir -Force | Out-Null
Copy-Item $setupExe (Join-Path $output 'DriverUpdaterSetup.exe') -Force
$payloads = @()
if ($msi.Count -gt 0) {
    $payloads += $msi[0]
}
if ($nsis.Count -gt 0) {
    $payloads += $nsis[0]
}
foreach ($payload in $payloads) {
    Copy-Item $payload.FullName (Join-Path $payloadDir $payload.Name) -Force
}

$archive = Join-Path $root "artifacts/setup/DriverUpdaterSetup-$Version-$arch.zip"
if (Test-Path $archive) {
    Remove-Item -LiteralPath $archive -Force
}
Compress-Archive -Path (Join-Path $output '*') -DestinationPath $archive -CompressionLevel Optimal
"Setup wizard: $output"
"Archive: $archive"
