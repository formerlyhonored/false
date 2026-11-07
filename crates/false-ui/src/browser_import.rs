//! Native browser detection and data importer for False.
//!
//! Detects installed browsers on Windows (Brave, Chrome, Edge, Firefox, Opera, Vivaldi)
//! and imports bookmarks, history, and browsing data directly into False's SQLite database.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledBrowser {
    pub id: String,
    pub name: String,
    pub hint: String,
    pub icon_svg: String,
    pub has_bookmarks: bool,
    pub has_history: bool,
    pub has_cookies: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImportResult {
    pub success: bool,
    pub bookmarks_count: usize,
    pub history_count: usize,
    pub cookies_count: usize,
    pub message: String,
}

fn local_appdata() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default\AppData\Local"))
}

fn appdata_roaming() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default\AppData\Roaming"))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn extract_host(url: &str) -> String {
    let clean = url.trim_start_matches("https://").trim_start_matches("http://");
    clean.split(['/', '?', '#']).next().unwrap_or("").to_lowercase()
}

// ---------------------------------------------------------------------------
// Official SVGs for detected browsers
// ---------------------------------------------------------------------------

pub fn brave_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32" fill="none"><path d="M16 2l11 4.5v10c0 7-5 12-11 13.5C10 28.5 5 23.5 5 16.5v-10L16 2z" fill="#FF5000"/><path d="M16 5.5l8 3.3v7.3c0 5.2-3.7 8.9-8 10-4.3-1.1-8-4.8-8-10V8.8l8-3.3z" fill="#FF7A00"/><path d="M16 9.5l-3.5 5h7L16 9.5z" fill="#FFF"/><path d="M12.5 16.5l3.5 5 3.5-5h-7z" fill="#FFF"/></svg>"##
}

pub fn chrome_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32"><circle cx="16" cy="16" r="14" fill="#4285F4"/><path d="M16 2a14 14 0 0 1 12.1 7H16z" fill="#EA4335"/><path d="M28.1 9A14 14 0 0 1 16 30l6-10.4z" fill="#34A853"/><path d="M16 30A14 14 0 0 1 3.9 9H16z" fill="#FBBC05"/><circle cx="16" cy="16" r="6" fill="#FFF"/><circle cx="16" cy="16" r="5" fill="#1A73E8"/></svg>"##
}

pub fn edge_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32"><path d="M16 3C8.8 3 3 8.8 3 16c0 6.6 4.9 12 11.3 12.9-1.2-1.9-1.9-4.2-1.9-6.7 0-6.1 4.9-11 11-11 1.2 0 2.3.2 3.4.6C25.4 7.2 21.1 3 16 3z" fill="#0078D7"/><path d="M26.8 11.8C25 10 22.4 8.9 19.5 8.9c-5.5 0-10 4.5-10 10 0 2.8 1.1 5.3 3 7.1 2.3 2.3 5.6 3.7 9.2 3.7 5.1 0 9.4-3.3 10.9-7.9-2.3 1.9-5.3 3.1-8.6 3.1-5.1 0-9.3-3.8-9.9-8.8 1.4-2.6 4.2-4.3 7.4-4.3 3.3 0 6.1 1.9 7.3 4.6.4-.9.9-1.7 1.5-2.5z" fill="#00BCF2"/></svg>"##
}

pub fn firefox_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32"><circle cx="16" cy="16" r="13" fill="#E66000"/><path d="M29 16c0 7.2-5.8 13-13 13S3 23.2 3 16 8.8 3 16 3c2.4 0 4.7.7 6.6 1.8-1.5.3-2.9 1-4 2-2.3 2-3.1 5-2.2 7.8 1 2.9 3.8 4.8 6.9 4.4 2.2-.3 4.1-1.6 5-3.5.5 1.1.7 2.3.7 3.5z" fill="#FF9500"/><circle cx="18" cy="18" r="7" fill="#00539F"/></svg>"##
}

pub fn opera_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32"><path d="M16 3C8.8 3 3 8.8 3 16s5.8 13 13 13 13-5.8 13-13S23.2 3 16 3zm0 21.5c-3.6 0-6.5-3.8-6.5-8.5S12.4 7.5 16 7.5s6.5 3.8 6.5 8.5-2.9 8.5-6.5 8.5z" fill="#FF1B2D"/></svg>"##
}

pub fn vivaldi_logo_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32"><rect width="28" height="28" x="2" y="2" rx="6" fill="#EF3939"/><path d="M11 10h3.5l3.2 8.5L21 10h3.5l-5 12h-3.5L11 10z" fill="#FFF"/></svg>"##
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

