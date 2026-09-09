# M6 Cursor Linux Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Cursor usage and activity discovery Linux-native while preserving the existing read-only SQLite/session borrowing model.

**Architecture:** Put platform/path differences behind pure helpers in `cursor.rs`; both usage and activity consume the same state database resolver. Keep live read-only SQLite first so WAL changes are visible, then immutable URI fallback for closed/checkpointed databases.

**Tech Stack:** Rust 2021, Tauri 2, rusqlite 0.32, ureq 2, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md`

## Global Constraints

- Windows + Linux X11 only.
- Linux Cursor DB: `${XDG_CONFIG_HOME:-~/.config}/Cursor/User/globalStorage/state.vscdb`.
- Windows Cursor DB remains `%APPDATA%/Cursor/User/globalStorage/state.vscdb`.
- Cursor database/auth is read-only; secrets never enter logs/UI/Nyrva snapshots.
- No Wayland or UI redesign.
- New path/URI behavior uses TDD.

---

### Task 1: Add explicit cross-platform state database resolution

**Files:** `nyrva/src/cursor.rs`, `nyrva/src/activity.rs`

**Interfaces:**
- `fn cursor_state_path_from(xdg: Option<&OsStr>, home: Option<&Path>, windows_config: Option<&Path>, os: &str) -> Option<PathBuf>`
- `pub fn store_url() -> Option<PathBuf>`

- [ ] **Step 1: Write failing path tests**

```rust
#[test]
fn linux_cursor_path_prefers_xdg() {
    let got = cursor_state_path_from(
        Some(OsStr::new("/tmp/xdg")),
        Some(Path::new("/home/me")),
        None,
        "linux",
    );
    assert_eq!(got, Some(PathBuf::from("/tmp/xdg/Cursor/User/globalStorage/state.vscdb")));
}

#[test]
fn linux_cursor_path_falls_back_to_dot_config() {
    let got = cursor_state_path_from(None, Some(Path::new("/home/me")), None, "linux");
    assert_eq!(got, Some(PathBuf::from("/home/me/.config/Cursor/User/globalStorage/state.vscdb")));
}

#[test]
fn windows_cursor_path_uses_config_dir() {
    let got = cursor_state_path_from(
        None,
        None,
        Some(Path::new("C:/Users/me/AppData/Roaming")),
        "windows",
    );
    assert_eq!(got, Some(PathBuf::from("C:/Users/me/AppData/Roaming/Cursor/User/globalStorage/state.vscdb")));
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva cursor::tests::linux_cursor_path -- --nocapture`

Expected: compile failure because `cursor_state_path_from` does not exist.

- [ ] **Step 3: Implement resolver and runtime wrapper**

```rust
fn cursor_state_path_from(
    xdg: Option<&OsStr>,
    home: Option<&Path>,
    windows_config: Option<&Path>,
    os: &str,
) -> Option<PathBuf> {
    let root = if os == "windows" {
        windows_config?.to_path_buf()
    } else {
        match xdg.filter(|v| !v.is_empty()) {
            Some(v) => PathBuf::from(v),
            None => home?.join(".config"),
        }
    };
    Some(root.join("Cursor").join("User").join("globalStorage").join("state.vscdb"))
}

pub fn store_url() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME");
    let home = dirs::home_dir();
    let config = dirs::config_dir();
    cursor_state_path_from(xdg.as_deref(), home.as_deref(), config.as_deref(), std::env::consts::OS)
}
```

- [ ] **Step 4: Prove activity uses the same resolver**

Extract in `activity.rs`:

```rust
fn cursor_activity_path() -> PathBuf {
    crate::cursor::store_url().unwrap_or_default()
}
```

Use it in `Ctx::new()` and add:

```rust
#[test]
fn cursor_activity_path_is_provider_store_path() {
    assert_eq!(cursor_activity_path(), crate::cursor::store_url().unwrap_or_default());
}
```

- [ ] **Step 5: Verify GREEN and commit**

Run: `cargo test -p nyrva cursor::tests activity::tests -- --nocapture`

Commit: `feat(cursor): add explicit Linux state path resolver`

---

### Task 2: Make immutable SQLite URI generation cross-platform

**Files:** `nyrva/src/cursor.rs`

**Interfaces:**
- `fn immutable_uri(path: &Path) -> String`
- existing `open_ro(path)` keeps normal read-only first, then immutable URI fallback.

- [ ] **Step 1: Write failing URI tests**

```rust
#[test]
fn immutable_uri_handles_unix_path() {
    assert_eq!(
        immutable_uri(Path::new("/home/me/Cursor data/state.vscdb")),
        "file:/home/me/Cursor%20data/state.vscdb?immutable=1"
    );
}

