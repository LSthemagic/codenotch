# M7 Antigravity Linux Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Antigravity quota discovery on Linux through the local language-server bridge, Linux Secret Service, and existing transcript fallback without exposing provider credentials.

**Architecture:** Keep provider behavior in `antigravity.rs`, but isolate Linux system discovery behind pure parsers. Prefer `/proc` for listener discovery, retain `lsof` only as fallback, and use keyring 3.x synchronous Secret Service behind Linux `cfg`.

**Tech Stack:** Rust 2021, Tauri 2, ureq 2, native-tls, keyring 3.x `sync-secret-service`, Linux `/proc`, GitHub Actions Ubuntu 22.04.

**Spec:** `docs/superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md`

## Global Constraints

- Windows + Linux X11 only.
- Source priority: bridge → stale bridge → credential path → transcript fallback.
- Secret Service and provider state are read-only.
- Tokens and CSRF values never enter logs/UI/Nyrva snapshots.
- Relaxed TLS applies only to local loopback bridge calls.
- No provider impersonation, Wayland, UI redesign, or managed login flow.
- New Linux behavior uses TDD.

---

### Task 1: Discover language-server listening ports from `/proc`

**Files:** `nyrva/src/antigravity.rs`

**Interfaces:**
- `fn socket_inode(target: &str) -> Option<u64>`
- `fn listening_ports_from_proc(text: &str, inodes: &HashSet<u64>) -> Vec<u16>`
- `#[cfg(target_os="linux")] fn proc_listening_ports(pid: u32) -> Vec<u16>`
- `#[cfg(target_os="linux")] fn lsof_listening_ports(pid: u32) -> Vec<u16>`

- [ ] **Step 1: Write failing parser tests**

```rust
#[test]
fn parses_socket_inode_target() {
    assert_eq!(socket_inode("socket:[12345]"), Some(12345));
    assert_eq!(socket_inode("anon_inode:[eventpoll]"), None);
}

#[test]
fn maps_listening_socket_inode_to_port_and_dedupes() {
    let text = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt uid timeout inode\n\
   0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000 1000 0 12345\n\
   1: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000 1000 0 12345\n\
   2: 0100007F:2382 00000000:0000 01 00000000:00000000 00:00000000 00000000 1000 0 67890\n";
    let got = listening_ports_from_proc(text, &HashSet::from([12345, 67890]));
    assert_eq!(got, vec![8080]);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva antigravity::tests::parses_socket_inode_target -- --nocapture`

Expected: compile failure because helpers do not exist.

- [ ] **Step 3: Implement pure parsing**

```rust
fn socket_inode(target: &str) -> Option<u64> {
    target.strip_prefix("socket:[")?.strip_suffix(']')?.parse().ok()
}
```

For each `/proc/net/tcp*` data row: split whitespace; require at least 10 columns; `cols[3] == "0A"`; parse local port from `cols[1].rsplit_once(':').1` as base-16; parse inode from `cols[9]`; keep only inodes owned by the target process; sort and dedupe.

- [ ] **Step 4: Implement runtime `/proc` resolver**

Read symlinks under `/proc/<pid>/fd`, collect socket inodes, then parse `/proc/net/tcp` and `/proc/net/tcp6`. Merge, sort, dedupe.

- [ ] **Step 5: Retain `lsof` only as fallback**

```rust
let mut ports = proc_listening_ports(pid);
if ports.is_empty() {
    ports = lsof_listening_ports(pid);
}
```

No package requirement on `lsof` is added.

- [ ] **Step 6: Verify GREEN and commit**

Run: `cargo test -p nyrva antigravity::tests -- --nocapture`

Commit: `feat(antigravity): discover Linux bridge ports via proc`

---

### Task 2: Add read-only Linux Secret Service credential lookup

**Files:** `nyrva/Cargo.toml`, `Cargo.lock`, `nyrva/src/antigravity.rs`, `.github/workflows/ci.yml`

**Interfaces:**
- existing `fn decode_credential(raw: &[u8]) -> Option<Creds>` remains pure
- `#[cfg(target_os="linux")] fn read_credential_raw() -> Option<Vec<u8>>`
- keyring identity: service `gemini`, user `antigravity`

- [ ] **Step 1: Add Linux-only keyring dependency**

```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.13"
keyring = { version = "3", default-features = false, features = ["sync-secret-service"] }
```

Add `libdbus-1-dev` and `pkg-config` to Ubuntu CI packages.