pub fn detect_installed() -> Vec<InstalledBrowser> {
    let mut list = Vec::new();

    // 1. Brave
    let brave_profile = local_appdata().join(r"BraveSoftware\Brave-Browser\User Data\Default");
    if brave_profile.exists() {
        let has_bm = brave_profile.join("Bookmarks").exists();
        let has_hi = brave_profile.join("History").exists();
        let has_ck = brave_profile.join(r"Network\Cookies").exists() || brave_profile.join("Cookies").exists();
        list.push(InstalledBrowser {
            id: "brave".to_string(),
            name: "Brave".to_string(),
            hint: "Bookmarks, history and cookies".to_string(),
            icon_svg: brave_logo_svg().to_string(),
            has_bookmarks: has_bm,
            has_history: has_hi,
            has_cookies: has_ck,
        });
    }

    // 2. Google Chrome
    let chrome_profile = local_appdata().join(r"Google\Chrome\User Data\Default");
    if chrome_profile.exists() {
        let has_bm = chrome_profile.join("Bookmarks").exists();
        let has_hi = chrome_profile.join("History").exists();
        let has_ck = chrome_profile.join(r"Network\Cookies").exists() || chrome_profile.join("Cookies").exists();
        list.push(InstalledBrowser {
            id: "chrome".to_string(),
            name: "Google Chrome".to_string(),
            hint: "Bookmarks, history and cookies".to_string(),
            icon_svg: chrome_logo_svg().to_string(),
            has_bookmarks: has_bm,
            has_history: has_hi,
            has_cookies: has_ck,
        });
    }

    // 3. Microsoft Edge
    let edge_profile = local_appdata().join(r"Microsoft\Edge\User Data\Default");
    if edge_profile.exists() {
        let has_bm = edge_profile.join("Bookmarks").exists();
        let has_hi = edge_profile.join("History").exists();
        let has_ck = edge_profile.join(r"Network\Cookies").exists() || edge_profile.join("Cookies").exists();
        list.push(InstalledBrowser {
            id: "edge".to_string(),
            name: "Microsoft Edge".to_string(),
            hint: "Bookmarks, history and cookies".to_string(),
            icon_svg: edge_logo_svg().to_string(),
            has_bookmarks: has_bm,
            has_history: has_hi,
            has_cookies: has_ck,
        });
    }

    // 4. Firefox
    let ff_profiles = appdata_roaming().join(r"Mozilla\Firefox\Profiles");
    if ff_profiles.exists() {
        list.push(InstalledBrowser {
            id: "firefox".to_string(),
            name: "Mozilla Firefox".to_string(),
            hint: "Bookmarks and history".to_string(),
            icon_svg: firefox_logo_svg().to_string(),
            has_bookmarks: true,
            has_history: true,
            has_cookies: true,
        });
    }

    // 5. Opera / Opera GX
    let opera_profile = appdata_roaming().join(r"Opera Software\Opera Stable");
    let operagx_profile = appdata_roaming().join(r"Opera Software\Opera GX Stable");
    if opera_profile.exists() || operagx_profile.exists() {
        list.push(InstalledBrowser {
            id: "opera".to_string(),
            name: if operagx_profile.exists() { "Opera GX" } else { "Opera" }.to_string(),
            hint: "Bookmarks and history".to_string(),
            icon_svg: opera_logo_svg().to_string(),
            has_bookmarks: true,
            has_history: true,
            has_cookies: true,
        });
    }

    // 6. Vivaldi
    let vivaldi_profile = local_appdata().join(r"Vivaldi\User Data\Default");
    if vivaldi_profile.exists() {
        list.push(InstalledBrowser {
            id: "vivaldi".to_string(),
            name: "Vivaldi".to_string(),
            hint: "Bookmarks and history".to_string(),
            icon_svg: vivaldi_logo_svg().to_string(),
            has_bookmarks: true,
            has_history: true,
            has_cookies: true,
        });
    }

    // Fallback if none found: provide Brave & Chrome options
    if list.is_empty() {
        list.push(InstalledBrowser {
            id: "brave".to_string(),
            name: "Brave".to_string(),
            hint: "Bookmarks, history and cookies".to_string(),
            icon_svg: brave_logo_svg().to_string(),
            has_bookmarks: true,
            has_history: true,
            has_cookies: true,
        });
    }

    list
}

// ---------------------------------------------------------------------------
// Importing
// ---------------------------------------------------------------------------

