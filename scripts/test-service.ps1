param(
    [string]$Configuration = 'debug'
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path -Parent $PSScriptRoot

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'The service smoke test must run with administrator rights.'
}

$serviceName = 'DriverUpdaterService'
$tokenDir = Join-Path $env:ProgramData 'Driver Updater'
$tokenPath = Join-Path $tokenDir 'service.token'
$serviceDir = Join-Path $env:ProgramFiles 'Driver Updater'
$servicePath = Join-Path $serviceDir 'driver-updater-service-test.exe'
$targetDir = Join-Path $root "src-tauri/target/$Configuration"
$built = Join-Path $targetDir 'driver-updater-service.exe'
$sc = Join-Path $env:SystemRoot 'System32/sc.exe'

function Remove-TestService {
    [CmdletBinding(SupportsShouldProcess = $true)]
    param()

    if (-not $PSCmdlet.ShouldProcess($serviceName, 'Remove the test service')) {
        return
    }
    & $sc stop $serviceName 2>$null | Out-Null
    Start-Sleep -Milliseconds 800
    & $sc delete $serviceName 2>$null | Out-Null
    Remove-Item -LiteralPath $servicePath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $serviceDir -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tokenPath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tokenDir -Force -ErrorAction SilentlyContinue
}

Remove-TestService
try {
    if ($Configuration -eq 'release') {
        & cargo build --release --manifest-path (Join-Path $root 'src-tauri/Cargo.toml') --bin driver-updater-service
    } else {
        & cargo build --manifest-path (Join-Path $root 'src-tauri/Cargo.toml') --bin driver-updater-service
    }
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $built)) {
        throw 'The service binary was not built.'
    }
    New-Item -ItemType Directory -Path $tokenDir -Force | Out-Null
    New-Item -ItemType Directory -Path $serviceDir -Force | Out-Null
    $token = (1..4 | ForEach-Object { [guid]::NewGuid().ToString('N') }) -join ''
    Set-Content -LiteralPath $tokenPath -Value $token -NoNewline
    $sid = ([Security.Principal.WindowsIdentity]::GetCurrent()).User.Value
    & (Join-Path $env:SystemRoot 'System32/icacls.exe') $tokenPath /inheritance:r /grant:r "*$sid`:R" /grant:r '*S-1-5-18:F' /grant:r '*S-1-5-32-544:F' | Out-Null
    Copy-Item -LiteralPath $built -Destination $servicePath -Force

    $binPath = '"' + $servicePath + '"'
    & $sc create $serviceName binPath= $binPath start= auto obj= LocalSystem | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'sc.exe create failed.' }
    & $sc start $serviceName | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'sc.exe start failed.' }

    $running = $false
    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        $query = & $sc query $serviceName
        if ($query -match 'RUNNING') { $running = $true; break }
        Start-Sleep -Milliseconds 250
    }
    if (-not $running) { throw 'The service did not reach the RUNNING state.' }

    $pipe = [System.IO.Pipes.NamedPipeClientStream]::new(
        '.',
        'driver-updater-service',
        [System.IO.Pipes.PipeDirection]::InOut
    )
    try {
        $pipe.Connect(5000)
        $request = @{
            version = 1
            token = $token
            operation = @{ op = 'ping' }
        } | ConvertTo-Json -Compress -Depth 5
        $bytes = [Text.Encoding]::UTF8.GetBytes($request)
        $pipe.Write($bytes, 0, $bytes.Length)
        $pipe.Flush()
        $buffer = New-Object byte[] 4096
        $read = $pipe.Read($buffer, 0, $buffer.Length)
        $response = [Text.Encoding]::UTF8.GetString($buffer, 0, $read) | ConvertFrom-Json
        if (-not $response.ok -or $response.result.service -ne 'driver-updater') {
            throw 'The service returned an unexpected ping response.'
        }
    } finally {
        $pipe.Dispose()
    }
    Write-Output 'Service install, start, ping and stop passed.'
} finally {
    Remove-TestService
}
