pub mod discord;
pub mod flse;
pub mod browser_import;
pub mod installer_ui;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{
    webview::Color, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Rect,
    Size, State, WebviewBuilder, WebviewUrl,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ---------------------------------------------------------------------------
// Logging + paths
// ---------------------------------------------------------------------------

pub(crate) fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("False")
}

fn log_path() -> PathBuf {
    data_dir().join("false.log")
}

fn log_debug(msg: &str) {
    let path = log_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[DEBUG] {}", msg);
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Page snapshots (used as a frozen backdrop while shell popovers are open)
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod page_capture {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size_image: u32,
        x_pels_per_meter: i32,
        y_pels_per_meter: i32,
        clr_used: u32,
        clr_important: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }

    type Hwnd = *mut c_void;
    type Hdc = *mut c_void;
    type HBitmap = *mut c_void;
    type Hgdiobj = *mut c_void;

    #[link(name = "user32")]
    extern "system" {
        fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn ReleaseDC(hwnd: Hwnd, hdc: Hdc) -> i32;
        fn PrintWindow(hwnd: Hwnd, hdc: Hdc, flags: u32) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
        fn CreateDIBSection(
            hdc: Hdc,
            bmi: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: *mut c_void,
            offset: u32,
        ) -> HBitmap;
        fn SelectObject(hdc: Hdc, obj: Hgdiobj) -> Hgdiobj;
        fn DeleteObject(obj: Hgdiobj) -> i32;
        fn DeleteDC(hdc: Hdc) -> i32;
    }

    /// Capture a window's pixels with PrintWindow(PW_RENDERFULLCONTENT) and
    /// return tightly packed RGBA bytes. WebView2 re-renders its surface for
    /// WM_PRINT, so this works even when another window overlaps ours.
    pub fn capture(hwnd: isize) -> Result<(u32, u32, Vec<u8>), String> {
        if hwnd == 0 {
            return Err("webview has no window handle".into());
        }
        unsafe {
            let hwnd = hwnd as Hwnd;
            let mut rect = Rect::default();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return Err("GetWindowRect failed".into());
            }
            let width = (rect.right - rect.left).max(1) as u32;
            let height = (rect.bottom - rect.top).max(1) as u32;
            if width > 16_384 || height > 16_384 {
                return Err("webview window is absurdly large".into());
            }

            let screen_dc = GetDC(hwnd);
            if screen_dc.is_null() {
                return Err("GetDC failed".into());
            }
            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_null() {
                ReleaseDC(hwnd, screen_dc);
                return Err("CreateCompatibleDC failed".into());
            }

            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                    width: width as i32,
                    height: -(height as i32), // top-down rows
                    planes: 1,
                    bit_count: 32,
                    compression: 0, // BI_RGB
                    ..Default::default()
                },
                colors: [0],
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(
                screen_dc,
                &info,
                0, // DIB_RGB_COLORS
                &mut bits,
                std::ptr::null_mut(),
                0,
            );
            if bitmap.is_null() || bits.is_null() {
                DeleteDC(mem_dc);
                ReleaseDC(hwnd, screen_dc);
                return Err("CreateDIBSection failed".into());
            }

            let old = SelectObject(mem_dc, bitmap as Hgdiobj);
            let rendered = PrintWindow(hwnd, mem_dc, 2 /* PW_RENDERFULLCONTENT */);
            SelectObject(mem_dc, old);

            // The DIB pixels live in the bitmap; copy them before freeing it.
            let copied = if rendered != 0 {
                let bgra =
                    std::slice::from_raw_parts(bits as *const u8, (width * height * 4) as usize);
                let mut rgba = Vec::with_capacity(bgra.len());
                for px in bgra.as_chunks::<4>().0 {
                    rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
                }
                Ok((width, height, rgba))
            } else {
                Err("PrintWindow failed".into())
            };

            DeleteObject(bitmap as Hgdiobj);
            DeleteDC(mem_dc);
            ReleaseDC(hwnd, screen_dc);
            copied
        }
    }
}

fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