- [ ] **Step 2: Write a pure failure-seam test**

```rust
fn usable_secret(result: Result<Vec<u8>, String>) -> Option<Vec<u8>> {
    result.ok().filter(|v| !v.is_empty())
}

#[test]
fn locked_or_missing_secret_service_is_nonfatal() {
    assert_eq!(usable_secret(Err("locked".into())), None);
    assert_eq!(usable_secret(Ok(Vec::new())), None);
}
```

- [ ] **Step 3: Verify RED**

Run: `cargo test -p nyrva antigravity::tests::locked_or_missing_secret_service_is_nonfatal -- --nocapture`

- [ ] **Step 4: Implement Linux read only**

```rust
#[cfg(target_os = "linux")]
fn read_credential_raw() -> Option<Vec<u8>> {
    let entry = keyring::Entry::new("gemini", "antigravity").ok()?;
    entry.get_secret().ok()
        .or_else(|| entry.get_password().ok().map(String::into_bytes))
        .filter(|v| !v.is_empty())
}
```

Never call `set_password`, `set_secret`, `delete_credential`, or any write API.

- [ ] **Step 5: Keep source priority unchanged**

Bridge remains first. A keyring failure/lock/no-item becomes `None` and flow continues to existing transcript fallback. Credential bytes are decoded only long enough to make provider requests and are not persisted.

- [ ] **Step 6: Verify lock/check/tests and commit**

```bash
cargo check --workspace
cargo test -p nyrva antigravity::tests -- --nocapture
git diff --check
```

Commit: `feat(antigravity): read Linux Secret Service credential`

---

### Task 3: Centralize transcript root and prove fallback/redaction behavior

**Files:** `nyrva/src/antigravity.rs`, `nyrva/src/activity.rs` only if it independently constructs the same root

**Interfaces:**
- `fn state_root_from(home: Option<&Path>) -> Option<PathBuf>`
- runtime `fn state_root() -> Option<PathBuf>` delegates to it
- `fn credential_error_note(kind: &str) -> String` takes only a classification, never secret bytes

- [ ] **Step 1: Write failing path tests**

```rust
#[test]
fn antigravity_root_is_under_gemini() {
    assert_eq!(
        state_root_from(Some(Path::new("/home/me"))),
        Some(PathBuf::from("/home/me/.gemini/antigravity"))
    );
    assert_eq!(state_root_from(None), None);
}
```

- [ ] **Step 2: Write secret-redaction test**

```rust
#[test]
fn credential_error_note_never_accepts_secret_material() {
    let note = credential_error_note("malformed");
    assert_eq!(note, "Antigravity credential malformed");
    assert!(!note.contains("token"));
}
```

Keep this formatter signature intentionally incapable of receiving raw credential data.

- [ ] **Step 3: Write transcript fallback parser/path regression using a temp tree**

```rust
#[test]
fn transcript_fallback_finds_model_step_under_state_root() {
    let base = std::env::temp_dir().join(format!("nyrva-ag-{}", std::process::id()));
    let root = base.join(".gemini/antigravity/brain/run/.system_generated/logs");
    std::fs::create_dir_all(&root).unwrap();
    let transcript = root.join("transcript.jsonl");
    std::fs::write(
        &transcript,
        "{\"created_at\":\"2026-09-09T12:00:00Z\",\"source\":\"MODEL\"}\n"
    ).unwrap();
    assert!(transcript.is_file());
    let _ = std::fs::remove_dir_all(base);
}
```

When wiring the existing fallback finder/parser, make its root injectable into a pure/testable helper rather than depending on the real home directory.

- [ ] **Step 4: Verify RED then GREEN**

Run: `cargo test -p nyrva antigravity::tests -- --nocapture`

- [ ] **Step 5: Verify activity uses the provider root/fallback semantics**

If `activity.rs` independently constructs `.gemini/antigravity`, replace it with `crate::antigravity::state_root()` or a shared pure path helper. Do not add a live IDE dependency to CI.

- [ ] **Step 6: Commit**

Commit: `test(antigravity): harden Linux fallback and redaction`

---

### Task 4: Milestone verification and PR gate

- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `git diff --check`
- [ ] `git grep -n 'read_credential_raw' -- nyrva/src/antigravity.rs` and verify Linux implementation is not a `None` stub.
- [ ] Require Windows + Linux CI green, including dbus/keyring compile and Linux package/sidecar verification.
- [ ] Open PR titled `M7: add Antigravity Linux parity` and stop before merge.
