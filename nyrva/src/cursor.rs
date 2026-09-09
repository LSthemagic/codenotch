//! Cursor usage adapter, implemented from the upstream Codenotch's documented behaviour.
//!
//! Data path (same trade-off as upstream: borrow the editor's own session):
//!   1. Credential: the editor keeps its sign-in in the global state database it inherited from
//!      VS Code: `%APPDATA%\Cursor\User\globalStorage\state.vscdb` on Windows and
//!      `${XDG_CONFIG_HOME:-~/.config}/Cursor/User/globalStorage/state.vscdb` on Linux (SQLite,
//!      table ItemTable(key,value)). Nyrva reads `cursorAuth/accessToken` +
//!      `cursorAuth/stripeMembershipAuthId`, joined into the cookie
//!      `WorkosCursorSessionToken=<authId>::<token>`. Non-secret identity cache:
//!      `cursorAuth/cachedEmail`, `cursorAuth/stripeMembershipType` (only the plan is shown).
//!   2. Endpoint: `GET https://cursor.com/api/usage-summary` (Cookie + Accept: application/json, 15 s).
//!      Reply: { billingCycleEnd, membershipType, isUnlimited,
//!              individualUsage: { plan: { totalPercentUsed, apiPercentUsed, used, limit, breakdown },
//!                                 onDemand: { enabled, used, limit } } }
//!      Cursor meters a percentage of the allowance, not requests: the dashboard's
//!      "Included usage · N% used" is totalPercentUsed. On the free plan used/limit are always 0
//!      (the allowance arrives as breakdown.bonus), so reading used/limit would report 10 % as 0 %.
//!      0 is a reading, not a gap (upstream's lesson). "API usage" is listed separately when
//!      apiPercentUsed > 0; "On demand" when onDemand has a real limit.
//!
//! SQLite opening rule: plain read-only first (it sees the token the editor just rotated into the
//! WAL), then `immutable=1` (once the editor has exited and the -shm is gone, plain read-only may
//! fail to read; by then the WAL has been checkpointed, so ignoring it costs nothing).
//! Read only, never written; token values never reach logs, events or the UI.

use crate::usage::{LimitWindow, UsageSnapshot};
use crate::AppState;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const ENDPOINT: &str = "https://cursor.com/api/usage-summary";
const POLL_SECS: u64 = 300;

static REFRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn request_refresh() {
    REFRESH.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn cursor_state_path(config_dir: &Path) -> PathBuf {
    config_dir
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")
}

fn linux_config_dir_from(xdg: Option<&OsStr>, home: Option<&Path>) -> Option<PathBuf> {
    xdg.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".config")))
}

#[cfg(windows)]
fn cursor_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

#[cfg(target_os = "linux")]
fn cursor_config_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME");
    let home = dirs::home_dir();
    linux_config_dir_from(xdg.as_deref(), home.as_deref())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn cursor_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

/// Windows: `%APPDATA%/Cursor/...`; Linux: `${XDG_CONFIG_HOME:-~/.config}/Cursor/...`.
pub fn store_url() -> Option<PathBuf> {
    cursor_config_dir().map(|config| cursor_state_path(&config))
}

fn store_path() -> PathBuf {
    crate::config::config_path().with_file_name("cursor.json")
}

pub fn load_persisted() -> UsageSnapshot {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<UsageSnapshot>(&t).ok())
        .map(|mut s| {
            if !s.windows.is_empty() {
                s.status = "stale".into();
            }
            s
        })
        .unwrap_or_default()
}

fn persist(s: &UsageSnapshot) {
    if let Ok(t) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(store_path(), t);
    }
}

pub fn present() -> bool {
    store_url().map(|p| p.is_file()).unwrap_or(false)
}

// ---------------- SQLite, read only ----------------

fn encode_uri_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'/'
            | b':'
            | b'-'
            | b'_'
            | b'.'
            | b'~' => out.push(*byte as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn immutable_uri(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let body = normalized.trim_start_matches('/');
    format!("file:///{}?immutable=1", encode_uri_path(body))
}

/// Plain read-only first so a live WAL is visible; immutable URI is only a closed/checkpointed fallback.
fn open_ro(path: &Path) -> Option<rusqlite::Connection> {
    use rusqlite::OpenFlags;
    if !path.is_file() {
        return None;
    }
    if let Ok(c) = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        // Actually verify that reads work (with the -shm missing, open can succeed and the first query fail).
        if c.prepare("SELECT 1 FROM ItemTable LIMIT 1")
            .and_then(|mut s| s.query([]).map(|_| ()))
            .is_ok()
        {
            return Some(c);
        }
    }

    let uri = immutable_uri(path);
    rusqlite::Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()
}

fn item(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |r| r.get::<_, String>(0))
        .ok()
        .filter(|s| !s.is_empty())
}

