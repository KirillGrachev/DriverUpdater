param(
    [Parameter(Mandatory = $true)]
    [string] $BundleDir,
    [switch] $Force
)

$ErrorActionPreference = 'Stop'
if (-not $Force -and $env:CI -ne 'true') {
    throw 'This script installs and uninstalls the app. Run it only on a disposable CI runner or pass -Force.'
}

$setup = Get-ChildItem -Path $BundleDir -Filter '*.exe' -File | Select-Object -First 1
if (-not $setup) {
    throw "No NSIS installer found in $BundleDir"
}

$target = Join-Path $env:TEMP ("driver-updater-nsis-smoke-" + [guid]::NewGuid().ToString('N'))
try {
    $install = Start-Process $setup.FullName -ArgumentList @('/S', "/D=$target") -Wait -PassThru
    if ($install.ExitCode -ne 0) {
        throw "Silent install failed with exit code $($install.ExitCode)"
    }
    $executable = Join-Path $target 'driver-updater.exe'
    if (-not (Test-Path $executable)) {
        throw 'The NSIS payload does not contain driver-updater.exe'
    }
    $version = (Get-Content package.json | ConvertFrom-Json).version
    if ((Get-Item $executable).VersionInfo.ProductVersion -notlike "$version*") {
        throw "Payload version $((Get-Item $executable).VersionInfo.ProductVersion) does not match $version"
    }
    $uninstaller = Join-Path $target 'uninstall.exe'
    if (-not (Test-Path $uninstaller)) {
        throw 'The NSIS payload does not contain uninstall.exe'
    }
    $uninstall = Start-Process $uninstaller -ArgumentList @('/S') -Wait -PassThru
    if ($uninstall.ExitCode -ne 0) {
        throw "Silent uninstall failed with exit code $($uninstall.ExitCode)"
    }
    if (Test-Path $target) {
        throw 'The silent uninstall left the installation directory behind'
    }
    Write-Output "NSIS smoke test passed: $($setup.Name) -> $version"
} finally {
    Remove-Item -LiteralPath $target -Recurse -Force -ErrorAction SilentlyContinue
}