#[cfg(windows)]
#[tauri::command]
async fn capture_tab(app: AppHandle, tab_id: u32) -> Result<String, String> {
    use base64::Engine as _;

    let wv = app
        .get_webview(&tab_label(tab_id))
        .ok_or_else(|| "tab is not showing a live page".to_string())?;

    let (tx, rx) = std::sync::mpsc::channel();
    wv.with_webview(move |platform| {
        let controller = platform.controller();
        let mut hwnd = Default::default();
        let result = match unsafe { controller.ParentWindow(&mut hwnd) } {
            Ok(()) => page_capture::capture(hwnd.0 as isize),
            Err(err) => Err(format!("ParentWindow failed: {}", err)),
        };
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    let (width, height, rgba) = rx
        .recv_timeout(Duration::from_millis(1500))
        .map_err(|_| "page capture timed out".to_string())??;
    let png = encode_png(width, height, &rgba)?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

#[cfg(not(windows))]
#[tauri::command]
async fn capture_tab(_app: AppHandle, _tab_id: u32) -> Result<String, String> {
    Err("page capture is only supported on Windows".into())
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageLink {
    pub text: String,
    pub href: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageSnapshot {
    pub url: String,
    pub title: String,
    pub text: String,
    #[serde(default)]
    pub headings: Vec<String>,
    #[serde(default)]
    pub links: Vec<PageLink>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub host: String,
    pub visited_at: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct BookmarkEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub host: String,
    pub added_at: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct BlockEntry {
    pub host: String,
    pub at: i64,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub color: String,
    pub created_at: i64,
}

/// Every tab gets its own native child webview (`web_<id>`), so tabs keep
/// independent history, cookies, scroll position and page state.
fn tab_label(tab_id: u32) -> String {
    format!("web_{}", tab_id)
}

fn parse_hex_color(value: &str) -> Color {
    let h = value.trim().trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        Color(r, g, b, 255)
    } else {
        Color(0, 0, 0, 255)
    }
}

fn host_of(url: &str) -> String {
    tauri::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// SQLite persistence
// ---------------------------------------------------------------------------

struct Db {
    conn: Mutex<Connection>,
}

fn apply_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS profiles (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            email      TEXT NOT NULL DEFAULT '',
            avatar     TEXT NOT NULL DEFAULT '',
            color      TEXT NOT NULL DEFAULT '#3b82f6',
            created_at INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO profiles (id, name, email, avatar, color, created_at)
        VALUES ('personal', 'Personal', '', '', '#3b82f6', 1700000000);
        INSERT OR IGNORE INTO profiles (id, name, email, avatar, color, created_at)
        VALUES ('work', 'Work', '', '', '#10b981', 1700000001);
        CREATE TABLE IF NOT EXISTS history (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            url        TEXT NOT NULL,
            title      TEXT NOT NULL DEFAULT '',
            host       TEXT NOT NULL DEFAULT '',
            visited_at INTEGER NOT NULL,
            profile_id TEXT NOT NULL DEFAULT 'personal'
        );
        CREATE INDEX IF NOT EXISTS idx_history_visited ON history(visited_at DESC);
        CREATE INDEX IF NOT EXISTS idx_history_host ON history(host);
        CREATE TABLE IF NOT EXISTS bookmarks (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            url        TEXT NOT NULL UNIQUE,
            title      TEXT NOT NULL DEFAULT '',
            host       TEXT NOT NULL DEFAULT '',
            added_at   INTEGER NOT NULL,
            profile_id TEXT NOT NULL DEFAULT 'personal'
        );
        CREATE TABLE IF NOT EXISTS adblock_allow (
            host TEXT PRIMARY KEY
        );
        "#,
    )?;
    let _ = conn.execute("ALTER TABLE history ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'personal'", []);
    let _ = conn.execute("ALTER TABLE bookmarks ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'personal'", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_history_profile ON history(profile_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_bookmarks_profile ON bookmarks(profile_id)", []);
    Ok(())
}

fn open_db() -> Result<Db, String> {
    let _ = std::fs::create_dir_all(data_dir());
    let conn = Connection::open(data_dir().join("false.db"))
        .or_else(|_| Connection::open_in_memory())
        .map_err(|e| format!("Could not open database: {}", e))?;
    apply_schema(&conn).map_err(|e| format!("Could not prepare database: {}", e))?;
    Ok(Db {
        conn: Mutex::new(conn),
    })
}

fn load_allow_hosts(db: &Db) -> HashSet<String> {
    let mut out = HashSet::new();
    let Ok(conn) = db.conn.lock() else {
        return out;
    };
    let Ok(mut stmt) = conn.prepare("SELECT host FROM adblock_allow") else {
        return out;
    };
    if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
        for row in rows.flatten() {
            out.insert(row);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Ad / tracker blocking proxy
// ---------------------------------------------------------------------------

const BLOCK_HOSTS: &[&str] = &[
    "2mdn.net",
    "33across.com",
    "3lift.com",
    "aax.amazon-adsystem.com",
    "ad-delivery.net",
    "ad.doubleclick.net",
    "adcolony.com",
    "adform.net",
    "adition.com",
    "adnxs-simple.com",
    "adnxs.com",
    "adroll.com",
    "ads.linkedin.com",
    "ads.twitter.com",
    "adsafeprotected.com",
    "adservice.google.com",
    "adsrvr.org",
    "adswizz.com",
    "advertising.com",
    "adzerk.net",
    "amazon-adsystem.com",
    "analytics.google.com",
    "analytics.tiktok.com",
    "analytics.twitter.com",
    "app-measurement.com",
    "appnexus.com",
    "atdmt.com",
    "bat.bing.com",
    "bidswitch.net",
    "bluekai.com",
    "branch.io",
    "c.amazon-adsystem.com",
    "casalemedia.com",
    "chartbeat.com",
    "cloudfront-labs.amazonaws.com",
    "connect.facebook.net",
    "criteo.com",
    "criteo.net",
    "crwdcntrl.net",
    "demdex.net",
    "dotomi.com",
    "doubleclick.net",
    "doubleverify.com",
    "exelator.com",
    "facebook.net",
    "fastclick.net",
    "firebaseinstallations.googleapis.com",
    "flashtalking.com",
    "google-analytics.com",
    "googleadservices.com",
    "googlesyndication.com",
    "googletagmanager.com",
    "googletagservices.com",
    "gstaticadssl.com",
    "hotjar.com",
    "hubspot.com",
    "ib.adnxs.com",
    "imrworldwide.com",
    "inmobi.com",
    "inspectlet.com",
    "intercom.io",
    "krxd.net",
    "licdn.com",
    "loggly.com",
    "marketo.net",
    "mathtag.com",
    "mc.yandex.ru",
    "media.net",
    "mixpanel.com",
    "moatads.com",
    "mookie1.com",
    "mxpnl.com",
    "newrelic.com",
    "nr-data.net",
    "omtrdc.net",
    "openx.net",
    "outbrain.com",
    "pagead2.googlesyndication.com",
    "permutive.com",
    "pinterest.com/ct",
    "pixel.facebook.com",
    "popads.net",
    "propellerads.com",
    "pubmatic.com",
    "quantserve.com",
    "rlcdn.com",
    "rubiconproject.com",
    "scorecardresearch.com",
    "segment.com",
    "segment.io",
    "serving-sys.com",
    "sharethrough.com",
    "siftscience.com",
    "smartadserver.com",
    "smaato.net",
    "spotxchange.com",
    "spotx.tv",
    "statcounter.com",
    "static.ads-twitter.com",
    "taboola.com",
    "teads.tv",
    "tremorhub.com",
    "tribalfusion.com",
    "turn.com",
    "tynt.com",
    "underscorejs.org",
    "unityads.unity3d.com",
    "vindicosuite.com",
    "visualwebsiteoptimizer.com",
    "weborama.fr",
    "yieldmo.com",
    "zedo.com",
    "zqtk.net",
];

#[allow(dead_code)]
struct AdBlock {
    enabled: AtomicBool,
    blocked: AtomicU64,
    allowed: AtomicU64,
    allow: RwLock<HashSet<String>>,
    block: RwLock<HashSet<String>>,
    recent: Mutex<VecDeque<BlockEntry>>,
}

impl AdBlock {
    fn new() -> Self {
        let block: HashSet<String> = BLOCK_HOSTS.iter().map(|s| s.to_string()).collect();
        Self {
            enabled: AtomicBool::new(true),
            blocked: AtomicU64::new(0),
            allowed: AtomicU64::new(0),
            allow: RwLock::new(HashSet::new()),
            block: RwLock::new(block),
            recent: Mutex::new(VecDeque::new()),
        }
    }

    fn empty() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            blocked: AtomicU64::new(0),
            allowed: AtomicU64::new(0),
            allow: RwLock::new(HashSet::new()),
            block: RwLock::new(HashSet::new()),
            recent: Mutex::new(VecDeque::new()),
        }
    }

    fn info(&self) -> serde_json::Value {
        let allow: Vec<String> = self
            .allow
            .read()
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        let recent: Vec<BlockEntry> = self
            .recent
            .lock()
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default();
        serde_json::json!({
            "enabled": self.enabled.load(Ordering::Relaxed),
            "blocked": self.blocked.load(Ordering::Relaxed),
            "allowed": self.allowed.load(Ordering::Relaxed),
            "allow": allow,
            "recent": recent,
        })
    }

    fn is_blocked(&self, host: &str) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        let h = normalize_host(host);
        if h.is_empty() {
            return false;
        }
        if let Ok(allow) = self.allow.read() {
            if allow.iter().any(|d| domain_match(&h, d)) {
                return false;
            }
        }
        if let Ok(block) = self.block.read() {
            if block.iter().any(|d| domain_match(&h, d)) {
                return true;
            }
        }
        false
    }

    fn record_block(&self, host: &str, kind: &str) {
        self.blocked.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut recent) = self.recent.lock() {
            recent.push_front(BlockEntry {
                host: host.to_string(),
                at: now_secs(),
                kind: kind.to_string(),
            });
            if recent.len() > 60 {
                recent.truncate(60);
            }
        }
        log_debug(&format!("blocked {} {}", kind, host));
    }

    fn record_allowed(&self) {
        self.allowed.fetch_add(1, Ordering::Relaxed);
    }
}

fn normalize_host(h: &str) -> String {
    let mut s = h.trim().to_ascii_lowercase();
    if s.starts_with('[') {
        if let Some(end) = s.find(']') {
            s = s[1..end].to_string();
        }
    } else if let Some((host, port)) = s.rsplit_once(':') {
        if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            s = host.to_string();
        }
    }
    s.trim_end_matches('.').to_string()
}

#[allow(dead_code)]
fn domain_match(host: &str, domain: &str) -> bool {
    host == domain
        || (host.len() > domain.len()
            && host.ends_with(domain)
            && host.as_bytes()[host.len() - domain.len() - 1] == b'.')
}

#[allow(dead_code)]
fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

#[allow(dead_code)]
async fn read_head(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(2048);
    let mut tmp = [0u8; 2048];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if find_header_end(&buf).is_some() || buf.len() > 128 * 1024 {
            break;
        }
    }
    Ok(buf)
}

#[allow(dead_code)]
async fn proxy_respond(stream: &mut TcpStream, status: &str, body: &str) {
    let msg = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    let _ = stream.write_all(msg.as_bytes()).await;
}

#[allow(dead_code)]
fn header_host(head: &str) -> Option<String> {
    for line in head.split("\r\n").skip(1) {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("host") {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
async fn handle_proxy_client(mut client: TcpStream, ad: Arc<AdBlock>) {

    let _ = client.set_nodelay(true);
    let head = match read_head(&mut client).await {
        Ok(h) => h,
        Err(_) => return,
    };
    if head.is_empty() {
        return;
    }
    let Some(header_end) = find_header_end(&head) else {
        proxy_respond(&mut client, "400 Bad Request", "bad request").await;
        return;
    };
    let head_bytes = &head[..header_end];
    let tail = &head[header_end..];
    let head_text = String::from_utf8_lossy(head_bytes).to_string();

    let mut first_line = head_text.split("\r\n").next().unwrap_or("").split_whitespace();
    let method = first_line.next().unwrap_or("").to_string();
    let target = first_line.next().unwrap_or("").to_string();
    let version = first_line.next().unwrap_or("HTTP/1.1").to_string();
    if method.is_empty() || target.is_empty() {
        proxy_respond(&mut client, "400 Bad Request", "bad request").await;
        return;
    }

    // HTTPS (and any CONNECT tunnel): we can only filter by hostname.
    if method.eq_ignore_ascii_case("CONNECT") {
        let host = normalize_host(target.rsplit_once(':').map(|(h, _)| h).unwrap_or(&target));
        if ad.is_blocked(&host) {
            ad.record_block(&host, "https");
            proxy_respond(&mut client, "403 Forbidden", "blocked by False").await;
            return;
        }
        let mut upstream = match TcpStream::connect(&target).await {
            Ok(s) => s,
            Err(_) => {
                proxy_respond(&mut client, "502 Bad Gateway", "could not connect").await;
                return;
            }
        };
        ad.record_allowed();
        if client
            .write_all(b"HTTP/1.1 200 Connection Established\r\nProxy-Agent: False\r\n\r\n")
            .await
            .is_err()
        {
            return;
        }
        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
        return;
    }

    // Plain HTTP: resolve the host from the absolute request URI or Host header.
    let parsed = tauri::Url::parse(&target).ok();
    let host = parsed
        .as_ref()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
        .or_else(|| header_host(&head_text).map(|h| normalize_host(&h)))
        .unwrap_or_default();
    if host.is_empty() {
        proxy_respond(&mut client, "400 Bad Request", "missing host").await;
        return;
    }
    if ad.is_blocked(&host) {
        ad.record_block(&host, "http");
        proxy_respond(&mut client, "403 Forbidden", "blocked by False").await;
        return;
    }
    let port = parsed
        .as_ref()
        .and_then(|u| u.port_or_known_default())
        .unwrap_or(80);
    let mut upstream = match TcpStream::connect(format!("{}:{}", host, port)).await {
        Ok(s) => s,
        Err(_) => {
            proxy_respond(&mut client, "502 Bad Gateway", "could not connect").await;
            return;
        }
    };
    ad.record_allowed();

    let path = if let Some(url) = &parsed {
        let mut p = url.path().to_string();
        if let Some(q) = url.query() {
            p.push('?');
            p.push_str(q);
        }
        p
    } else {
        target
    };
    let rest = head_text
        .split_once("\r\n")
        .map(|(_, r)| r)
        .unwrap_or("");
    let request = format!("{} {} {}\r\n{}", method, path, version, rest);
    if upstream.write_all(request.as_bytes()).await.is_err() {
        return;
    }
    if !tail.is_empty() && upstream.write_all(tail).await.is_err() {
        return;
    }
    let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
}

#[allow(dead_code)]
fn start_proxy(ad: Arc<AdBlock>) -> Option<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    listener.set_nonblocking(true).ok()?;
    std::thread::Builder::new()
        .name("false-adblock".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log_debug(&format!("proxy runtime failed: {}", e));
                    return;
                }
            };
            rt.block_on(async move {
                let listener = match tokio::net::TcpListener::from_std(listener) {
                    Ok(l) => l,
                    Err(e) => {
                        log_debug(&format!("proxy listener failed: {}", e));
                        return;
                    }
                };
                log_debug(&format!("adblock proxy listening on 127.0.0.1:{}", port));
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let ad = ad.clone();
                            tokio::spawn(async move { handle_proxy_client(stream, ad).await });
                        }
                        Err(_) => tokio::time::sleep(Duration::from_millis(40)).await,
                    }
                }
            });
        })
        .ok()?;
    Some(port)
}

