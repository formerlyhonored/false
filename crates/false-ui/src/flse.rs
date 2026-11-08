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

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Ed25519 public key (raw 32 bytes, base64) pinned into every release.
const UPDATE_PUBLIC_KEY_B64: &str = include_str!("../keys/update-signing.pub");
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_ENTRY_BYTES: u64 = 512 * 1024 * 1024;

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
            version: env!("CARGO_PKG_VERSION").to_string(),
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

fn decode_public_key() -> Result<VerifyingKey, String> {
    let raw = B64
        .decode(UPDATE_PUBLIC_KEY_B64.trim())
        .map_err(|e| format!("Embedded update public key is invalid: {}", e))?;
    let bytes: [u8; 32] = raw
        .as_slice()
        .try_into()
        .map_err(|_| "Embedded update public key must be 32 bytes.".to_string())?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|e| format!("Embedded update public key is invalid: {}", e))
}

fn verify_manifest_signature_with_key(
    key: &VerifyingKey,
    manifest_bytes: &[u8],
    signature_b64: &str,
) -> Result<(), String> {
    let raw = B64
        .decode(signature_b64.trim())
        .map_err(|e| format!("Package signature is not valid base64: {}", e))?;
    let sig = Signature::from_slice(&raw)
        .map_err(|e| format!("Package signature has the wrong length: {}", e))?;
    key.verify_strict(manifest_bytes, &sig).map_err(|_| {
        "Package signature verification failed. This update was not signed by False.".to_string()
    })
}

fn verify_manifest_signature(manifest_bytes: &[u8], signature_b64: &str) -> Result<(), String> {
    let key = decode_public_key()?;
    verify_manifest_signature_with_key(&key, manifest_bytes, signature_b64)
}

fn read_verified_manifest<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
) -> Result<(FlseManifest, Vec<u8>), String> {
    let manifest_name = if zip.by_name("manifest.json").is_ok() {
        "manifest.json"
    } else {
        "false.manifest.json"
    };

    let mut manifest_bytes = Vec::new();
    {
        let mut file = zip
            .by_name(manifest_name)
            .map_err(|_| "Package is missing manifest.json.".to_string())?;
        if file.size() > MAX_MANIFEST_BYTES {
            return Err("Package manifest is unexpectedly large.".to_string());
        }
        file.read_to_end(&mut manifest_bytes)
            .map_err(|e| format!("Cannot read package manifest: {}", e))?;
    }

    let mut signature_text = String::new();
    {
        let mut file = zip
            .by_name("manifest.sig")
            .map_err(|_| "Unsigned package rejected: manifest.sig is missing.".to_string())?;
        if file.size() > 8 * 1024 {
            return Err("Package signature file is unexpectedly large.".to_string());
        }
        file.read_to_string(&mut signature_text)
            .map_err(|e| format!("Cannot read package signature: {}", e))?;
    }

    verify_manifest_signature(&manifest_bytes, &signature_text)?;
    let manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("Invalid package manifest JSON: {}", e))?;
    Ok((manifest, manifest_bytes))
}

fn is_reserved_windows_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

/// Reject absolute paths, traversal, drive prefixes, alternate data streams,
/// and Windows device names before joining a package entry into `ui_root`.
fn validate_package_relative_path(rel: &str) -> Result<(), String> {
    let mut saw_component = false;
    for component in Path::new(rel).components() {
        match component {
            Component::Normal(part) => {
                let name = part.to_string_lossy();
                if name.is_empty() || name == "." || name == ".." {
                    return Err(format!("Unsafe package entry: {}", rel));
                }
                if name.contains(':') {
                    return Err(format!("Unsafe package entry (stream): {}", rel));
                }
                if name.ends_with(' ') || name.ends_with('.') {
                    return Err(format!(
                        "Unsafe package entry (trailing dot/space): {}",
                        rel
                    ));
                }
                if is_reserved_windows_name(&name) {
                    return Err(format!("Unsafe package entry (reserved device): {}", rel));
                }
                saw_component = true;
            }
            _ => return Err(format!("Package entry escapes the UI directory: {}", rel)),
        }
    }
    if !saw_component {
        return Err("Package entry has an empty path.".to_string());
    }
    Ok(())
}

fn safe_join(base: &Path, rel: &str) -> Result<PathBuf, String> {
    validate_package_relative_path(rel)?;
    Ok(base.join(rel))
}

