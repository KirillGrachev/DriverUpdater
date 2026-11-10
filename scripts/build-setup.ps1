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

# The setup wizard is a WebView2 application. Windows 10 LTSC and some
# server images do not ship the Evergreen runtime, so keep Microsoft's
# offline installer next to the native bootstrapper.
$webview2Dir = Join-Path $root "installer/webview2"
$webview2Installer = Join-Path $webview2Dir 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
$webview2Url = 'https://go.microsoft.com/fwlink/?linkid=2124701'
New-Item -ItemType Directory -Path $webview2Dir -Force | Out-Null
$downloadWebView2 = -not (Test-Path $webview2Installer)
if (-not $downloadWebView2) {
    $downloadWebView2 = (Get-Item $webview2Installer).Length -lt 100MB
}
if ($downloadWebView2) {
    $temporary = "$webview2Installer.download"
    Remove-Item $temporary -Force -ErrorAction SilentlyContinue
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        & $curl.Source --location --fail --retry 3 --retry-delay 5 --output $temporary $webview2Url
        if ($LASTEXITCODE -ne 0) {
            Remove-Item $temporary -Force -ErrorAction SilentlyContinue
            throw "WebView2 Runtime download failed with curl exit code $LASTEXITCODE"
        }
    } else {
        Invoke-WebRequest -Uri $webview2Url -OutFile $temporary -UseBasicParsing
    }
    $downloaded = (Test-Path $temporary) -and (Get-Item $temporary).Length -gt 100MB
    if (-not $downloaded) {
        Remove-Item $temporary -Force -ErrorAction SilentlyContinue
        throw 'WebView2 Runtime download is incomplete'
    }
    Move-Item $temporary $webview2Installer -Force
}
if (-not (Test-Path $webview2Installer)) {
    throw "WebView2 offline installer is missing: $webview2Installer"
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

$payload = if ($nsis.Count -gt 0) { $nsis[0] } else { $msi[0] }
$payloadName = "payload$($payload.Extension.ToLowerInvariant())"
$payloadHash = (Get-FileHash -LiteralPath $payload.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$previousPayloadHash = $env:DRIVER_UPDATER_PAYLOAD_SHA256
$env:DRIVER_UPDATER_PAYLOAD_SHA256 = $payloadHash

$serviceExe = Join-Path $root "src-tauri/target/$Target/release/driver-updater-service.exe"
$serviceOutLog = Join-Path $env:TEMP "driver-updater-service-build.out.log"
$serviceErrLog = Join-Path $env:TEMP "driver-updater-service-build.err.log"
$serviceProcess = Start-Process cargo -ArgumentList @(
    'build',
    '--release',
    '--manifest-path',
    (Join-Path $root 'src-tauri/Cargo.toml'),
    '--target',
    $Target,
    '--bin',
    'driver-updater-service'
) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $serviceOutLog -RedirectStandardError $serviceErrLog
if ($serviceProcess.ExitCode -ne 0) {
    Get-Content $serviceOutLog, $serviceErrLog -Tail 80 -ErrorAction SilentlyContinue
    throw "The Windows service build failed with exit code $($serviceProcess.ExitCode)"
}
Remove-Item -LiteralPath $serviceOutLog, $serviceErrLog -Force -ErrorAction SilentlyContinue
if (-not (Test-Path $serviceExe)) {
    throw "Service executable was not produced: $serviceExe"
}
$previousServiceBin = $env:DRIVER_UPDATER_SERVICE_BIN
$env:DRIVER_UPDATER_SERVICE_BIN = $serviceExe

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
    if ($null -eq $previousPayloadHash) {
        Remove-Item Env:DRIVER_UPDATER_PAYLOAD_SHA256 -ErrorAction SilentlyContinue
    } else {
        $env:DRIVER_UPDATER_PAYLOAD_SHA256 = $previousPayloadHash
    }
    if ($null -eq $previousServiceBin) {
        Remove-Item Env:DRIVER_UPDATER_SERVICE_BIN -ErrorAction SilentlyContinue
    } else {
        $env:DRIVER_UPDATER_SERVICE_BIN = $previousServiceBin
    }
    Remove-Item -LiteralPath $stdoutLog, $stderrLog -Force -ErrorAction SilentlyContinue
}
$setupExe = Join-Path $setupProject "target/$Target/release/driver-updater-setup.exe"
if (-not (Test-Path $setupExe)) {
    throw "Setup executable was not produced: $setupExe"
}

$bootstrapperProject = Join-Path $root 'installer/bootstrapper'
$bootstrapperExe = Join-Path $bootstrapperProject "target/$Target/release/driver-updater-bootstrapper.exe"
$bootstrapperOutLog = Join-Path $env:TEMP 'driver-updater-bootstrapper-build.out.log'
$bootstrapperErrLog = Join-Path $env:TEMP 'driver-updater-bootstrapper-build.err.log'
$previousRustFlags = $env:RUSTFLAGS
try {
    # The native bootstrapper must run before any redistributable is present.
    $env:RUSTFLAGS = "$previousRustFlags -C target-feature=+crt-static".Trim()
    $bootstrapperProcess = Start-Process cargo -ArgumentList @(
        'build',
        '--release',
        '--manifest-path',
        (Join-Path $bootstrapperProject 'Cargo.toml'),
        '--target',
        $Target
    ) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $bootstrapperOutLog -RedirectStandardError $bootstrapperErrLog
    if ($bootstrapperProcess.ExitCode -ne 0) {
        Get-Content $bootstrapperOutLog, $bootstrapperErrLog -Tail 80 -ErrorAction SilentlyContinue
        throw "The setup bootstrapper build failed with exit code $($bootstrapperProcess.ExitCode)"
    }
} finally {
    $env:RUSTFLAGS = $previousRustFlags
    Remove-Item -LiteralPath $bootstrapperOutLog, $bootstrapperErrLog -Force -ErrorAction SilentlyContinue
}
if (-not (Test-Path $bootstrapperExe)) {
    throw "Setup bootstrapper was not produced: $bootstrapperExe"
}

$output = Join-Path $root "artifacts/setup/$arch"
if (Test-Path $output) {
    Remove-Item -LiteralPath $output -Recurse -Force
}
$payloadDir = Join-Path $output 'payload'
$webview2Output = Join-Path $output 'webview2'
New-Item -ItemType Directory -Path $payloadDir, $webview2Output -Force | Out-Null
Copy-Item $bootstrapperExe (Join-Path $output 'DriverUpdaterSetup.exe') -Force
Copy-Item $setupExe (Join-Path $output 'driver-updater-setup-ui.exe') -Force
Copy-Item $webview2Installer (Join-Path $webview2Output 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe') -Force
if ($nsis.Count -eq 0 -and $msi.Count -eq 0) {
    throw "No installer payload was available for $arch"
}
Copy-Item $payload.FullName (Join-Path $payloadDir $payloadName) -Force

$archive = Join-Path $root "artifacts/setup/DriverUpdaterSetup-$Version-$arch.zip"
if (Test-Path $archive) {
    Remove-Item -LiteralPath $archive -Force
}
$sevenZip = Join-Path $env:ProgramFiles '7-Zip\7z.exe'
if (Test-Path $sevenZip) {
    & $sevenZip a -tzip -mx=5 $archive (Join-Path $output '*') | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "7-Zip failed to create $archive" }
} else {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory(
        $output,
        $archive,
        [System.IO.Compression.CompressionLevel]::Optimal,
        $true
    )
}
"Setup wizard: $output"
"Archive: $archive"