// ---------------------------------------------------------------------------
// Injected page script
// ---------------------------------------------------------------------------

fn init_script() -> &'static str {
    r#"
    (function () {
        // Calm & Instant Cosmetic Ad / Cookie Consent Banner Blocker
        try {
            var calmCss = [
                'ins.adsbygoogle',
                '.adsbygoogle',
                '.ad-container',
                '.ad-box',
                '.ad-placement',
                '.banner-ad',
                '[id^="google_ads_"]',
                '[id^="aswift_"]',
                '#onetrust-banner-sdk',
                '#onetrust-consent-sdk',
                '.cookie-banner',
                '.cookie-consent',
                '.truste_box_overlay',
                '.cmpbox',
                '#cookie-law-info-bar',
                '.qc-cmp2-container',
                '#didomi-host'
            ].join(',') + '{display:none!important;visibility:hidden!important;opacity:0!important;pointer-events:none!important;height:0!important;min-height:0!important;}';

            var calmStyle = document.createElement('style');
            calmStyle.id = '__false_calm_blocker';
            calmStyle.textContent = calmCss;
            (document.head || document.documentElement).appendChild(calmStyle);

            function calmClean() {
                var banners = document.querySelectorAll('#onetrust-banner-sdk, #onetrust-consent-sdk, .cookie-banner, .cookie-consent, .qc-cmp2-container, #didomi-host');
                for (var i = 0; i < banners.length; i++) {
                    try { banners[i].remove(); } catch(e){}
                }
                if (document.body && document.body.style.overflow === 'hidden' && document.querySelectorAll('.modal, .dialog').length === 0) {
                    document.body.style.overflow = '';
                }
            }
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', calmClean);
            } else {
                calmClean();
            }
            var calmObs = new MutationObserver(function () {
                calmClean();
            });
            calmObs.observe(document.documentElement, { childList: true, subtree: true });
            setTimeout(function() { calmObs.disconnect(); }, 8000);
        } catch (e) {}

        // Browser shortcuts must keep working while a page (not the shell) has
        // focus. We cannot use Tauri IPC from a remote origin, so send a
        // sentinel navigation that the host recognises and cancels.
        try {
            function shortcutId(e) {
                var ctrl = e.ctrlKey || e.metaKey;
                var k = (e.key || '').toLowerCase();
                if (e.key === 'F5') return 'reload';
                if (e.key === 'F12') return 'devtools';
                if (e.altKey && e.key === 'ArrowLeft') return 'back';
                if (e.altKey && e.key === 'ArrowRight') return 'forward';
                if (!ctrl) return null;
                if (k === 'tab') return e.shiftKey ? 'prev-tab' : 'next-tab';
                if (e.shiftKey && k === 't') return 'reopen';
                if (e.shiftKey && k === 'r') return 'reload';
                if (e.shiftKey && k === 'b') return 'toggle-bookmarks-bar';
                if (k === 't') return 'new-tab';
                if (k === 'w') return 'close-tab';
                if (k === 'l') return 'focus-omni';
                if (k === 'd') return 'bookmark';
                if (k === 'h') return 'history';
                if (k === 'f') return 'find';
                if (k === 'r') return 'reload';
                if (k === 'b') return 'toggle-side';
                if (k === 'j') return 'agents';
                if (k === ',') return 'settings';
                if (k === '+') return 'zoom-in';
                if (k === '=') return 'zoom-in';
                if (k === '-') return 'zoom-out';
                if (k === '_') return 'zoom-out';
                if (k === '0') return 'zoom-reset';
                if (/^[1-9]$/.test(k)) return 'tab-' + k;
                return null;
            }
            document.addEventListener('keydown', function (e) {
                var id = shortcutId(e);
                if (!id) return;
                e.preventDefault();
                e.stopPropagation();
                try { window.location.href = 'https://false.invalid/__false/key/' + id; } catch (err) {}
            }, true);
        } catch (e) {}

        // Report in-page (SPA) URL changes so the shell address bar stays live.
        try {
            function reportUrl() {
                var u = location.href;
                if (!/^https?:/i.test(u)) return;
                if (window.__falseUrl === u) return;
                window.__falseUrl = u;
                try {
                    window.location.href = 'https://false.invalid/__false/url/' + encodeURIComponent(u);
                } catch (err) {}
            }
            var push = history.pushState;
            var replace = history.replaceState;
            history.pushState = function () {
                var r = push.apply(this, arguments);
                setTimeout(reportUrl, 0);
                return r;
            };
            history.replaceState = function () {
                var r = replace.apply(this, arguments);
                setTimeout(reportUrl, 0);
                return r;
            };
            window.addEventListener('popstate', function () { setTimeout(reportUrl, 0); });
        } catch (e) {}

        // Let the host know when a single-page app changes its own title.
        try {
            var lastTitle = document.title;
            setInterval(function () {
                if (document.title !== lastTitle) {
                    lastTitle = document.title;
                    document.dispatchEvent(new CustomEvent('false:title', { detail: lastTitle }));
                }
            }, 800);
        } catch (e) {}
    })();
    "#
}