struct Creds {
    cookie: String,
    plan: Option<String>,
}

/// Re-read every time: the editor rotates the token, and holding on to an old value signs us out.
fn read_credentials() -> Option<Creds> {
    let path = store_url()?;
    let conn = open_ro(&path)?;
    let token = item(&conn, "cursorAuth/accessToken")?;
    let auth_id = item(&conn, "cursorAuth/stripeMembershipAuthId")?;
    let plan = item(&conn, "cursorAuth/stripeMembershipType");
    Some(Creds { cookie: format!("WorkosCursorSessionToken={auth_id}::{token}"), plan })
}

fn credential_probe(cookie_len: usize, plan: Option<&str>) -> String {
    format!(
        "Cursor: session borrowed (cookie {cookie_len} chars, plan={})",
        plan.unwrap_or("?")
    )
}

/// For doctor: contains no secret values.
pub fn probe() -> String {
    let Some(p) = store_url() else {
        return "Cursor: cannot locate the configuration directory".into();
    };
    if !p.is_file() {
        return format!("Cursor: {} not found (not installed, or not signed in)", p.display());
    }
    match read_credentials() {
        Some(c) => credential_probe(c.cookie.len(), c.plan.as_deref()),
        None => format!(
            "Cursor: {} exists but cursorAuth/* could not be read (editor not signed in, or SQLite failed to open)",
            p.display()
        ),
    }
}

// ---------------- Parsing ----------------

fn pct(v: Option<&serde_json::Value>) -> Option<f64> {
    v.and_then(|x| x.as_f64()).map(|p| (p / 100.0).clamp(0.0, 1.0))
}

fn parse_iso(v: Option<&serde_json::Value>) -> Option<u64> {
    v.and_then(|x| x.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis().max(0) as u64)
}

/// usage-summary → (windows, note). When there are no windows the note says why (Unlimited / free plan without an allowance).
pub fn parse_summary(v: &serde_json::Value) -> (Vec<LimitWindow>, String) {
    let resets_at = parse_iso(v.get("billingCycleEnd"));
    let usage = v.get("individualUsage").cloned().unwrap_or(serde_json::Value::Null);
    let plan = usage.get("plan").cloned().unwrap_or(serde_json::Value::Null);
    let mut out = Vec::new();
    if let Some(total) = pct(plan.get("totalPercentUsed")) {
        out.push(LimitWindow { id: "included".into(), label: "Included usage".into(), used: total, resets_at, ..Default::default() });
    }
    if let Some(api) = pct(plan.get("apiPercentUsed")) {
        if api > 0.0 {
            out.push(LimitWindow { id: "api".into(), label: "API usage".into(), used: api, resets_at, ..Default::default() });
        }
    }
    if let Some(od) = usage.get("onDemand") {
        let enabled = od.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false);
        let limit = od.get("limit").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let used = od.get("used").and_then(|x| x.as_f64());
        if enabled && limit > 0.0 {
            if let Some(u) = used {
                out.push(LimitWindow {
                    id: "on_demand".into(),
                    label: "On demand".into(),
                    used: (u / limit).clamp(0.0, 1.0),
                    resets_at,
                    ..Default::default()
                });
            }
        }
    }
    if !out.is_empty() {
        return (out, String::new());
    }
    let membership = v.get("membershipType").and_then(|x| x.as_str()).unwrap_or("this");
    let note = if v.get("isUnlimited").and_then(|x| x.as_bool()) == Some(true) {
        format!("Unlimited on the {membership} plan — nothing to meter")
    } else {
        format!("The {membership} plan has nothing for Cursor to meter yet")
    };
    (out, note)
}

enum FetchErr {
    NeedsAuth,
    Other(String),
}

