//! False Package (.flse) importer and updater.
//!
//! A `.flse` file is a ZIP package containing:
//! - `manifest.json`: Metadata, target version, description.
//! - `ui/index.html` or `index.html`: Updated browser frontend.
//! - Optional `ui/`: Extra frontend assets (icons, styles).
//! - Optional `false-ui.exe` or `false.exe`: Native Rust browser executable.
//!
//! Importing a `.flse` file updates False's UI immediately, and if a new
//! executable is present, stages it and restarts seamlessly.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FlseManifest {
    pub name: String,
    pub version: String,
    #[serde(default = "default_author")]
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub has_binary: bool,
    #[serde(default)]
    pub has_ui: bool,
}

fn default_author() -> String {
    "zaidi".to_string()
}

impl Default for FlseManifest {
    fn default() -> Self {
        Self {
            name: "False Update".to_string(),
            version: "0.1.0".to_string(),
            author: "zaidi".to_string(),
            description: "False browser update package".to_string(),
            created_at: String::new(),
            has_binary: false,
            has_ui: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FlseUpdateResult {
    pub success: bool,
    pub version: String,
    pub name: String,
    pub description: String,
    pub updated_files: Vec<String>,
    pub requires_restart: bool,
    pub message: String,
}

fn now_stamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", now)
}

/// Applies a .flse update package from raw bytes (either from disk or base64 upload).
pub fn apply_package_bytes(bytes: &[u8]) -> Result<FlseUpdateResult, String> {
    if bytes.is_empty() {
        return Err("Package data is empty.".to_string());
    }

    let cursor = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid .flse package (failed to read zip archive): {}", e))?;

    // 1. Read manifest.json if present
    let mut manifest = FlseManifest::default();
    let mut manifest_text: Option<String> = None;
    if let Ok(mut f) = zip.by_name("manifest.json") {
        let mut text = String::new();
        if f.read_to_string(&mut text).is_ok() {
            manifest_text = Some(text);
        }
    }
    if manifest_text.is_none() {
        if let Ok(mut f) = zip.by_name("false.manifest.json") {
            let mut text = String::new();
            if f.read_to_string(&mut text).is_ok() {
                manifest_text = Some(text);
            }
        }
    }
    if let Some(text) = manifest_text {
        if let Ok(m) = serde_json::from_str::<FlseManifest>(&text) {
            manifest = m;
        }
    }

    let ui_dir = crate::ui_root();
    let updates_dir = crate::data_dir().join("updates");
    let backups_dir = crate::data_dir().join("backups").join(format!("backup_{}", now_stamp()));
    let _ = std::fs::create_dir_all(&updates_dir);
    let _ = std::fs::create_dir_all(&backups_dir);

    let mut updated_files = Vec::new();
    let mut staged_binary: Option<PathBuf> = None;

    // 2. Extract contents
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let raw_name = match file.name() {
            Ok(n) => n.to_string(),
            Err(_) => continue,
        };
        let normalized = raw_name.replace('\\', "/");

        // Skip directories or traversal
        if normalized.ends_with('/') || normalized.contains("..") {
            continue;
        }

        // Handle binary update
        if normalized == "false-ui.exe" || normalized == "false.exe" || normalized.ends_with("/false-ui.exe") {
            let target = updates_dir.join("false-ui_new.exe");
            let mut out = File::create(&target).map_err(|e| format!("Cannot write staged binary: {}", e))?;
            std::io::copy(&mut file, &mut out).map_err(|e| format!("Failed to extract binary: {}", e))?;
            staged_binary = Some(target);
            updated_files.push("false-ui.exe (native binary)".to_string());
            continue;
        }

        // Handle UI update
        if normalized == "index.html" || normalized == "ui/index.html" || normalized.starts_with("ui/") {
            let rel_path = normalized.trim_start_matches("ui/").trim_start_matches('/');
            let target_path = ui_dir.join(rel_path);

            if let Some(parent) = target_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Backup existing file if present
            if target_path.exists() {
                let backup_file = backups_dir.join(rel_path);
                if let Some(parent) = backup_file.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::copy(&target_path, backup_file);
            }

            let mut out = File::create(&target_path)
                .map_err(|e| format!("Cannot write file {}: {}", target_path.display(), e))?;
            std::io::copy(&mut file, &mut out)
                .map_err(|e| format!("Failed to extract file {}: {}", rel_path, e))?;

            updated_files.push(rel_path.to_string());
        }
    }

    if updated_files.is_empty() {
        return Err("No update payload found in .flse package (expected index.html or false-ui.exe).".to_string());
    }

    // Record the installed version
    let version_file = crate::data_dir().join("version.json");
    let version_record = serde_json::json!({
        "version": manifest.version,
        "name": manifest.name,
        "description": manifest.description,
        "installed_at": now_stamp(),
    });
    let _ = std::fs::write(&version_file, serde_json::to_string_pretty(&version_record).unwrap_or_default());

    // If a binary was updated, trigger the hot restart updater script
    let requires_restart = staged_binary.is_some();
    if let Some(staged) = staged_binary {
        if let Ok(current_exe) = std::env::current_exe() {
            let batch_path = updates_dir.join("apply_update.bat");
            let batch_content = format!(
                "@echo off\r\ntimeout /t 1 /nobreak > nul\r\ntaskkill /f /im false-ui.exe > nul 2>&1\r\nmove /y \"{}\" \"{}\" > nul\r\nstart \"\" \"{}\"\r\ndel \"%~f0\" & exit\r\n",
                staged.display(),
                current_exe.display(),
                current_exe.display()
            );
            let _ = std::fs::write(&batch_path, batch_content);

            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "", &batch_path.to_string_lossy()])
                .spawn();

            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(400));
                std::process::exit(0);
            });
        }
    }

    let message = if requires_restart {
        format!(
            "Successfully installed False update v{}! False is restarting with the new version.",
            manifest.version
        )
    } else {
        format!(
            "Successfully imported False update v{} ({} files updated). The UI has been refreshed.",
            manifest.version,
            updated_files.len()
        )
    };

    Ok(FlseUpdateResult {
        success: true,
        version: manifest.version,
        name: manifest.name,
        description: manifest.description,
        updated_files,
        requires_restart,
        message,
    })
}

/// Creates a new .flse package from the current workspace files
pub fn create_package(
    version: &str,
    description: &str,
    include_binary: bool,
    out_path: &Path,
) -> Result<(), String> {
    let file = File::create(out_path)
        .map_err(|e| format!("Failed to create {}: {}", out_path.display(), e))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Write manifest
    let manifest = FlseManifest {
        name: "False Browser Update".to_string(),
        version: version.to_string(),
        author: "zaidi".to_string(),
        description: description.to_string(),
        created_at: now_stamp(),
        has_binary: include_binary,
        has_ui: true,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;

    zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
    zip.write_all(&manifest_bytes).map_err(|e| e.to_string())?;

    // 2. Write UI files (index.html, logos)
    let ui_dir = crate::ui_root();
    if let Ok(entries) = std::fs::read_dir(&ui_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let zip_name = format!("ui/{}", fname);
                if let Ok(data) = std::fs::read(&path) {
                    let _ = zip.start_file(zip_name, options);
                    let _ = zip.write_all(&data);
                }
            }
        }
    }

    // 3. Write binary if requested
    if include_binary {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Ok(bin_data) = std::fs::read(&exe_path) {
                let _ = zip.start_file("false-ui.exe", options);
                let _ = zip.write_all(&bin_data);
            }
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize .flse zip: {}", e))?;
    Ok(())
}
