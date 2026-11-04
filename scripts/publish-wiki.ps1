# Publishes the Markdown pages in wiki/ to the GitHub wiki repository.
#
# GitHub does not expose a REST API for creating the first wiki page. Open the
# repository wiki once, create a placeholder Home page, then run this script.
#
# Usage:
#   pwsh -File scripts/publish-wiki.ps1
#   pwsh -File scripts/publish-wiki.ps1 -Message "docs: update wiki"

param(
    [string]$Message = "docs: sync wiki pages"
)

$ErrorActionPreference = "Stop"
$repo = "https://github.com/KirillGrachev/DriverUpdater.wiki.git"
$source = Join-Path $PSScriptRoot "..\docs\wiki"
$temp = Join-Path $env:TEMP ("driver-updater-wiki-" + [guid]::NewGuid().ToString("N"))

if (-not (Test-Path $source)) {
    throw "wiki source directory is missing: $source"
}

git clone $repo $temp
if ($LASTEXITCODE -ne 0) {
    throw "The wiki repository is not initialized. Open https://github.com/KirillGrachev/DriverUpdater/wiki and create the first page, then rerun this script."
}

Copy-Item (Join-Path $source "*.md") $temp -Force
Push-Location $temp
try {
    git add .
    if (git diff --cached --quiet) {
        Write-Output "wiki is already up to date"
        return
    }
    git -c user.name="KirillGrachev" -c user.email="NeyComiunity@gmail.com" commit -m $Message
    git push origin HEAD
} finally {
    Pop-Location
    Remove-Item $temp -Recurse -Force
}