fn fetch_once(cookie: &str) -> Result<serde_json::Value, FetchErr> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build();
    match agent.get(ENDPOINT).set("Cookie", cookie).set("Accept", "application/json").call() {
        Ok(r) => r.into_json::<serde_json::Value>().map_err(|e| FetchErr::Other(format!("parse: {e}"))),
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => Err(FetchErr::NeedsAuth),
        Err(ureq::Error::Status(code, _)) => Err(FetchErr::Other(format!("HTTP {code}"))),
        Err(e) => Err(FetchErr::Other(format!("{e}"))),
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn read_once(prev: &UsageSnapshot) -> UsageSnapshot {
    let mut snap = prev.clone();
    let Some(creds) = read_credentials() else {
        snap.status = "needsAuth".into();
        snap.note = "Sign in to Cursor (the editor) to see usage.".into();
        return snap;
    };
    match fetch_once(&creds.cookie) {
        Ok(v) => {
            let (windows, note) = parse_summary(&v);
            snap.fetched_at = now_ms();
            if windows.is_empty() {
                snap.status = "none".into();
                snap.windows.clear();
                snap.note = note;
            } else {
                snap.status = "ok".into();
                snap.windows = windows;
                snap.note = match (&creds.plan, v.get("membershipType").and_then(|x| x.as_str())) {
                    (_, Some(m)) => format!("{} · via Cursor", cap(m)),
                    (Some(p), None) => format!("{} · via Cursor", cap(p)),
                    _ => String::new(),
                };
            }
        }
        Err(FetchErr::NeedsAuth) => {
            snap.status = "needsAuth".into();
            snap.note = "Cursor session was rejected — sign in again in the editor".into();
        }
        Err(FetchErr::Other(msg)) => {
            snap.status = if snap.windows.is_empty() { "error" } else { "stale" }.into();
            snap.note = msg;
        }
    }
    snap
}

fn broadcast(app: &AppHandle, snap: UsageSnapshot) {
    let st = app.state::<AppState>();
    *st.cursor.lock().unwrap() = snap.clone();
    persist(&snap);
    let _ = app.emit("cursor", &snap);
}

fn sleep_interruptible(secs: u64) {
    for _ in 0..secs {
        if REFRESH.swap(false, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        {
            let st = app.state::<AppState>();
            let snap = st.cursor.lock().unwrap().clone();
            let _ = app.emit("cursor", &snap);
        }
        if !present() {
            broadcast(&app, UsageSnapshot { status: "absent".into(), ..Default::default() });
            loop {
                sleep_interruptible(600);
                if present() {
                    break;
                }
            }
        }
        loop {
            let prev = {
                let st = app.state::<AppState>();
                let s = st.cursor.lock().unwrap().clone();
                s
            };
            let snap = read_once(&prev);
            if snap.status == "error" || snap.status == "stale" {
                crate::applog(&format!("cursor: {}", snap.note));
            }
            broadcast(&app, snap);
            sleep_interruptible(POLL_SECS);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn linux_config_prefers_xdg_config_home() {
        assert_eq!(
            linux_config_dir_from(Some(OsStr::new("/tmp/xdg")), Some(Path::new("/home/me"))),
            Some(PathBuf::from("/tmp/xdg"))
        );
    }

    #[test]
    fn linux_config_falls_back_to_dot_config() {
        assert_eq!(
            linux_config_dir_from(Some(OsStr::new("")), Some(Path::new("/home/me"))),
            Some(PathBuf::from("/home/me/.config"))
        );
    }

    #[test]
    fn cursor_state_path_is_shared_shape() {
        assert_eq!(
            cursor_state_path(Path::new("/tmp/config")),
            PathBuf::from("/tmp/config/Cursor/User/globalStorage/state.vscdb")
        );
    }

    #[test]
    fn immutable_uri_handles_unix_path() {
        assert_eq!(
            immutable_uri(Path::new("/home/me/Cursor Data/state#?.vscdb")),
            "file:///home/me/Cursor%20Data/state%23%3F.vscdb?immutable=1"
        );
    }

    #[test]
    fn immutable_uri_handles_windows_shape() {
        assert_eq!(
            immutable_uri(Path::new(r"C:\Users\Me\Cursor Data\state.vscdb")),
            "file:///C:/Users/Me/Cursor%20Data/state.vscdb?immutable=1"
        );
    }

    #[test]
    fn diagnostic_does_not_render_cookie_secret() {
        let secret = "top-secret-cookie";
        let rendered = credential_probe(secret.len(), Some("pro"));
        assert_eq!(rendered, "Cursor: session borrowed (cookie 17 chars, plan=pro)");
        assert!(!rendered.contains(secret));
    }

    #[test]
    fn readonly_open_sees_live_wal() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nyrva-cursor-wal-{}-{stamp}.sqlite", std::process::id()));

        let writer = rusqlite::Connection::open(&path).unwrap();
        writer
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA wal_autocheckpoint=0;
                 CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);
                 PRAGMA wal_checkpoint(TRUNCATE);
                 INSERT INTO ItemTable(key,value) VALUES ('cursorAuth/accessToken','fresh-token');",
            )
            .unwrap();

        let reader = open_ro(&path).expect("read-only connection should see the live WAL");
        assert_eq!(item(&reader, "cursorAuth/accessToken").as_deref(), Some("fresh-token"));

        drop(reader);
        drop(writer);
        let _ = std::fs::remove_file(&path);
        let mut wal = path.as_os_str().to_owned();
        wal.push("-wal");
        let _ = std::fs::remove_file(PathBuf::from(wal));
        let mut shm = path.as_os_str().to_owned();
        shm.push("-shm");
        let _ = std::fs::remove_file(PathBuf::from(shm));
    }
}
