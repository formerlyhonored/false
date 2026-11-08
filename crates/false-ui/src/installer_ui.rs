use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x00000008;
#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

const ICON_BYTES: &[u8] = include_bytes!("../icons/icon.ico");

fn run_silent_command(cmd: &mut Command) {
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.output();
}

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

pub fn default_install_dir() -> PathBuf {
    local_app_data().join("Programs").join("False")
}

/// Validate a user-supplied install folder before it is interpolated into
/// VBScript or passed to `reg`. Rejects shell/script metacharacters.
fn validate_install_path(value: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Install folder is empty.".to_string());
    }
    for forbidden in [
        '%', '"', '\'', '\r', '\n', '\0', ';', '&', '|', '<', '>', '$', '`',
    ] {
        if trimmed.contains(forbidden) {
            return Err(format!(
                "Install folder cannot contain the character '{}'.",
                forbidden
            ));
        }
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err("Install folder must be an absolute path.".to_string());
    }
    if path.parent().is_none() {
        return Err("Refusing to install into a drive root.".to_string());
    }
    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::CurDir) {
            return Err("Install folder cannot contain '.' or '..' components.".to_string());
        }
    }
    Ok(path)
}

pub fn resolve_path(p: Option<String>) -> PathBuf {
    match p {
        Some(s) if validate_install_path(&s).is_ok() => PathBuf::from(s.trim()),
        _ => default_install_dir(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct InstallerDefaults {
    pub path: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct InstallOptions {
    pub path: Option<String>,
    #[serde(alias = "desktopShortcut")]
    pub desktop_shortcut: Option<bool>,
    #[serde(alias = "setDefault")]
    pub set_default: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UninstallOptions {
    #[serde(alias = "deleteData")]
    pub delete_data: Option<bool>,
}

#[tauri::command]
pub fn installer_get_defaults() -> InstallerDefaults {
    InstallerDefaults {
        path: default_install_dir().to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub fn installer_pick_folder(current: Option<String>) -> Option<String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.FolderBrowserDialog
$f.Description = "Select installation folder for False Browser"
$f.ShowNewFolderButton = $true
if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $f.SelectedPath
}
"#;
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !path.is_empty() {
        Some(path)
    } else {
        current
    }
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

fn setup_shortcuts(
    desktop_lnk: &Path,
    start_lnk: &Path,
    target_exe: &Path,
    install_dir: &Path,
    icon_path: &Path,
    desktop_shortcut: bool,
) {
    if let Some(p) = desktop_lnk.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Some(p) = start_lnk.parent() {
        let _ = std::fs::create_dir_all(p);
    }

    let mut vbs = String::from("Set ws = CreateObject(\"WScript.Shell\")\r\n");
    if desktop_shortcut {
        vbs.push_str(&format!(
            "Set s1 = ws.CreateShortcut(\"{}\")\r\ns1.TargetPath = \"{}\"\r\ns1.WorkingDirectory = \"{}\"\r\ns1.IconLocation = \"{},0\"\r\ns1.Description = \"False Browser\"\r\ns1.Save\r\n",
            desktop_lnk.display(),
            target_exe.display(),
            install_dir.display(),
            icon_path.display()
        ));
    }
    vbs.push_str(&format!(
        "Set s2 = ws.CreateShortcut(\"{}\")\r\ns2.TargetPath = \"{}\"\r\ns2.WorkingDirectory = \"{}\"\r\ns2.IconLocation = \"{},0\"\r\ns2.Description = \"False Browser\"\r\ns2.Save\r\n",
        start_lnk.display(),
        target_exe.display(),
        install_dir.display(),
        icon_path.display()
    ));

    let temp_vbs = std::env::temp_dir().join(format!("false_lnk_{}.vbs", std::process::id()));
    if std::fs::write(&temp_vbs, vbs).is_ok() {
        let mut cmd = Command::new("cscript");
        cmd.args(["//nologo", &temp_vbs.to_string_lossy()]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.output();
        let _ = std::fs::remove_file(&temp_vbs);
    }
}

fn setup_registry(target_exe: &Path, install_dir: &Path, icon_path: &Path) {
    let target_str = target_exe.to_string_lossy();
    let install_str = install_dir.to_string_lossy();
    let icon_str = icon_path.to_string_lossy();

    let uninstall_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\False";
    let entries = [
        ("DisplayName", "False Browser"),
        ("DisplayVersion", env!("CARGO_PKG_VERSION")),
        ("Publisher", "zaidi"),
        ("DisplayIcon", &format!("{},0", icon_str)),
        ("InstallLocation", &install_str),
        (
            "UninstallString",
            &format!("\"{}\" --uninstaller", target_str),
        ),
    ];

    for (name, val) in entries {
        let mut cmd = Command::new("reg");
        cmd.args([
            "add",
            uninstall_key,
            "/v",
            name,
            "/t",
            "REG_SZ",
            "/d",
            val,
            "/f",
        ]);
        run_silent_command(&mut cmd);
    }

    for (name, val) in [("NoModify", "1"), ("NoRepair", "1")] {
        let mut cmd = Command::new("reg");
        cmd.args([
            "add",
            uninstall_key,
            "/v",
            name,
            "/t",
            "REG_DWORD",
            "/d",
            val,
            "/f",
        ]);
        run_silent_command(&mut cmd);
    }

    // .flse file association
    let mut cmd = Command::new("reg");
    cmd.args([
        "add",
        r"HKCU\Software\Classes\.flse",
        "/ve",
        "/d",
        "False.UpdatePackage",
        "/f",
    ]);
    run_silent_command(&mut cmd);

    let mut cmd = Command::new("reg");
    cmd.args([
        "add",
        r"HKCU\Software\Classes\False.UpdatePackage",
        "/ve",
        "/d",
        "False Browser Update Package",
        "/f",
    ]);
    run_silent_command(&mut cmd);

    let mut cmd = Command::new("reg");
    cmd.args([
        "add",
        r"HKCU\Software\Classes\False.UpdatePackage\DefaultIcon",
        "/ve",
        "/d",
        &format!("{},0", icon_str),
        "/f",
    ]);
    run_silent_command(&mut cmd);

    let mut cmd = Command::new("reg");
    cmd.args([
        "add",
        r"HKCU\Software\Classes\False.UpdatePackage\shell\open\command",
        "/ve",
        "/d",
        &format!("\"{}\" \"%1\"", target_str),
        "/f",
    ]);
    run_silent_command(&mut cmd);
}

#[tauri::command]
pub async fn installer_run(
    path: Option<String>,
    desktop_shortcut: Option<bool>,
    set_default: Option<bool>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let install_dir = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(value) => validate_install_path(value)?,
            None => default_install_dir(),
        };

        // Terminate any running instances so binaries are not locked
        let mut kill1 = Command::new("taskkill");
        kill1.args(["/f", "/im", "false.exe"]);
        run_silent_command(&mut kill1);

        let mut kill2 = Command::new("taskkill");
        kill2.args(["/f", "/im", "false-ui.exe"]);
        run_silent_command(&mut kill2);

        std::thread::sleep(std::time::Duration::from_millis(100));

        let ui_dest = install_dir.join("ui");
        let icons_dest = install_dir.join("icons");
        let _ = std::fs::create_dir_all(&install_dir);
        let _ = std::fs::create_dir_all(&ui_dest);
        let _ = std::fs::create_dir_all(&icons_dest);

        // Always extract clean icon
        let icon_file = icons_dest.join("icon.ico");
        let _ = std::fs::write(&icon_file, ICON_BYTES);

        // Locate source executable
        let current_exe = std::env::current_exe().unwrap_or_default();
        let target_false = install_dir.join("false.exe");
        let target_ui = install_dir.join("false-ui.exe");

        let mut copied = false;
        if current_exe.exists()
            && current_exe != target_false
            && std::fs::copy(&current_exe, &target_false).is_ok()
        {
            let _ = std::fs::copy(&current_exe, &target_ui);
            copied = true;
        }

        if !copied {
            let dev_exe = PathBuf::from(r"d:\false\target\debug\false-ui.exe");
            if dev_exe.exists() && dev_exe != target_false {
                let _ = std::fs::copy(&dev_exe, &target_false);
                let _ = std::fs::copy(&dev_exe, &target_ui);
            }
        }

        if !target_false.exists() {
            return Err("Failed to copy false.exe to installation folder".to_string());
        }

        // Copy UI files if available on disk
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut ui_source = current_exe.parent().unwrap_or(&cwd).join("ui");
        if !ui_source.exists() {
            ui_source = PathBuf::from(r"d:\false\crates\false-ui\ui");
        }
        if ui_source.exists() && ui_source != ui_dest {
            copy_dir_all(&ui_source, &ui_dest);
        }

        // Shortcuts
        let desktop_lnk = user_profile().join("Desktop").join("False.lnk");
        let start_lnk = app_data_roaming()
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("False.lnk");

        setup_shortcuts(
            &desktop_lnk,
            &start_lnk,
            &target_false,
            &install_dir,
            &icon_file,
            desktop_shortcut.unwrap_or(true),
        );

        // Registry configuration
        setup_registry(&target_false, &install_dir, &icon_file);

        if set_default.unwrap_or(false) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/c", "start", "ms-settings:defaultapps"]);
            run_silent_command(&mut cmd);
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn installer_launch_app(path: Option<String>) {
    let install_dir = resolve_path(path);
    let target_false = install_dir.join("false.exe");
    if target_false.exists() {
        let mut cmd = Command::new(&target_false);
        cmd.current_dir(&install_dir);
        #[cfg(windows)]
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        let _ = cmd.spawn();
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    std::process::exit(0);
}

#[tauri::command]
pub fn installer_close() {
    std::process::exit(0);
}

#[tauri::command]
pub async fn uninstaller_run(delete_data: Option<bool>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        // 1. Remove Desktop & Start Menu shortcuts
        let desktop_lnk = user_profile().join("Desktop").join("False.lnk");
        let start_lnk = app_data_roaming()
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("False.lnk");

        let _ = std::fs::remove_file(desktop_lnk);
        let _ = std::fs::remove_file(start_lnk);

        // 2. Clean registry keys
        let keys = [
            r"HKCU\Software\Classes\.flse",
            r"HKCU\Software\Classes\False.UpdatePackage",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\False",
        ];
        for key in keys {
            let mut cmd = Command::new("reg");
            cmd.args(["delete", key, "/f"]);
            run_silent_command(&mut cmd);
        }

        // 3. Optional user data deletion
        if delete_data.unwrap_or(false) {
            let false_data = local_app_data().join("False");
            let false_ui_data = local_app_data().join("false-ui");
            let _ = std::fs::remove_dir_all(false_data);
            let _ = std::fs::remove_dir_all(false_ui_data);
        }

        // 4. Schedule installation directory cleanup after process exit
        let install_dir = default_install_dir();
        let cleanup_cmd = format!(
            "ping 127.0.0.1 -n 3 > nul & rmdir /s /q \"{}\"",
            install_dir.display()
        );
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", &cleanup_cmd]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.spawn();

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn uninstaller_close() {
    std::process::exit(0);
}