// ---------------------------------------------------------------------------
// Webview helpers
// ---------------------------------------------------------------------------

fn main_window(app: &AppHandle) -> Result<tauri::Window, String> {
    app.get_window("main").ok_or_else(|| "main window not found".into())
}

/// Child webviews only (the UI itself is the `main` webview).
fn tab_webviews(app: &AppHandle) -> Vec<tauri::Webview> {
    main_window(app)
        .map(|w| {
            w.webviews()
                .into_iter()
                .filter(|v| v.label().starts_with("web_"))
                .collect()
        })
        .unwrap_or_default()
}

fn hide_other_webviews(app: &AppHandle, keep: Option<u32>) {
    let keep_label = keep.map(tab_label);
    for wv in tab_webviews(app) {
        if Some(wv.label().to_string()) != keep_label {
            let _ = wv.hide();
        }
    }
}

fn apply_bounds(wv: &tauri::Webview, bounds: &WebBounds) {
    let _ = wv.set_bounds(Rect {
        position: Position::Logical(LogicalPosition::new(bounds.x, bounds.y)),
        size: Size::Logical(LogicalSize::new(bounds.width, bounds.height)),
    });
}

fn builder_for(
    app: &AppHandle,
    tab_id: u32,
    url: tauri::Url,
    bg: Color,
    proxy: Option<tauri::Url>,
) -> WebviewBuilder<tauri::Wry> {
    let label = tab_label(tab_id);
    let app_handle = app.clone();

    let mut builder = WebviewBuilder::new(label, WebviewUrl::External(url))
        .initialization_script(init_script())
        .background_color(bg);
    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
        builder = builder.devtools(true);
    }
    if let Some(p) = proxy {
        builder = builder.proxy_url(p);
    }

    builder
        .on_document_title_changed({
            let app = app_handle.clone();
            move |_wv, title| {
                let _ = app.emit(
                    "tab-title-changed",
                    serde_json::json!({ "id": tab_id, "title": title }),
                );
            }
        })
        .on_navigation({
            let app = app_handle.clone();
            move |url| {
                let u = url.to_string();
                // Shortcut channel from pages: the script navigates to a
                // sentinel origin which we cancel, emitting the key to the shell.
                const KEY_PREFIX: &str = "https://false.invalid/__false/key/";
                if let Some(key) = u.strip_prefix(KEY_PREFIX) {
                    let _ = app.emit(
                        "false-key",
                        serde_json::json!({ "key": key.to_string() }),
                    );
                    return false;
                }
                // Live URL channel for single-page apps (also cancelled).
                const URL_PREFIX: &str = "https://false.invalid/__false/url/";
                if let Some(encoded) = u.strip_prefix(URL_PREFIX) {
                    let _ = app.emit(
                        "false-url",
                        serde_json::json!({ "id": tab_id, "url": encoded.to_string() }),
                    );
                    return false;
                }
                let _ = app.emit(
                    "tab-url-changed",
                    serde_json::json!({ "id": tab_id, "url": u }),
                );
                true
            }
        })
        .on_page_load({
            let app = app_handle.clone();
            move |_wv, payload| {
                let u = payload.url().to_string();
                let status = match payload.event() {
                    tauri::webview::PageLoadEvent::Started => "started",
                    tauri::webview::PageLoadEvent::Finished => "finished",
                };
                let _ = app.emit(
                    "tab-page-load",
                    serde_json::json!({ "id": tab_id, "url": u, "status": status }),
                );
            }
        })
}

// ---------------------------------------------------------------------------
// Tab / webview commands
// ---------------------------------------------------------------------------

/// Create (if needed), position and show a tab's webview. `force` navigates
/// even when the webview is already on that URL (used when the user hits
/// Enter in the address bar).
#[tauri::command]
async fn show_webview(
    app: AppHandle,
    tab_id: u32,
    url: String,
    bounds: WebBounds,
    force: Option<bool>,
    background: Option<String>,
    keep_others: Option<bool>,
) -> Result<(), String> {
    let target: tauri::Url = url
        .parse()
        .map_err(|e| format!("Invalid URL '{}': {}", url, e))?;
    let bg = parse_hex_color(background.as_deref().unwrap_or("#000000"));
    let keep_others = keep_others.unwrap_or(false);
    let proxy = app
        .try_state::<ProxyInfo>()
        .and_then(|p| p.url.parse::<tauri::Url>().ok());

    if !keep_others {
        hide_other_webviews(&app, Some(tab_id));
    }

    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        apply_bounds(&wv, &bounds);
        let current = wv.url().ok().map(|u| u.to_string());
        let same = current.as_deref() == Some(target.as_str());
        if !same || force.unwrap_or(false) {
            wv.navigate(target).map_err(|e| e.to_string())?;
        }
        if !keep_others {
            let _ = wv.show();
            let _ = wv.set_focus();
        }
        return Ok(());
    }

    let window = main_window(&app)?;
    let builder = builder_for(&app, tab_id, target, bg, proxy);
    let wv = window
        .add_child(
            builder,
            Position::Logical(LogicalPosition::new(bounds.x, bounds.y)),
            Size::Logical(LogicalSize::new(bounds.width, bounds.height)),
        )
        .map_err(|e| format!("Could not create tab webview: {}", e))?;

    if keep_others {
        let _ = wv.hide();
    } else {
        let _ = wv.show();
        let _ = wv.set_focus();
    }
    log_debug(&format!(
        "Created {}{}",
        tab_label(tab_id),
        if keep_others { " (background)" } else { "" }
    ));
    Ok(())
}