fn validate_package_entries<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
) -> Result<(), String> {
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|e| e.to_string())?;
        let raw_name = file.name().map_err(|e| e.to_string())?.replace('\\', "/");
        if raw_name.ends_with('/') {
            continue;
        }
        if matches!(
            raw_name.as_str(),
            "manifest.json" | "false.manifest.json" | "manifest.sig"
        ) {
            continue;
        }
        if file.size() > MAX_ENTRY_BYTES {
            return Err(format!("Package entry is too large: {}", raw_name));
        }
        if raw_name == "false-ui.exe"
            || raw_name == "false.exe"
            || raw_name.ends_with("/false-ui.exe")
        {
            if file.size() == 0 {
                return Err("Package contains an empty binary.".to_string());
            }
            continue;
        }
        if raw_name == "index.html" {
            continue;
        }
        if let Some(rest) = raw_name.strip_prefix("ui/") {
            validate_package_relative_path(rest)?;
        }
    }
    Ok(())
}

/// Reads a package and returns its verified manifest without applying it.
pub fn preview_package_bytes(bytes: &[u8]) -> Result<FlseManifest, String> {
    if bytes.is_empty() {
        return Err("Package data is empty.".to_string());
    }
    let cursor = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid .flse package (failed to read zip archive): {}", e))?;
    let (manifest, _) = read_verified_manifest(&mut zip)?;
    validate_package_entries(&mut zip)?;
    Ok(manifest)
}

/// Verifies and previews a package file on disk.
pub fn preview_package_file(path: &Path) -> Result<FlseManifest, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Cannot read update package {}: {}", path.display(), e))?;
    preview_package_bytes(&bytes)
}

/// Applies a .flse update package from raw bytes (either from disk or base64 upload).
pub fn apply_package_bytes(bytes: &[u8]) -> Result<FlseUpdateResult, String> {
    if bytes.is_empty() {
        return Err("Package data is empty.".to_string());
    }

    let cursor = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid .flse package (failed to read zip archive): {}", e))?;

    // 1. Read and verify the signed manifest before touching the filesystem.
    let (manifest, _manifest_bytes) = read_verified_manifest(&mut zip)?;
    validate_package_entries(&mut zip)?;

    let ui_dir = crate::ui_root();
    let updates_dir = crate::data_dir().join("updates");
    let backups_dir = crate::data_dir()
        .join("backups")
        .join(format!("backup_{}", now_stamp()));
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

        // Skip directories; all file paths are re-validated by safe_join().
        if normalized.ends_with('/') {
            continue;
        }

        // Handle binary update
        if normalized == "false-ui.exe"
            || normalized == "false.exe"
            || normalized.ends_with("/false-ui.exe")
        {
            let target = updates_dir.join("false-ui_new.exe");
            let mut out =
                File::create(&target).map_err(|e| format!("Cannot write staged binary: {}", e))?;
            std::io::copy(&mut file, &mut out)
                .map_err(|e| format!("Failed to extract binary: {}", e))?;
            staged_binary = Some(target);
            updated_files.push("false-ui.exe (native binary)".to_string());
            continue;
        }

        // Handle UI update
        let rel_path = if normalized == "index.html" || normalized == "ui/index.html" {
            "index.html".to_string()
        } else if let Some(rest) = normalized.strip_prefix("ui/") {
            rest.to_string()
        } else {
            continue;
        };
        if rel_path.is_empty() {
            continue;
        }
        let target_path = safe_join(&ui_dir, &rel_path)?;

        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Backup existing file if present
        if target_path.exists() {
            let backup_file = safe_join(&backups_dir, &rel_path)?;
            if let Some(parent) = backup_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::copy(&target_path, backup_file);
        }

        let mut out = File::create(&target_path)
            .map_err(|e| format!("Cannot write file {}: {}", target_path.display(), e))?;
        std::io::copy(&mut file, &mut out)
            .map_err(|e| format!("Failed to extract file {}: {}", rel_path, e))?;

        updated_files.push(rel_path);
    }

    if updated_files.is_empty() {
        return Err(
            "No update payload found in .flse package (expected index.html or false-ui.exe)."
                .to_string(),
        );
    }

    // Record the installed version
    let version_file = crate::data_dir().join("version.json");
    let version_record = serde_json::json!({
        "version": manifest.version,
        "name": manifest.name,
        "description": manifest.description,
        "installed_at": now_stamp(),
    });
    let _ = std::fs::write(
        &version_file,
        serde_json::to_string_pretty(&version_record).unwrap_or_default(),
    );

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

fn default_signing_key_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("False")
        .join("signing")
        .join("update-signing.key.pem")
}