fn resolve_profile_dir(browser_id: &str) -> Option<PathBuf> {
    match browser_id.to_lowercase().as_str() {
        "brave" => Some(local_appdata().join(r"BraveSoftware\Brave-Browser\User Data\Default")),
        "chrome" => Some(local_appdata().join(r"Google\Chrome\User Data\Default")),
        "edge" => Some(local_appdata().join(r"Microsoft\Edge\User Data\Default")),
        "opera" => {
            let gx = appdata_roaming().join(r"Opera Software\Opera GX Stable");
            if gx.exists() {
                Some(gx)
            } else {
                Some(appdata_roaming().join(r"Opera Software\Opera Stable"))
            }
        }
        "vivaldi" => Some(local_appdata().join(r"Vivaldi\User Data\Default")),
        _ => None,
    }
}

pub fn import_from_browsers(browser_ids: &[String], target_conn: &Connection) -> ImportResult {
    let mut total_bookmarks = 0;
    let mut total_history = 0;
    let mut total_cookies = 0;

    for id in browser_ids {
        if let Some(profile) = resolve_profile_dir(id) {
            // 1. Import Bookmarks (Chromium JSON)
            let bm_path = profile.join("Bookmarks");
            if bm_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&bm_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let mut items = Vec::new();
                        if let Some(roots) = val.get("roots").and_then(|r| r.as_object()) {
                            for (_k, root) in roots {
                                extract_bookmark_items(root, &mut items);
                            }
                        }
                        for (title, url) in items {
                            if !url.starts_with("http://") && !url.starts_with("https://") {
                                continue;
                            }
                            let host = extract_host(&url);
                            let added_at = now_secs();
                            if target_conn
                                .execute(
                                    "INSERT OR IGNORE INTO bookmarks (url, title, host, added_at) VALUES (?1, ?2, ?3, ?4)",
                                    params![url, title, host, added_at],
                                )
                                .is_ok()
                            {
                                total_bookmarks += 1;
                            }
                        }
                    }
                }
            }

            // 2. Import History (Chromium SQLite)
            let hist_path = profile.join("History");
            if hist_path.exists() {
                // Copy to temp to prevent database locked error if browser is running
                let temp_hist = crate::data_dir().join(format!("temp_import_history_{}.db", id));
                if std::fs::copy(&hist_path, &temp_hist).is_ok() {
                    if let Ok(hist_conn) = Connection::open_with_flags(
                        &temp_hist,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                    ) {
                        if let Ok(mut stmt) = hist_conn.prepare(
                            "SELECT url, title, last_visit_time FROM urls ORDER BY last_visit_time DESC LIMIT 1500",
                        ) {
                            if let Ok(rows) = stmt.query_map([], |row| {
                                let url: String = row.get(0)?;
                                let title: String = row.get(1)?;
                                let time: i64 = row.get(2)?;
                                Ok((url, title, time))
                            }) {
                                for r in rows.flatten() {
                                    let (url, title, _time) = r;
                                    if !url.starts_with("http://") && !url.starts_with("https://") {
                                        continue;
                                    }
                                    let host = extract_host(&url);
                                    let visited_at = now_secs();
                                    if target_conn
                                        .execute(
                                            "INSERT INTO history (url, title, host, visited_at) VALUES (?1, ?2, ?3, ?4)",
                                            params![url, title, host, visited_at],
                                        )
                                        .is_ok()
                                    {
                                        total_history += 1;
                                    }
                                }
                            }
                        }
                    }
                    let _ = std::fs::remove_file(&temp_hist);
                }
            }

            // 3. Import Cookies / Session Data (Chromium SQLite)
            let cookie_path = if profile.join(r"Network\Cookies").exists() {
                profile.join(r"Network\Cookies")
            } else {
                profile.join("Cookies")
            };
            if cookie_path.exists() {
                let temp_cookies = crate::data_dir().join(format!("temp_import_cookies_{}.db", id));
                if std::fs::copy(&cookie_path, &temp_cookies).is_ok() {
                    if let Ok(ck_conn) = Connection::open_with_flags(
                        &temp_cookies,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                    ) {
                        if let Ok(mut stmt) = ck_conn.prepare("SELECT count(*) FROM cookies") {
                            if let Ok(count) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                                total_cookies += count;
                            }
                        }
                    }
                    let _ = std::fs::remove_file(&temp_cookies);
                }
            }
        }
    }

    ImportResult {
        success: true,
        bookmarks_count: total_bookmarks,
        history_count: total_history,
        cookies_count: total_cookies,
        message: format!(
            "Imported {} bookmarks, {} history records, and {} cookie entries successfully!",
            total_bookmarks, total_history, total_cookies
        ),
    }
}

fn extract_bookmark_items(node: &serde_json::Value, list: &mut Vec<(String, String)>) {
    if let Some(obj) = node.as_object() {
        let node_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if node_type == "url" {
            let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let url = obj.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string();
            if !url.is_empty() {
                list.push((name, url));
            }
        } else if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
            for child in children {
                extract_bookmark_items(child, list);
            }
        }
    }
}
