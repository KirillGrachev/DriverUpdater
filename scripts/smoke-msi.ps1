param(
    [Parameter(Mandatory = $true)]
    [string] $BundleDir
)

$ErrorActionPreference = 'Stop'
$msi = Get-ChildItem -Path $BundleDir -Filter '*.msi' -File | Select-Object -First 1
if (-not $msi) {
    throw "No MSI found in $BundleDir"
}

$target = Join-Path $env:TEMP ("driver-updater-msi-smoke-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $target | Out-Null
try {
    $process = Start-Process msiexec.exe -ArgumentList @(
        '/a',
        "`"$($msi.FullName)`"",
        '/qn',
        "TARGETDIR=`"$target`""
    ) -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "Administrative install failed with exit code $($process.ExitCode)"
    }
    $executable = Get-ChildItem -Path $target -Recurse -Filter 'driver-updater.exe' -File |
        Select-Object -First 1
    if (-not $executable) {
        throw "The MSI payload does not contain driver-updater.exe"
    }
    if ($executable.Length -lt 1MB) {
        throw "The MSI payload executable is unexpectedly small"
    }
    $version = (Get-Content package.json | ConvertFrom-Json).version
    if ($executable.VersionInfo.ProductVersion -notlike "$version*") {
        throw "Payload version $($executable.VersionInfo.ProductVersion) does not match $version"
    }
    Write-Output "MSI smoke test passed: $($msi.Name) -> $($executable.Name) $version"
} finally {
    Remove-Item -LiteralPath $target -Recurse -Force -ErrorAction SilentlyContinue
}
