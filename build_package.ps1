param(
    [string]$Version = "",
    [string]$ReleaseNotes = "False browser stable update: fixed startup crash, signed packages, hardened paths",
    [switch]$SkipUIBuild,
    [switch]$SkipChecks,
    [switch]$Bundle
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
$UiDir = Join-Path $Root "crates\false-ui\ui"
$RustSource = Join-Path $Root "crates\false-ui\src\lib.rs"

# crates/false-ui/Cargo.toml is the single version source; propagate it before
# the checker and Tauri build read their metadata.
node (Join-Path $Root "scripts\sync-version.mjs")
if ($LASTEXITCODE -ne 0) { throw "Version sync failed" }

if (-not $Version) {
    $cargoToml = Get-Content (Join-Path $Root "crates\false-ui\Cargo.toml") -Raw
    $m = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $m.Success) { throw "Could not read the version from crates/false-ui/Cargo.toml" }
    $Version = $m.Groups[1].Value
}

if (-not $SkipChecks) {
    Write-Host "=== [1/4] Frontend guardrail ===" -ForegroundColor Cyan
    node (Join-Path $Root "scripts\check-frontend.mjs")
    if ($LASTEXITCODE -ne 0) { throw "Frontend checks failed" }

    Write-Host "=== [2/4] Clippy ===" -ForegroundColor Cyan
    cargo clippy -p false-ui --all-targets --no-deps -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed" }
}

if (-not $SkipUIBuild) {
    Write-Host "=== [3/4] Building false-ui (release) ===" -ForegroundColor Cyan
    cargo build --release -p false-ui
    if ($LASTEXITCODE -ne 0) { throw "Failed to build false-ui" }
}

Write-Host "=== [4/4] Preparing GUI installer + packaging false-v$Version.flse ===" -ForegroundColor Cyan

$uiExe = Join-Path $Root "target\release\false-ui.exe"
if (-not (Test-Path $uiExe)) {
    throw "Missing release binary: $uiExe (run without -SkipUIBuild)"
}

# FalseSetup.exe is the same binary; it switches to installer mode from its name/args.
$setupDest = Join-Path $Root "FalseSetup.exe"
Copy-Item $uiExe $setupDest -Force
Write-Host "Updated FalseSetup.exe (GUI installer, release build)" -ForegroundColor Green

$stage = Join-Path $env:TEMP "flse_build_stage"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
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

$manifestPath = Join-Path $stage "manifest.json"
$manifestSig = Join-Path $stage "manifest.sig"
# Write without a BOM: the signature is over these exact bytes.
[System.IO.File]::WriteAllText($manifestPath, $manifest, (New-Object System.Text.UTF8Encoding($false)))

Write-Host "Signing manifest..." -ForegroundColor Cyan
node (Join-Path $Root "scripts\sign-manifest.mjs") $manifestPath $manifestSig
if ($LASTEXITCODE -ne 0) { throw "Manifest signing failed" }
node (Join-Path $Root "scripts\verify-manifest.mjs") $manifestPath $manifestSig
if ($LASTEXITCODE -ne 0) { throw "Manifest signature verification failed" }

$uiDest = Join-Path $stage "ui"
New-Item -ItemType Directory -Path $uiDest | Out-Null
Copy-Item (Join-Path $UiDir "*") -Destination $uiDest -Recurse -Force

# Verify the file that will actually ship, not the source tree.
Write-Host "Verifying packaged UI..." -ForegroundColor Cyan
node (Join-Path $Root "scripts\check-frontend.mjs") (Join-Path $uiDest "index.html") $RustSource
if ($LASTEXITCODE -ne 0) { throw "Packaged frontend failed checks" }

$iconsDest = Join-Path $stage "icons"
New-Item -ItemType Directory -Path $iconsDest | Out-Null
Copy-Item (Join-Path $Root "crates\false-ui\icons\*") -Destination $iconsDest -Recurse -Force
Copy-Item $uiExe -Destination (Join-Path $stage "false.exe") -Force

$tempZip = Join-Path $Root "false-temp.zip"
$flseFile = Join-Path $Root "false-v$Version.flse"

if (Test-Path $tempZip) { Remove-Item $tempZip -Force }
if (Test-Path $flseFile) { Remove-Item $flseFile -Force }

Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $tempZip -Force
Move-Item $tempZip $flseFile -Force
Remove-Item $stage -Recurse -Force

Write-Host "=== All artifacts built & ready for distribution ===" -ForegroundColor Green

if ($Bundle) {
    Write-Host "=== Building the NSIS installer with tauri build ===" -ForegroundColor Cyan
    $tauri = Get-Command tauri -ErrorAction SilentlyContinue
    if (-not $tauri) {
        $tauri = Get-Command tauri.cmd -ErrorAction SilentlyContinue
    }
    if (-not $tauri) {
        throw "tauri CLI not found. Install it with: npm install -g @tauri-apps/cli@^2"
    }
    Push-Location (Join-Path $Root "crates\false-ui")
    try {
        & $tauri.Source build --bundles nsis
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
    } finally {
        Pop-Location
    }
    $nsis = Join-Path $Root "target\release\bundle\nsis\False_${Version}_x64-setup.exe"
    if (-not (Test-Path $nsis)) {
        throw "NSIS installer not found: $nsis"
    }
    $setupOut = Join-Path $Root "False_${Version}_x64-setup.exe"
    Copy-Item $nsis $setupOut -Force
    Write-Host "NSIS installer: $setupOut" -ForegroundColor Green
}

Get-Item $setupDest, $flseFile | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
