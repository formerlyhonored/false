use std::path::{Path, PathBuf};
use std::process::Command;

fn local_app_data() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default\AppData\Local"))
}

fn user_profile() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn app_data_roaming() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

const EMBEDDED_FALSE_EXE: &[u8] = include_bytes!("../../../target/debug/false-ui.exe");
const EMBEDDED_ICON_ICO: &[u8] = include_bytes!("../../false-ui/icons/icon.ico");

fn main() {
    println!("\x1b[1;36m==========================================================\x1b[0m");
    println!("\x1b[1;37m                 FALSE BROWSER INSTALLER                  \x1b[0m");
    println!("\x1b[1;36m==========================================================\x1b[0m");

    let install_dir = local_app_data().join("Programs").join("False");
    println!("[1/5] Preparing installation directory: {}", install_dir.display());

    // Stop existing instances if running
    let _ = Command::new("taskkill")
        .args(["/f", "/im", "false.exe"])
        .output();
    let _ = Command::new("taskkill")
        .args(["/f", "/im", "false-ui.exe"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(300));

    let _ = std::fs::create_dir_all(&install_dir);
    let ui_dest = install_dir.join("ui");
    let icons_dest = install_dir.join("icons");
    let _ = std::fs::create_dir_all(&ui_dest);
    let _ = std::fs::create_dir_all(&icons_dest);

    // Clean stale sessions from previous runs to guarantee clean first launch
    let db_path = local_app_data().join("False").join("false.db");
    if db_path.exists() {
        let _ = Command::new("powershell")
            .args([
                "-NoProfile",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &format!(
                    r#"python -c "import sqlite3; c=sqlite3.connect(r'{}'); c.execute(\"DELETE FROM settings WHERE key='session' OR value LIKE '%accounts.google.com%'\"); c.commit(); c.close()" 2>$null"#,
                    db_path.display()
                ),
            ])
            .output();
    }

    println!("[2/5] Installing binaries and UI components...");
    let dest_false_exe = install_dir.join("false.exe");
    let dest_ui_exe = install_dir.join("false-ui.exe");
    if let Err(e) = std::fs::write(&dest_false_exe, EMBEDDED_FALSE_EXE) {
        eprintln!("[!] Note writing false.exe: {}", e);
    }
    if let Err(e) = std::fs::write(&dest_ui_exe, EMBEDDED_FALSE_EXE) {
        eprintln!("[!] Note writing false-ui.exe: {}", e);
    }

    let _ = std::fs::write(icons_dest.join("icon.ico"), EMBEDDED_ICON_ICO);

    // Copy additional loose UI files if present
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut ui_source = cwd.join("crates").join("false-ui").join("ui");
    if !ui_source.exists() {
        ui_source = PathBuf::from(r"d:\false\crates\false-ui\ui");
    }
    if ui_source.exists() {
        copy_dir_all(&ui_source, &ui_dest);
    }

    println!("[3/5] Creating Desktop and Start Menu shortcuts...");
    let desktop_shortcut = user_profile().join("Desktop").join("False.lnk");
    let start_menu_shortcut = app_data_roaming()
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("False.lnk");

    create_shortcut(
        &install_dir.join("false.exe"),
        &install_dir,
        &icons_dest.join("icon.ico"),
        &desktop_shortcut,
    );
    create_shortcut(
        &install_dir.join("false.exe"),
        &install_dir,
        &icons_dest.join("icon.ico"),
        &start_menu_shortcut,
    );

    println!("[4/5] Registering .flse file extension association...");
    register_flse_extension(&install_dir);

    println!("[5/5] Creating uninstaller...");
    create_uninstaller(&install_dir, &desktop_shortcut, &start_menu_shortcut);

    println!("\x1b[1;32m==========================================================\x1b[0m");
    println!("\x1b[1;32mInstallation complete! False is ready to use.\x1b[0m");
    println!("Installed at: {}", install_dir.display());
    println!("Extension association: .flse -> False Browser Update");
    println!("\x1b[1;36mLaunching False...\x1b[0m");

    let _ = Command::new(install_dir.join("false.exe")).spawn();
}

fn copy_dir_all(src: &Path, dst: &Path) {
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let p = entry.path();
            let dest_file = dst.join(entry.file_name());
            if p.is_file() {
                let _ = std::fs::copy(&p, &dest_file);
            } else if p.is_dir() {
                let _ = std::fs::create_dir_all(&dest_file);
                copy_dir_all(&p, &dest_file);
            }
        }
    }
}

fn create_shortcut(target: &Path, workdir: &Path, icon: &Path, lnk: &Path) {
    let script = format!(
        r#"$w = New-Object -ComObject WScript.Shell; $s = $w.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.WorkingDirectory = '{}'; $s.IconLocation = '{},0'; $s.Description = 'False Browser'; $s.Save()"#,
        lnk.display(),
        target.display(),
        workdir.display(),
        icon.display()
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

fn register_flse_extension(install_dir: &Path) {
    let target_exe = install_dir.join("false.exe");
    let icon_path = install_dir.join("icons").join("icon.ico");

    let script = format!(
        r#"
New-Item -Path "HKCU:\Software\Classes\.flse" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\.flse" -Name "(Default)" -Value "False.UpdatePackage" -Force
New-Item -Path "HKCU:\Software\Classes\False.UpdatePackage" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\False.UpdatePackage" -Name "(Default)" -Value "False Browser Update Package" -Force
New-Item -Path "HKCU:\Software\Classes\False.UpdatePackage\DefaultIcon" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\False.UpdatePackage\DefaultIcon" -Name "(Default)" -Value "{},0" -Force
New-Item -Path "HKCU:\Software\Classes\False.UpdatePackage\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\False.UpdatePackage\shell\open\command" -Name "(Default)" -Value '"{}" "%1"' -Force

$u = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\False"
New-Item -Path $u -Force | Out-Null
Set-ItemProperty -Path $u -Name "DisplayName" -Value "False Browser" -Force
Set-ItemProperty -Path $u -Name "DisplayVersion" -Value "0.1.1" -Force
Set-ItemProperty -Path $u -Name "Publisher" -Value "zaidi" -Force
Set-ItemProperty -Path $u -Name "DisplayIcon" -Value "{}" -Force
Set-ItemProperty -Path $u -Name "InstallLocation" -Value "{}" -Force
Set-ItemProperty -Path $u -Name "UninstallString" -Value '"{}\uninstall.bat"' -Force
"#,
        icon_path.display(),
        target_exe.display(),
        icon_path.display(),
        install_dir.display(),
        install_dir.display()
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

fn create_uninstaller(install_dir: &Path, desktop_lnk: &Path, start_lnk: &Path) {
    let content = format!(
        r#"@echo off
echo Uninstalling False...
taskkill /f /im false.exe > nul 2>&1
taskkill /f /im false-ui.exe > nul 2>&1
del "{}" > nul 2>&1
del "{}" > nul 2>&1
reg delete "HKCU\Software\Classes\.flse" /f > nul 2>&1
reg delete "HKCU\Software\Classes\False.UpdatePackage" /f > nul 2>&1
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\False" /f > nul 2>&1
cd /d "%TEMP%"
rmdir /s /q "{}" > nul 2>&1
echo False has been uninstalled.
pause
"#,
        desktop_lnk.display(),
        start_lnk.display(),
        install_dir.display()
    );
    let _ = std::fs::write(install_dir.join("uninstall.bat"), content);
}