#[tauri::command]
fn hide_webviews(app: AppHandle) -> Result<(), String> {
    hide_other_webviews(&app, None);
    Ok(())
}

#[tauri::command]
fn update_web_bounds(app: AppHandle, tab_id: u32, bounds: WebBounds) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        apply_bounds(&wv, &bounds);
    }
    Ok(())
}

#[tauri::command]
fn close_webview(app: AppHandle, tab_id: u32) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        let _ = wv.close();
    }
    Ok(())
}

#[tauri::command]
fn reload_webview(app: AppHandle, tab_id: u32) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        let _ = wv.reload();
    }
    Ok(())
}

#[tauri::command]
fn back_webview(app: AppHandle, tab_id: u32) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        let _ = wv.eval("history.back()");
    }
    Ok(())
}

#[tauri::command]
fn forward_webview(app: AppHandle, tab_id: u32) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        let _ = wv.eval("history.forward()");
    }
    Ok(())
}

#[tauri::command]
fn set_web_zoom(app: AppHandle, tab_id: u32, zoom: f64) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        let _ = wv.set_zoom(zoom.clamp(0.35, 3.0));
    }
    Ok(())
}

#[tauri::command]
fn set_web_background(app: AppHandle, color: String) -> Result<(), String> {
    let c = parse_hex_color(&color);
    for wv in tab_webviews(&app) {
        let _ = wv.set_background_color(Some(c));
    }
    Ok(())
}