#[test]
fn immutable_uri_handles_windows_path_shape() {
    assert_eq!(
        immutable_uri(Path::new("C:\\Users\\Me\\Cursor Data\\state.vscdb")),
        "file:///C:/Users/Me/Cursor%20Data/state.vscdb?immutable=1"
    );
}

#[test]
fn immutable_uri_escapes_reserved_chars() {
    assert_eq!(
        immutable_uri(Path::new("/tmp/a#b?c%/state.vscdb")),
        "file:/tmp/a%23b%3Fc%25/state.vscdb?immutable=1"
    );
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva cursor::tests::immutable_uri_ -- --nocapture`

- [ ] **Step 3: Implement URI helper**

Normalize `\\` to `/`; encode `%` → `%25`, space → `%20`, `#` → `%23`, `?` → `%3F`; use `file:///` for `X:/...` drive shapes and `file:` for absolute Unix paths; append `?immutable=1`.

- [ ] **Step 4: Keep open order exactly WAL-first**

```rust
fn open_ro(path: &Path) -> Option<rusqlite::Connection> {
    use rusqlite::OpenFlags;
    let ro = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    if let Ok(c) = rusqlite::Connection::open_with_flags(path, ro) {
        if c.prepare("SELECT 1 FROM ItemTable LIMIT 1")
            .and_then(|mut s| s.query([]).map(|_| ()))
            .is_ok()
        {
            return Some(c);
        }
    }
    let uri = immutable_uri(path);
    rusqlite::Connection::open_with_flags(
        uri,
        ro | OpenFlags::SQLITE_OPEN_URI,
    ).ok()
}
```

- [ ] **Step 5: Add executable read-only regression**

```rust
#[test]
fn open_ro_reads_closed_cursor_db_without_writing() {
    let dir = std::env::temp_dir().join(format!("nyrva-cursor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("state.vscdb");
    {
        let c = rusqlite::Connection::open(&db).unwrap();
        c.execute("CREATE TABLE ItemTable(key TEXT PRIMARY KEY, value TEXT)", []).unwrap();
        c.execute("INSERT INTO ItemTable(key,value) VALUES('cursorAuth/cachedEmail','safe@example.test')", []).unwrap();
    }
    let c = open_ro(&db).expect("read-only open");
    let value: String = c.query_row(
        "SELECT value FROM ItemTable WHERE key='cursorAuth/cachedEmail'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(value, "safe@example.test");
    assert!(c.execute("INSERT INTO ItemTable(key,value) VALUES('x','y')", []).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
```

- [ ] **Step 6: Verify GREEN and commit**

Run: `cargo test -p nyrva cursor::tests -- --nocapture`

Commit: `fix(cursor): make immutable SQLite URI cross-platform`

---

### Task 3: Make diagnostics Linux-aware and secret-safe

**Files:** `nyrva/src/cursor.rs`

**Interfaces:** `fn probe_message(path: Option<&Path>, cookie_len: Option<usize>, plan: Option<&str>, os: &str) -> String`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn linux_probe_does_not_say_appdata() {
    let msg = probe_message(
        Some(Path::new("/home/me/.config/Cursor/User/globalStorage/state.vscdb")),
        None,
        None,
        "linux",
    );
    assert!(!msg.contains("%APPDATA%"));
    assert!(msg.contains("state.vscdb"));
}

#[test]
fn probe_accepts_only_secret_length() {
    let msg = probe_message(Some(Path::new("state.vscdb")), Some(42), Some("pro"), "linux");
    assert!(msg.contains("42 chars"));
    assert!(msg.contains("plan=pro"));
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva cursor::tests::probe_ -- --nocapture`

- [ ] **Step 3: Implement formatter and wire `probe()`**

`probe()` may compute `c.cookie.len()` but must never pass the cookie string into the formatter. Missing Linux path says `cannot locate Cursor config directory`; `%APPDATA%` may appear only in Windows-specific output.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test -p nyrva cursor::tests -- --nocapture`

Commit: `fix(cursor): make diagnostics Linux-aware and secret-safe`

---

### Task 4: Milestone verification and PR gate

- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `git diff --check`
- [ ] `git grep -n '%APPDATA%\|file:///' -- nyrva/src/cursor.rs nyrva/src/activity.rs` and verify Windows-only wording is scoped to Windows/tests.
- [ ] Require Windows + Linux GitHub Actions green, including existing Linux Tauri bundle/sidecar gate.
- [ ] Open PR titled `M6: add Cursor Linux parity` and stop before merge.
