//! Discord Rich Presence client for False.
//!
//! Intentionally only publishes app-level status text (e.g. "Using False", "Browsing with False")
//! and NEVER tab titles, visited URLs, or browsing history.
//!
//! Config lives in `%LOCALAPPDATA%\False\discord.json`.
//! Can be toggled on/off dynamically from False Settings.

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default application ID provided for False
pub const DEFAULT_CLIENT_ID: &str = "1568204261509046323";

/// False's official star logo hosted on Catbox for Discord RPC
pub const DEFAULT_LOGO: &str = "https://files.catbox.moe/922005.jpg";

static RPC_RUNNING: AtomicBool = AtomicBool::new(false);
static RPC_ENABLED: AtomicBool = AtomicBool::new(true);
static CURRENT_STATUS: Mutex<Option<String>> = Mutex::new(None);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub client_id: String,
    pub details: String,
    pub state: String,
    pub large_image: String,
    pub large_text: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            client_id: DEFAULT_CLIENT_ID.to_string(),
            details: "Using False".to_string(),
            state: "Fast & Clean".to_string(),
            large_image: DEFAULT_LOGO.to_string(),
            large_text: "False".to_string(),
        }
    }
}

pub fn config_path() -> PathBuf {
    crate::data_dir().join("discord.json")
}

pub fn load_config() -> DiscordConfig {
    let path = config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<DiscordConfig>(&text) {
            return cfg;
        }
    }

    let default_cfg = DiscordConfig::default();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
        let _ = std::fs::write(&path, json);
    }
    default_cfg
}

pub fn save_config(cfg: &DiscordConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn is_connected() -> bool {
    RPC_RUNNING.load(Ordering::Relaxed)
}

pub fn is_enabled() -> bool {
    RPC_ENABLED.load(Ordering::Relaxed)
}

pub fn set_enabled(enabled: bool) {
    RPC_ENABLED.store(enabled, Ordering::Relaxed);
    let mut cfg = load_config();
    cfg.enabled = enabled;
    let _ = save_config(&cfg);
    if !enabled {
        RPC_RUNNING.store(false, Ordering::Relaxed);
        crate::log_debug("Discord RPC disabled via Settings");
    } else {
        crate::log_debug("Discord RPC enabled via Settings");
    }
}

pub fn current_status() -> String {
    if let Ok(lock) = CURRENT_STATUS.lock() {
        if let Some(s) = &*lock {
            return s.clone();
        }
    }
    "Idle".to_string()
}

pub fn start() {
    let mut cfg = load_config();

    // Environment variable overrides (if provided)
    if let Ok(id) = std::env::var("FALSE_DISCORD_CLIENT_ID") {
        if !id.trim().is_empty() {
            cfg.client_id = id;
        }
    }
    if let Ok(img) = std::env::var("FALSE_DISCORD_LARGE_IMAGE") {
        if !img.trim().is_empty() {
            cfg.large_image = img;
        }
    }
    if std::env::var("FALSE_DISCORD_DISABLE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        cfg.enabled = false;
    }

    RPC_ENABLED.store(cfg.enabled, Ordering::Relaxed);

    // Fallback client_id if empty
    if cfg.client_id.trim().is_empty() {
        cfg.client_id = DEFAULT_CLIENT_ID.to_string();
    }
    if cfg.large_image.trim().is_empty() {
        cfg.large_image = DEFAULT_LOGO.to_string();
    }
    if cfg.large_text.trim().is_empty() {
        cfg.large_text = "False".to_string();
    }

    let _ = std::thread::Builder::new()
        .name("false-discord-rpc".into())
        .spawn(move || run_rpc_loop(cfg));
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn run_rpc_loop(cfg: DiscordConfig) {
    let start_timestamp = now_secs();
    crate::log_debug(&format!(
        "Discord RPC thread started for client_id: {}",
        cfg.client_id
    ));

    loop {
        // If disabled, wait quietly without connecting
        if !RPC_ENABLED.load(Ordering::Relaxed) {
            RPC_RUNNING.store(false, Ordering::Relaxed);
            std::thread::sleep(Duration::from_secs(2));
            continue;
        }

        match DiscordIpcClient::new(&cfg.client_id) {
            Ok(mut client) => {
                match client.connect() {
                    Ok(_) => {
                        RPC_RUNNING.store(true, Ordering::Relaxed);
                        crate::log_debug("Discord RPC connected successfully");

                        if let Ok(mut lock) = CURRENT_STATUS.lock() {
                            *lock = Some(cfg.details.clone());
                        }

                        // Build activity
                        let timestamps = activity::Timestamps::new().start(start_timestamp);
                        let assets = activity::Assets::new()
                            .large_image(&cfg.large_image)
                            .large_text(&cfg.large_text);

                        let mut act = activity::Activity::new()
                            .details(&cfg.details)
                            .timestamps(timestamps)
                            .assets(assets);

                        if !cfg.state.trim().is_empty() {
                            act = act.state(&cfg.state);
                        }

                        if let Err(err) = client.set_activity(act) {
                            crate::log_debug(&format!("Discord RPC set_activity error: {}", err));
                            RPC_RUNNING.store(false, Ordering::Relaxed);
                            let _ = client.close();
                        } else {
                            // Maintain connection alive with periodic refresh
                            loop {
                                std::thread::sleep(Duration::from_secs(3));

                                // Check if user turned it off in settings
                                if !RPC_ENABLED.load(Ordering::Relaxed) {
                                    let _ = client.close();
                                    RPC_RUNNING.store(false, Ordering::Relaxed);
                                    break;
                                }

                                // Periodic re-assert every 30s
                                std::thread::sleep(Duration::from_secs(27));

                                if !RPC_ENABLED.load(Ordering::Relaxed) {
                                    let _ = client.close();
                                    RPC_RUNNING.store(false, Ordering::Relaxed);
                                    break;
                                }

                                let timestamps = activity::Timestamps::new().start(start_timestamp);
                                let assets = activity::Assets::new()
                                    .large_image(&cfg.large_image)
                                    .large_text(&cfg.large_text);

                                let mut act = activity::Activity::new()
                                    .details(&cfg.details)
                                    .timestamps(timestamps)
                                    .assets(assets);

                                if !cfg.state.trim().is_empty() {
                                    act = act.state(&cfg.state);
                                }

                                if let Err(err) = client.set_activity(act) {
                                    crate::log_debug(&format!(
                                        "Discord RPC heartbeat error: {}",
                                        err
                                    ));
                                    RPC_RUNNING.store(false, Ordering::Relaxed);
                                    let _ = client.close();
                                    break;
                                }
                            }
                        }
                    }
                    Err(_err) => {
                        RPC_RUNNING.store(false, Ordering::Relaxed);
                        // Discord not running or pipe busy; sleep before retry
                        std::thread::sleep(Duration::from_secs(15));
                    }
                }
            }
            Err(err) => {
                crate::log_debug(&format!("Failed to initialize Discord IPC client: {}", err));
                std::thread::sleep(Duration::from_secs(30));
            }
        }
    }
}
