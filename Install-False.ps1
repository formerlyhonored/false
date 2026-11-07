# ============================================================================
# False Browser - Windows Official Installer
# ============================================================================
param(
    [switch]$Silent,
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "               INSTALLING FALSE BROWSER                   " -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Cyan

$InstallDir = "$env:LOCALAPPDATA\Programs\False"
$BinSource  = "d:\false\target\debug\false-ui.exe"
$UiSource   = "d:\false\crates\false-ui\ui"
$IconSource = "d:\false\crates\false-ui\icons"

if (!(Test-Path $BinSource)) {
    Write-Host "[!] Error: false-ui.exe not found at $BinSource" -ForegroundColor Red
    exit 1
}

Write-Host "[1/5] Stopping any running False instances..." -ForegroundColor Yellow
Get-Process false-ui -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

Write-Host "[2/5] Copying application files to $InstallDir..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$InstallDir\ui" | Out-Null
New-Item -ItemType Directory -Force -Path "$InstallDir\icons" | Out-Null

Copy-Item $BinSource "$InstallDir\false.exe" -Force
Copy-Item $BinSource "$InstallDir\false-ui.exe" -Force
Copy-Item "$UiSource\*" "$InstallDir\ui\" -Recurse -Force
Copy-Item "$IconSource\*" "$InstallDir\icons\" -Recurse -Force

Write-Host "[3/5] Creating Desktop & Start Menu shortcuts..." -ForegroundColor Yellow
$WshShell = New-Object -ComObject WScript.Shell

# Desktop shortcut
$DesktopShortcutPath = "$env:USERPROFILE\Desktop\False.lnk"
$Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
$Shortcut.TargetPath = "$InstallDir\false.exe"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.IconLocation = "$InstallDir\icons\icon.ico, 0"
$Shortcut.Description = "False - Minimal, fast web navigator"
$Shortcut.Save()

# Start Menu shortcut
$StartMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$StartShortcutPath = "$StartMenuDir\False.lnk"
$Shortcut2 = $WshShell.CreateShortcut($StartShortcutPath)
$Shortcut2.TargetPath = "$InstallDir\false.exe"
$Shortcut2.WorkingDirectory = $InstallDir
$Shortcut2.IconLocation = "$InstallDir\icons\icon.ico, 0"
$Shortcut2.Description = "False - Minimal, fast web navigator"
$Shortcut2.Save()

Write-Host "[4/5] Registering .flse file extension in Windows..." -ForegroundColor Yellow
$RegClasses = "HKCU:\Software\Classes"

# Register .flse extension
New-Item -Path "$RegClasses\.flse" -Force | Out-Null
Set-ItemProperty -Path "$RegClasses\.flse" -Name "(Default)" -Value "False.UpdatePackage" -Force

# Register ProgID False.UpdatePackage
New-Item -Path "$RegClasses\False.UpdatePackage" -Force | Out-Null
Set-ItemProperty -Path "$RegClasses\False.UpdatePackage" -Name "(Default)" -Value "False Browser Update Package" -Force

# Icon for .flse files
New-Item -Path "$RegClasses\False.UpdatePackage\DefaultIcon" -Force | Out-Null
Set-ItemProperty -Path "$RegClasses\False.UpdatePackage\DefaultIcon" -Name "(Default)" -Value "$InstallDir\icons\icon.ico,0" -Force

# Shell open command for .flse files
New-Item -Path "$RegClasses\False.UpdatePackage\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "$RegClasses\False.UpdatePackage\shell\open\command" -Name "(Default)" -Value "`"$InstallDir\false.exe`" `"%1`"" -Force

# Windows Add/Remove Programs entry
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\False"
New-Item -Path $UninstallKey -Force | Out-Null
Set-ItemProperty -Path $UninstallKey -Name "DisplayName" -Value "False Browser" -Force
Set-ItemProperty -Path $UninstallKey -Name "DisplayVersion" -Value "0.1.0" -Force
Set-ItemProperty -Path $UninstallKey -Name "Publisher" -Value "zaidi" -Force
Set-ItemProperty -Path $UninstallKey -Name "DisplayIcon" -Value "$InstallDir\icons\icon.ico" -Force
Set-ItemProperty -Path $UninstallKey -Name "InstallLocation" -Value $InstallDir -Force
Set-ItemProperty -Path $UninstallKey -Name "UninstallString" -Value "`"$InstallDir\uninstall.bat`"" -Force

# Create clean uninstaller
$UninstallScript = @"
@echo off
echo Uninstalling False...
taskkill /f /im false.exe > nul 2>&1
taskkill /f /im false-ui.exe > nul 2>&1
del "$DesktopShortcutPath" > nul 2>&1
del "$StartShortcutPath" > nul 2>&1
reg delete "HKCU\Software\Classes\.flse" /f > nul 2>&1
reg delete "HKCU\Software\Classes\False.UpdatePackage" /f > nul 2>&1
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\False" /f > nul 2>&1
cd /d "%TEMP%"
rmdir /s /q "$InstallDir" > nul 2>&1
echo False has been uninstalled successfully.
pause
"@
Set-Content -Path "$InstallDir\uninstall.bat" -Value $UninstallScript -Encoding ASCII

Write-Host "[5/5] Installation complete!" -ForegroundColor Green
Write-Host "Installed to: $InstallDir" -ForegroundColor Cyan
Write-Host "File association: .flse -> False Browser Update" -ForegroundColor Cyan
Write-Host "Shortcuts: Desktop & Start Menu created with custom star icon" -ForegroundColor Cyan

if (-not $NoLaunch) {
    Write-Host "Launching False..." -ForegroundColor Green
    Start-Process "$InstallDir\false.exe"
}