#[tauri::command]
fn open_web_devtools(app: AppHandle, tab_id: u32) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_label(tab_id)) {
        #[cfg(any(debug_assertions, feature = "devtools"))]
        wv.open_devtools();
        #[cfg(not(any(debug_assertions, feature = "devtools")))]
        let _ = wv;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Page reading (powers the Falses' Observe step)
// ---------------------------------------------------------------------------

const READ_PAGE_SCRIPT: &str = r#"
(function () {
    try {
        var body = document.body ? document.body.innerText : '';
        var headings = Array.prototype.slice.call(document.querySelectorAll('h1,h2,h3'))
            .slice(0, 16)
            .map(function (h) { return (h.innerText || '').trim(); })
            .filter(Boolean);
        var links = Array.prototype.slice.call(document.querySelectorAll('a[href]'))
            .slice(0, 80)
            .map(function (a) {
                return { text: (a.innerText || '').trim().slice(0, 100), href: a.href };
            })
            .filter(function (l) { return l.text && /^https?:/i.test(l.href); });
        return {
            url: location.href,
            title: document.title || '',
            text: body.replace(/\s+/g, ' ').trim().slice(0, 24000),
            headings: headings,
            links: links,
            error: null
        };
    } catch (e) {
        return {
            url: location.href,
            title: document.title || '',
            text: '',
            headings: [],
            links: [],
            error: String(e)
        };
    }
})()
"#;

#[tauri::command]
async fn read_page(app: AppHandle, tab_id: u32) -> Result<PageSnapshot, String> {
    let wv = app
        .get_webview(&tab_label(tab_id))
        .ok_or_else(|| "That tab has not been opened yet.".to_string())?;

    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let tx = std::sync::Mutex::new(Some(tx));
    wv.eval_with_callback(READ_PAGE_SCRIPT, move |raw| {
        if let Ok(mut guard) = tx.lock() {
            if let Some(sender) = guard.take() {
                let _ = sender.send(raw);
            }
        }
    })
    .map_err(|e| format!("Could not evaluate page: {}", e))?;

    let raw = tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| "Timed out reading the page.".to_string())?
        .map_err(|_| "Page read was cancelled.".to_string())?;

    serde_json::from_str::<PageSnapshot>(&raw)
        .map_err(|e| format!("Could not parse page snapshot: {} (raw: {})", e, raw))
}

/// Click an element in a tab's page. Used by the agent layer.
#[tauri::command]
fn click_page(app: AppHandle, tab_id: u32, selector: String) -> Result<(), String> {
    let wv = app
        .get_webview(&tab_label(tab_id))
        .ok_or_else(|| "That tab has not been opened yet.".to_string())?;
    let escaped = serde_json::to_string(&selector).map_err(|e| e.to_string())?;
    let script = format!(
        "(function(){{var el=document.querySelector({});if(el){{el.click();return true}}return false}})()",
        escaped
    );
    wv.eval(script).map_err(|e| e.to_string())
}

#[tauri::command]
fn eval_page(app: AppHandle, tab_id: u32, script: String) -> Result<(), String> {
    let wv = app
        .get_webview(&tab_label(tab_id))
        .ok_or_else(|| "That tab has not been opened yet.".to_string())?;
    wv.eval(script).map_err(|e| e.to_string())
}

/// Reader-mode data for the shell. Same payload as `read_page`; the UI calls
/// this name, while older agent code calls `read_page`.
#[tauri::command]
async fn get_page_content(app: AppHandle, tab_id: u32) -> Result<PageSnapshot, String> {
    read_page(app, tab_id).await
}

/// Open the user's Downloads folder in Explorer.
#[tauri::command]
fn open_downloads_folder() -> Result<(), String> {
    let dir = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|p| p.join("Downloads"))
        .unwrap_or_else(std::env::temp_dir);
    std::process::Command::new("explorer")
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Reveal a downloaded file (or folder) in Explorer.
#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Per-tab mute is tracked by the shell UI; WebView2's public Tauri wrapper
/// does not expose an audio-mute switch, so this is a successful no-op.
#[tauri::command]
fn set_tab_muted(_app: AppHandle, _tab_id: u32, _muted: bool) -> Result<(), String> {
    Ok(())
}

// ---------------------------------------------------------------------------
// State, history, bookmarks
// ---------------------------------------------------------------------------

#[tauri::command]
fn save_state(db: State<'_, Db>, key: String, value: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute(
        "INSERT INTO settings(key,value) VALUES(?1,?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_state(db: State<'_, Db>) -> Result<HashMap<String, String>, String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let mut stmt = conn
        .prepare("SELECT key,value FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut out = HashMap::new();
    for row in rows.flatten() {
        out.insert(row.0, row.1);
    }
    Ok(out)
}

#[tauri::command]
fn reset_state(db: State<'_, Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute("DELETE FROM settings", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn history_add(
    db: State<'_, Db>,
    url: String,
    title: String,
    profile_id: Option<String>,
) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(());
    }
    let host = host_of(&url);
    let title = if title.trim().is_empty() {
        host.clone()
    } else {
        title
    };
    let now = now_secs();
    let prof = profile_id.unwrap_or_else(|| "personal".to_string());
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;

    let last: Option<(i64, i64, String)> = conn
        .query_row(
            "SELECT id, visited_at, url FROM history WHERE profile_id=?1 ORDER BY id DESC LIMIT 1",
            params![prof],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some((id, at, last_url)) = last {
        if last_url == url && at >= now - 120 {
            conn.execute(
                "UPDATE history SET title=?1, visited_at=?2 WHERE id=?3",
                params![title, now, id],
            )
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    conn.execute(
        "INSERT INTO history(url,title,host,visited_at,profile_id) VALUES(?1,?2,?3,?4,?5)",
        params![url, title, host, now, prof],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT 5000)",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn history_list(
    db: State<'_, Db>,
    query: Option<String>,
    limit: Option<i64>,
    profile_id: Option<String>,
) -> Result<Vec<HistoryEntry>, String> {
    let q = query.unwrap_or_default();
    let like = format!("%{}%", q);
    let lim = limit.unwrap_or(400).clamp(1, 2000);
    let prof = profile_id.unwrap_or_default();
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id,url,title,host,visited_at FROM history
             WHERE (?1='' OR profile_id=?1) AND (?2='' OR url LIKE ?3 OR title LIKE ?3)
             ORDER BY visited_at DESC, id DESC LIMIT ?4",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![prof, q, like, lim], |r| {
            Ok(HistoryEntry {
                id: r.get(0)?,
                url: r.get(1)?,
                title: r.get(2)?,
                host: r.get(3)?,
                visited_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn history_delete(db: State<'_, Db>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute("DELETE FROM history WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn history_clear(db: State<'_, Db>, profile_id: Option<String>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    if let Some(prof) = profile_id {
        conn.execute("DELETE FROM history WHERE profile_id=?1", params![prof])
            .map_err(|e| e.to_string())?;
    } else {
        conn.execute("DELETE FROM history", [])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn bookmark_toggle(
    db: State<'_, Db>,
    url: String,
    title: String,
    profile_id: Option<String>,
) -> Result<bool, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(false);
    }
    let host = host_of(&url);
    let title = if title.trim().is_empty() {
        host.clone()
    } else {
        title
    };
    let prof = profile_id.unwrap_or_else(|| "personal".to_string());
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM bookmarks WHERE url=?1 AND profile_id=?2",
            params![url, prof],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(id) = existing {
        conn.execute("DELETE FROM bookmarks WHERE id=?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT OR REPLACE INTO bookmarks(url,title,host,added_at,profile_id) VALUES(?1,?2,?3,?4,?5)",
            params![url, title, host, now_secs(), prof],
        )
        .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
fn bookmark_is_set(db: State<'_, Db>, url: String, profile_id: Option<String>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let prof = profile_id.unwrap_or_default();
    let found: Option<i64> = conn
        .query_row(
            "SELECT id FROM bookmarks WHERE url=?1 AND (?2='' OR profile_id=?2)",
            params![url, prof],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(found.is_some())
}

#[tauri::command]
fn bookmarks_list(db: State<'_, Db>, profile_id: Option<String>) -> Result<Vec<BookmarkEntry>, String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let prof = profile_id.unwrap_or_default();
    let mut stmt = conn
        .prepare(
            "SELECT id,url,title,host,added_at FROM bookmarks
             WHERE (?1='' OR profile_id=?1)
             ORDER BY added_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![prof], |r| {
            Ok(BookmarkEntry {
                id: r.get(0)?,
                url: r.get(1)?,
                title: r.get(2)?,
                host: r.get(3)?,
                added_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn bookmark_remove(db: State<'_, Db>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute("DELETE FROM bookmarks WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Profiles & Account Management
// ---------------------------------------------------------------------------

#[tauri::command]
fn profiles_list(db: State<'_, Db>) -> Result<Vec<ProfileRecord>, String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, email, avatar, color, created_at FROM profiles ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ProfileRecord {
                id: r.get(0)?,
                name: r.get(1)?,
                email: r.get(2)?,
                avatar: r.get(3)?,
                color: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn profile_save(db: State<'_, Db>, profile: ProfileRecord) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute(
        "INSERT INTO profiles(id, name, email, avatar, color, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            email = excluded.email,
            avatar = excluded.avatar,
            color = excluded.color",
        params![
            profile.id,
            profile.name,
            profile.email,
            profile.avatar,
            profile.color,
            profile.created_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn profile_delete(db: State<'_, Db>, id: String) -> Result<(), String> {
    if id == "personal" {
        return Err("Cannot delete default personal profile".into());
    }
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute("DELETE FROM profiles WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM bookmarks WHERE profile_id=?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM history WHERE profile_id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn profile_set_active(db: State<'_, Db>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute(
        "INSERT INTO settings(key, value) VALUES('active_profile', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn profile_get_active(db: State<'_, Db>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    let active: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key='active_profile'", [], |r| r.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(active.unwrap_or_else(|| "personal".into()))
}

// ---------------------------------------------------------------------------
// Adblock commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn adblock_info(ad: State<'_, Arc<AdBlock>>) -> serde_json::Value {
    ad.info()
}

#[tauri::command]
fn adblock_set_enabled(ad: State<'_, Arc<AdBlock>>, enabled: bool) {
    ad.enabled.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
fn adblock_allow_site(
    ad: State<'_, Arc<AdBlock>>,
    db: State<'_, Db>,
    host: String,
) -> Result<(), String> {
    let host = normalize_host(&host);
    if host.is_empty() {
        return Ok(());
    }
    if let Ok(mut allow) = ad.allow.write() {
        allow.insert(host.clone());
    }
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO adblock_allow(host) VALUES(?1)",
        params![host],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn adblock_disallow_site(
    ad: State<'_, Arc<AdBlock>>,
    db: State<'_, Db>,
    host: String,
) -> Result<(), String> {
    let host = normalize_host(&host);
    if let Ok(mut allow) = ad.allow.write() {
        allow.remove(&host);
    }
    let conn = db.conn.lock().map_err(|_| "database lock".to_string())?;
    conn.execute("DELETE FROM adblock_allow WHERE host=?1", params![host])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Cookies / site data
// ---------------------------------------------------------------------------

#[tauri::command]
async fn clear_site_data(app: AppHandle) -> Result<(), String> {
    for wv in tab_webviews(&app) {
        let _ = wv.close();
    }
    tokio::time::sleep(Duration::from_millis(150)).await;
    let wv = app
        .get_webview("main")
        .ok_or_else(|| "main webview not found".to_string())?;
    wv.clear_all_browsing_data()
        .map_err(|e| format!("Could not clear site data: {}", e))
}

// ---------------------------------------------------------------------------
// Window commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn minimize_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_window("main") {
        let _ = w.minimize();
    }
    Ok(())
}

#[tauri::command]
fn toggle_maximize_window(app: AppHandle) -> Result<bool, String> {
    if let Some(w) = app.get_window("main") {
        if let Ok(is_max) = w.is_maximized() {
            if is_max {
                let _ = w.unmaximize();
                return Ok(false);
            } else {
                let _ = w.maximize();
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[tauri::command]
fn is_maximized_window(app: AppHandle) -> Result<bool, String> {
    Ok(app
        .get_window("main")
        .and_then(|w| w.is_maximized().ok())
        .unwrap_or(false))
}

#[tauri::command]
fn close_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_window("main") {
        let _ = w.close();
    }
    Ok(())
}

/// Return keyboard focus from a child page webview to the shell UI.
#[tauri::command]
fn focus_shell(app: AppHandle) -> Result<(), String> {
    if let Some(wv) = app.get_webview("main") {
        let _ = wv.set_focus();
    }
    if let Some(w) = app.get_window("main") {
        let _ = w.set_focus();
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Built-in Updater
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: String,
    pub error: Option<String>,
}

fn semver_compare(v1: &str, v2: &str) -> i32 {
    let clean = |s: &str| s.trim_start_matches('v').trim_start_matches('V').to_string();
    let parts1: Vec<u64> = clean(v1).split('.').filter_map(|p| p.parse().ok()).collect();
    let parts2: Vec<u64> = clean(v2).split('.').filter_map(|p| p.parse().ok()).collect();
    let len = std::cmp::max(parts1.len(), parts2.len());
    for i in 0..len {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);
        if p1 != p2 {
            return if p1 > p2 { 1 } else { -1 };
        }
    }
    0
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn check_browser_update(endpoint: Option<String>) -> Result<UpdateCheckResult, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let target = endpoint.unwrap_or_else(|| {
        let local_manifest = std::env::current_dir()
            .map(|p| p.join("dist").join("update.json"))
            .unwrap_or_else(|_| PathBuf::from("update.json"));
        if local_manifest.exists() {
            local_manifest.to_string_lossy().to_string()
        } else {
            "https://raw.githubusercontent.com/formerlyhonored/false/main/dist/update.json".to_string()
        }
    });

    let content = if target.starts_with("http://") || target.starts_with("https://") {
        let script = r#"$ProgressPreference = 'SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (Invoke-WebRequest -Uri $env:FALSE_UPDATE_URL -UseBasicParsing -TimeoutSec 6).Content"#;
        let out = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .env("FALSE_UPDATE_URL", &target)
            .output()
            .await
            .map_err(|e| format!("Failed to reach update server: {}", e))?;
        if !out.status.success() {
            return Ok(UpdateCheckResult {
                available: false,
                current_version: current.clone(),
                latest_version: current,
                release_notes: String::new(),
                download_url: String::new(),
                error: Some("Unable to connect to update server.".to_string()),
            });
        }
        String::from_utf8_lossy(&out.stdout).to_string()
    } else {
        std::fs::read_to_string(&target)
            .map_err(|e| format!("Cannot read update manifest: {}", e))?
    };

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid update manifest JSON: {}", e))?;

    let latest = parsed.get("version").and_then(|v| v.as_str()).unwrap_or(&current).to_string();
    let notes = parsed.get("notes").and_then(|n| n.as_str()).unwrap_or("Performance improvements and fixes.").to_string();
    let download_url = parsed.get("download_url").and_then(|u| u.as_str()).unwrap_or("").to_string();

    let is_newer = semver_compare(&latest, &current) > 0;

    Ok(UpdateCheckResult {
        available: is_newer,
        current_version: current,
        latest_version: latest,
        release_notes: notes,
        download_url,
        error: None,
    })
}

#[tauri::command]
async fn apply_browser_update(source: String) -> Result<String, String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("Cannot resolve current executable: {}", e))?;
    let updates_dir = data_dir().join("updates");
    let _ = std::fs::create_dir_all(&updates_dir);
    let staged_exe = updates_dir.join("false-ui_new.exe");

    if source.starts_with("http://") || source.starts_with("https://") {
        let script = r#"$ProgressPreference = 'SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri $env:FALSE_UPDATE_URL -OutFile $env:FALSE_UPDATE_OUT"#;
        let out = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .env("FALSE_UPDATE_URL", &source)
            .env("FALSE_UPDATE_OUT", &staged_exe)
            .output()
            .await
            .map_err(|e| format!("Update download failed: {}", e))?;
        if !out.status.success() {
            return Err("Failed to download update binary.".to_string());
        }
    } else {
        let p = PathBuf::from(&source);
        if !p.exists() {
            return Err(format!("Update file not found: {}", source));
        }
        std::fs::copy(&p, &staged_exe).map_err(|e| format!("Failed to copy update file: {}", e))?;
    }

    if !staged_exe.exists() || staged_exe.metadata().map(|m| m.len()).unwrap_or(0) < 1000 {
        return Err("Update file is incomplete or invalid.".to_string());
    }

    let batch_path = updates_dir.join("apply_update.bat");
    let batch_content = format!(
        "@echo off\r\ntimeout /t 1 /nobreak > nul\r\ntaskkill /f /im false-ui.exe > nul 2>&1\r\nmove /y \"{}\" \"{}\" > nul\r\nstart \"\" \"{}\"\r\ndel \"%~f0\" & exit\r\n",
        staged_exe.display(),
        current_exe.display(),
        current_exe.display()
    );
    std::fs::write(&batch_path, batch_content).map_err(|e| format!("Failed to write updater script: {}", e))?;

    std::process::Command::new("cmd")
        .args(["/c", "start", "", &batch_path.to_string_lossy()])
        .spawn()
        .map_err(|e| format!("Failed to launch updater: {}", e))?;

    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        std::process::exit(0);
    });

    Ok("Update downloaded! False will now restart with the new version.".to_string())
}

#[tauri::command]
async fn apply_flse_update(
    source_path: Option<String>,
    base64_content: Option<String>,
) -> Result<flse::FlseUpdateResult, String> {
    if let Some(b64) = base64_content {
        let clean = b64.trim();
        let payload = if let Some(idx) = clean.find(',') {
            &clean[idx + 1..]
        } else {
            clean
        };
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| format!("Invalid base64 payload: {}", e))?;
        return flse::apply_package_bytes(&bytes);
    }

    if let Some(path) = source_path {
        let p = PathBuf::from(path);
        if !p.exists() {
            return Err("Specified .flse package path does not exist.".to_string());
        }
        let bytes = std::fs::read(&p).map_err(|e| format!("Failed to read {}: {}", p.display(), e))?;
        return flse::apply_package_bytes(&bytes);
    }

    Err("No package data or file path provided.".to_string())
}

static PENDING_FLSE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Records a `.flse` passed on the command line. It is never applied until the
/// user confirms the verified manifest in the UI.
pub fn set_pending_package(path: PathBuf) {
    if let Ok(mut slot) = PENDING_FLSE.lock() {
        *slot = Some(path);
    }
}

#[derive(Serialize)]
pub struct PendingFlse {
    pub path: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub has_binary: bool,
}

/// Returns a verified preview of a command-line package without applying it.
#[tauri::command]
fn get_pending_flse() -> Option<PendingFlse> {
    let path = PENDING_FLSE.lock().ok()?.clone()?;
    match flse::preview_package_file(&path) {
        Ok(manifest) => Some(PendingFlse {
            path: path.to_string_lossy().to_string(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author,
            description: manifest.description,
            has_binary: manifest.has_binary,
        }),
        Err(e) => {
            log_debug(&format!(
                "Ignoring pending package {}: {}",
                path.display(),
                e
            ));
            if let Ok(mut slot) = PENDING_FLSE.lock() {
                *slot = None;
            }
            None
        }
    }
}

#[tauri::command]
fn clear_pending_flse() {
    if let Ok(mut slot) = PENDING_FLSE.lock() {
        *slot = None;
    }
}

#[tauri::command]
async fn export_flse_package(
    version: String,
    description: String,
    out_path: Option<String>,
) -> Result<String, String> {
    let target_path = if let Some(p) = out_path {
        PathBuf::from(p)
    } else {
        let downloads = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|p| p.join("Downloads"))
            .unwrap_or_else(std::env::temp_dir);
        downloads.join(format!("false-v{}.flse", version.trim()))
    };

    flse::create_package(&version, &description, false, &target_path)?;
    Ok(format!("Package exported successfully to: {}", target_path.display()))
}

#[tauri::command]
fn get_package_info() -> serde_json::Value {
    let version_file = data_dir().join("version.json");
    if let Ok(text) = std::fs::read_to_string(&version_file) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
            return val;
        }
    }
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "False Browser",
        "developer": "zaidi",
        "owner": "zaidi",
        "description": "Fast, minimal web navigator"
    })
}

// ---------------------------------------------------------------------------
// Native False Browser Engine IPC

#[tauri::command]
fn detect_installed_browsers() -> Vec<browser_import::InstalledBrowser> {
    browser_import::detect_installed()
}

#[tauri::command]
fn import_browser_data(browser_ids: Vec<String>, db: State<Db>) -> Result<browser_import::ImportResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(browser_import::import_from_browsers(&browser_ids, &conn))
}

#[tauri::command]
fn get_discord_rpc_enabled() -> bool {
    discord::is_enabled()
}

#[tauri::command]
fn set_discord_rpc_enabled(enabled: bool) -> Result<bool, String> {
    discord::set_enabled(enabled);
    Ok(enabled)
}
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngineRenderResponse {
    pub data_url: String,
    pub dom_tree: String,
    pub command_count: usize,
    pub render_time_ms: f64,
}

#[tauri::command]
fn false_engine_render(html: String, width: u32, height: u32) -> Result<EngineRenderResponse, String> {
    let start = std::time::Instant::now();
    let output = false_engine::Engine::render(&html, width.max(100), height.max(100));
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok(EngineRenderResponse {
        data_url: output.to_data_url(),
        dom_tree: output.dom_tree,
        command_count: output.display_list.len(),
        render_time_ms: (elapsed * 100.0).round() / 100.0,
    })
}

// ---------------------------------------------------------------------------
// App setup
// ---------------------------------------------------------------------------

/// The proxy URL is resolved at runtime (the OS picks a free loopback port),
/// then handed to every webview so they share one WebView2 environment.
struct ProxyInfo {
    url: String,
}

/// Where the swappable shell UI lives.
///
/// Precedence:
///   1. `FALSE_UI_DIR` environment variable
///   2. `<exe folder>/ui` (portable / packaged app)
///   3. the source `ui/` directory (development)
///
/// The HTML is served from disk through the `false://` protocol, so replacing
/// `index.html` and restarting is enough to install a brand-new UI. If no
/// folder is found, Tauri's embedded assets are used as a fallback.
pub(crate) fn ui_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("FALSE_UI_DIR") {
        let p = PathBuf::from(dir);
        if p.join("index.html").exists() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("ui");
            if p.join("index.html").exists() {
                return p;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn mime_for(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let exe_stem = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let is_uninstaller = args.iter().any(|a| a == "--uninstaller") || exe_stem.contains("uninstall");
    let is_installer = !is_uninstaller && (args.iter().any(|a| a == "--installer" || a == "--setup") || exe_stem.contains("setup") || exe_stem.contains("installer"));

    let (adblock, proxy_url) = if is_installer || is_uninstaller {
        (Arc::new(AdBlock::empty()), String::new())
    } else {
        (Arc::new(AdBlock::new()), String::new())
    };

    let page = if is_uninstaller {
        "uninstaller.html"
    } else if is_installer {
        "installer.html"
    } else {
        "index.html"
    };

    let title = if is_uninstaller {
        "Uninstall False"
    } else if is_installer {
        "Install False"
    } else {
        "False"
    };

    let (win_width, win_height) = if is_installer || is_uninstaller {
        (1080.0, 640.0)
    } else {
        (1320.0, 860.0)
    };

    let ui_dir = ui_root();
    let has_disk_ui = ui_dir.join(page).exists();
    let start_url = if has_disk_ui {
        WebviewUrl::CustomProtocol(
            format!("false://localhost/{}", page)
                .parse()
                .expect("valid false:// UI url"),
        )
    } else {
        WebviewUrl::App(page.into())
    };
    let ui_dir_for_protocol = ui_dir.clone();

    tauri::Builder::default()
        .register_uri_scheme_protocol("false", move |_ctx, request| {
            let raw_path = request.uri().path().trim_start_matches('/');
            let raw_path = if raw_path.is_empty() {
                "index.html"
            } else {
                raw_path
            };

            // Resolve safely inside the UI root; no path traversal.
            let mut file = ui_dir_for_protocol.clone();
            for part in raw_path.split('/') {
                if part.is_empty() || part == "." {
                    continue;
                }
                if part == ".." {
                    break;
                }
                file.push(part);
            }

            match std::fs::read(&file) {
                Ok(data) => tauri::http::Response::builder()
                    .header(tauri::http::header::CONTENT_TYPE, mime_for(&file))
                    .header(tauri::http::header::CACHE_CONTROL, "no-cache")
                    .body(data)
                    .unwrap(),
                Err(_) => tauri::http::Response::builder()
                    .status(tauri::http::StatusCode::NOT_FOUND)
                    .header(tauri::http::header::CONTENT_TYPE, "text/plain")
                    .body(b"Not found".to_vec())
                    .unwrap(),
            }
        })
        .plugin(tauri_plugin_opener::init())
        .manage(adblock)
        .manage(ProxyInfo {
            url: proxy_url.clone(),
        })
        .setup(move |app| {
            let db = if is_installer || is_uninstaller {
                let conn = Connection::open_in_memory().expect("in-memory db");
                Db {
                    conn: Mutex::new(conn),
                }
            } else {
                let db = open_db().unwrap_or_else(|e| {
                    log_debug(&e);
                    let conn = Connection::open_in_memory().expect("in-memory db");
                    let _ = apply_schema(&conn);
                    Db {
                        conn: Mutex::new(conn),
                    }
                });

                // Restore per-site adblock exceptions before the first page loads.
                {
                    let allow_hosts = load_allow_hosts(&db);
                    let ad = app.state::<Arc<AdBlock>>().inner().clone();
                    if let Ok(mut allow) = ad.allow.write() {
                        allow.extend(allow_hosts);
                    };
                }
                db
            };
            app.manage(db);

            let mut builder = tauri::WebviewWindowBuilder::new(app, "main", start_url.clone())
                .title(title)
                .inner_size(win_width, win_height)
                .min_inner_size(if is_installer || is_uninstaller { 800.0 } else { 720.0 }, if is_installer || is_uninstaller { 500.0 } else { 480.0 })
                .resizable(true)
                .decorations(false)
                .shadow(true)
                .background_color(tauri::webview::Color(5, 5, 6, 255))
                .center();
            if !proxy_url.is_empty() {
                if let Ok(url) = proxy_url.parse::<tauri::Url>() {
                    builder = builder.proxy_url(url);
                }
            }

            let window = builder.build()?;
            if let Some(icon) = app.default_window_icon() {
                let _ = window.set_icon(icon.clone());
            }
            let _ = window.set_focus();
            if !is_installer && !is_uninstaller {
                discord::start();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_webview,
            hide_webviews,
            capture_tab,
            update_web_bounds,
            close_webview,
            reload_webview,
            back_webview,
            forward_webview,
            set_web_zoom,
            set_web_background,
            open_web_devtools,
            read_page,
            get_page_content,
            open_downloads_folder,
            open_path,
            set_tab_muted,
            click_page,
            eval_page,
            save_state,
            load_state,
            reset_state,
            history_add,
            history_list,
            history_delete,
            history_clear,
            bookmark_toggle,
            bookmark_is_set,
            bookmarks_list,
            bookmark_remove,
            profiles_list,
            profile_save,
            profile_delete,
            profile_set_active,
            profile_get_active,
            adblock_info,
            adblock_set_enabled,
            adblock_allow_site,
            adblock_disallow_site,
            clear_site_data,
            minimize_window,
            toggle_maximize_window,
            is_maximized_window,
            close_window,
            focus_shell,
            get_app_version,
            check_browser_update,
            apply_browser_update,
            apply_flse_update,
            export_flse_package,
            get_pending_flse,
            clear_pending_flse,
            get_package_info,
            detect_installed_browsers,
            import_browser_data,
            get_discord_rpc_enabled,
            set_discord_rpc_enabled,
            false_engine_render,
            installer_ui::installer_get_defaults,
            installer_ui::installer_pick_folder,
            installer_ui::installer_run,
            installer_ui::installer_launch_app,
            installer_ui::installer_close,
            installer_ui::uninstaller_run,
            installer_ui::uninstaller_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running False browser");
}