fn load_signing_key() -> Result<SigningKey, String> {
    let path = std::env::var_os("FALSE_SIGNING_KEY")
        .map(PathBuf::from)
        .unwrap_or_else(default_signing_key_path);
    let pem = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "Cannot read signing key {}: {}. Run scripts/generate-signing-key.mjs first.",
            path.display(),
            e
        )
    })?;

    // Accept PKCS#8 PEM (Node's export) or a raw base64 32-byte seed.
    let b64: String = pem
        .lines()
        .filter(|line| !line.trim_start().starts_with("-----"))
        .collect();
    let der = B64
        .decode(b64.trim())
        .map_err(|e| format!("Signing key is not valid base64: {}", e))?;
    if der.len() < 32 {
        return Err("Signing key is too short.".to_string());
    }
    let seed: [u8; 32] = der[der.len() - 32..]
        .try_into()
        .map_err(|_| "Signing key seed must be 32 bytes.".to_string())?;
    let key = SigningKey::from_bytes(&seed);

    let expected = decode_public_key()?;
    if key.verifying_key().as_bytes() != expected.as_bytes() {
        return Err(
            "Signing key does not match the committed public key (keys/update-signing.pub)."
                .to_string(),
        );
    }
    Ok(key)
}

/// Creates a new .flse package from the current workspace files.
/// The manifest is signed with the release key before packaging.
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
    let signing_key = load_signing_key()?;
    let signature = signing_key.sign(&manifest_bytes);

    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(&manifest_bytes).map_err(|e| e.to_string())?;

    zip.start_file("manifest.sig", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(format!("{}\n", B64.encode(signature.to_bytes())).as_bytes())
        .map_err(|e| e.to_string())?;

    // 2. Write UI files (index.html, logos)
    let ui_dir = crate::ui_root();
    if let Ok(entries) = std::fs::read_dir(&ui_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let fname = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
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

    zip.finish()
        .map_err(|e| format!("Failed to finalize .flse zip: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_validation_rejects_traversal_and_absolute_paths() {
        assert!(validate_package_relative_path("../evil.html").is_err());
        assert!(validate_package_relative_path("assets/../../evil.html").is_err());
        assert!(validate_package_relative_path("C:/Windows/evil.html").is_err());
        assert!(validate_package_relative_path("/etc/passwd").is_err());
        assert!(validate_package_relative_path("file:stream").is_err());
        assert!(validate_package_relative_path("name. ").is_err());
        assert!(validate_package_relative_path("CON").is_err());
        assert!(validate_package_relative_path("assets/COM1.txt").is_err());
        assert!(validate_package_relative_path("assets/app.js").is_ok());
        assert!(validate_package_relative_path("assets/icons/icon.png").is_ok());
    }

    #[test]
    fn signature_accepts_matching_key_and_rejects_tampering() {
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let manifest = br#"{"name":"False","version":"9.9.9"}"#;
        let signature = key.sign(manifest);
        let encoded = B64.encode(signature.to_bytes());

        assert!(
            verify_manifest_signature_with_key(&key.verifying_key(), manifest, &encoded).is_ok()
        );

        let mut tampered = manifest.to_vec();
        tampered[0] = b'X';
        assert!(
            verify_manifest_signature_with_key(&key.verifying_key(), &tampered, &encoded).is_err()
        );

        assert!(
            verify_manifest_signature_with_key(&key.verifying_key(), manifest, "not-base64!")
                .is_err()
        );
    }

    #[test]
    fn unsigned_package_is_rejected() {
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("manifest.json", options).unwrap();
            zip.write_all(br#"{"name":"x","version":"9.9.9","description":"x"}"#)
                .unwrap();
            zip.finish().unwrap();
        }

        let error = apply_package_bytes(&buffer).unwrap_err();
        assert!(
            error.contains("manifest.sig"),
            "unexpected error: {}",
            error
        );
    }

    #[test]
    fn package_signed_by_another_key_is_rejected() {
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let manifest = br#"{"name":"x","version":"9.9.9","description":"x"}"#;
        let signature = key.sign(manifest);
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("manifest.json", options).unwrap();
            zip.write_all(manifest).unwrap();
            zip.start_file("manifest.sig", options).unwrap();
            zip.write_all(B64.encode(signature.to_bytes()).as_bytes())
                .unwrap();
            zip.start_file("ui/../../evil.html", options).unwrap();
            zip.write_all(b"pwn").unwrap();
            zip.finish().unwrap();
        }

        let error = apply_package_bytes(&buffer).unwrap_err();
        assert!(
            error.contains("signature verification failed"),
            "unexpected error: {}",
            error
        );
    }
}
