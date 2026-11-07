param(
    [string]$Version = "0.1.0",
    [string]$ReleaseNotes = "False custom update package with GUI installer and uninstaller",
    [switch]$SkipUIBuild
)

$ErrorActionPreference = "Stop"

if (-not $SkipUIBuild) {
    Write-Host "=== [1/3] Building false-ui ===" -ForegroundColor Cyan
    cargo build -p false-ui
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to build false-ui"
        exit 1
    }
}

Write-Host "=== [2/3] Preparing GUI Installer (FalseSetup.exe) ===" -ForegroundColor Cyan
# FalseSetup.exe is the standalone Tauri GUI installer embedding installer.html & uninstaller.html
$uiExe = "d:\false\target\debug\false-ui.exe"
$setupDest = "d:\false\FalseSetup.exe"
if (Test-Path $uiExe) {
    Copy-Item $uiExe $setupDest -Force
    Write-Host "Updated FalseSetup.exe (GUI Installer) at root." -ForegroundColor Green
}

Write-Host "=== [3/3] Packaging false-v$Version.flse ===" -ForegroundColor Cyan
$stage = Join-Path $env:TEMP "flse_build_stage"
if (Test-Path $stage) {
    Remove-Item $stage -Recurse -Force
}
New-Item -ItemType Directory -Path $stage | Out-Null

$manifest = @{
    name = "False Browser"
    version = $Version
    author = "zaidi"
    description = $ReleaseNotes
    created_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    has_binary = $true
    has_ui = $true
} | ConvertTo-Json -Depth 5

Set-Content -Path (Join-Path $stage "manifest.json") -Value $manifest -Encoding UTF8

$uiDest = Join-Path $stage "ui"
New-Item -ItemType Directory -Path $uiDest | Out-Null
Copy-Item "d:\false\crates\false-ui\ui\*" -Destination $uiDest -Recurse -Force

$iconsDest = Join-Path $stage "icons"
New-Item -ItemType Directory -Path $iconsDest | Out-Null
Copy-Item "d:\false\crates\false-ui\icons\*" -Destination $iconsDest -Recurse -Force
Copy-Item "d:\false\target\debug\false-ui.exe" -Destination (Join-Path $stage "false.exe") -Force

$tempZip = "d:\false\false-temp.zip"
$flseFile = "d:\false\false-v$Version.flse"

if (Test-Path $tempZip) { Remove-Item $tempZip -Force }
if (Test-Path $flseFile) { Remove-Item $flseFile -Force }

Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $tempZip -Force
Move-Item $tempZip $flseFile -Force
Remove-Item $stage -Recurse -Force

Write-Host "=== All Artifacts Built & Ready for Distribution ===" -ForegroundColor Green
Get-Item $setupDest, $flseFile | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
