# M5 Codex Linux Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Codex usage and activity discovery fully Linux-aware while preserving Windows behavior and the existing read-only integration.

**Architecture:** Keep network/parsing logic in `nyrva/src/codex.rs`, introduce deterministic platform/path helpers, and make `activity.rs` consume the same Codex root. Normal polling never launches Codex. The current UI has no non-Claude activity-focus action, so M5 does not invent one; it only ensures Codex Linux activity has no Windows-only focus dependency and remains compatible with the generic X11 platform layer.

**Tech Stack:** Rust 2021, Tauri 2, rusqlite 0.32, ureq 2, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md`

## Global Constraints

- Windows + Linux X11 only; no Wayland.
- Preserve Windows behavior.
- Non-empty `CODEX_HOME` overrides `$HOME/.codex`.
- Provider auth/state remains read-only and secret values never enter logs/UI/Nyrva snapshots.
- No new provider login flow, UI redesign, or Codex background process.
- New platform/path behavior uses TDD.

---

### Task 1: Centralize Codex home and all provider-owned paths

**Files:** `nyrva/src/codex.rs`, `nyrva/src/activity.rs`

**Interfaces:**
- `fn codex_home_from(env_home: Option<&OsStr>, user_home: Option<&Path>) -> Option<PathBuf>`
- `pub(crate) fn codex_home() -> Option<PathBuf>`
- `fn codex_activity_paths(home: &Path) -> (PathBuf, PathBuf)`

- [ ] **Step 1: Write failing resolver tests**

```rust
#[test]
fn codex_home_prefers_non_empty_env() {
    assert_eq!(
        codex_home_from(Some(OsStr::new("/tmp/custom")), Some(Path::new("/home/me"))),
        Some(PathBuf::from("/tmp/custom"))
    );
}

#[test]
fn codex_home_falls_back_to_dot_codex() {
    assert_eq!(
        codex_home_from(Some(OsStr::new("")), Some(Path::new("/home/me"))),
        Some(PathBuf::from("/home/me/.codex"))
    );
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva codex::tests::codex_home_ -- --nocapture`

Expected: compile failure because `codex_home_from` does not exist.

- [ ] **Step 3: Implement the resolver**

```rust
fn codex_home_from(env_home: Option<&OsStr>, user_home: Option<&Path>) -> Option<PathBuf> {
    if let Some(raw) = env_home.filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(raw));
    }
    user_home.map(|h| h.join(".codex"))
}

pub(crate) fn codex_home() -> Option<PathBuf> {
    let env_home = std::env::var_os("CODEX_HOME");
    let home = dirs::home_dir();
    codex_home_from(env_home.as_deref(), home.as_deref())
}
```

Make `auth_path()` derive `auth.json`, `newest_rollout()` derive `sessions/`, and optional `bin/` lookup derive from this root.

- [ ] **Step 4: Make activity use the same root**

```rust
fn codex_activity_paths(home: &Path) -> (PathBuf, PathBuf) {
    (home.join("thread_history_1.sqlite"), home.join("state_5.sqlite"))
}
```

`Ctx::new()` must use `crate::codex::codex_home()` for `thread_history_1.sqlite`; `codex_turns_in_progress()` must use the same root for `state_5.sqlite`.

- [ ] **Step 5: Add path-sharing regression**

```rust
#[test]
fn codex_activity_paths_share_resolved_home() {
    let root = Path::new("/tmp/codex-home");
    let (turns, names) = codex_activity_paths(root);
    assert_eq!(turns, root.join("thread_history_1.sqlite"));
    assert_eq!(names, root.join("state_5.sqlite"));
}
```

- [ ] **Step 6: Verify GREEN and commit**

Run: `cargo test -p nyrva codex::tests activity::tests -- --nocapture`

Commit: `feat(codex): centralize provider home paths`

---

### Task 2: Make executable discovery platform-aware

**Files:** `nyrva/src/codex.rs`

**Interfaces:**
- `fn executable_candidates(codex_home: Option<&Path>, config_dir: Option<&Path>, path: Option<&OsStr>) -> Vec<PathBuf>`
- `pub fn find_executable() -> Option<PathBuf>`

- [ ] **Step 1: Write Linux candidate test**

```rust
#[cfg(target_os = "linux")]
#[test]
fn linux_candidates_include_home_and_path() {
    let got = executable_candidates(
        Some(Path::new("/home/me/.codex")),
        Some(Path::new("/home/me/.config")),
        Some(OsStr::new("/usr/local/bin:/usr/bin")),
    );
    assert_eq!(got[0], PathBuf::from("/home/me/.codex/bin/codex"));
    assert!(got.contains(&PathBuf::from("/usr/local/bin/codex")));
    assert!(got.contains(&PathBuf::from("/usr/bin/codex")));
}
```

Also add Windows-only regression asserting existing `codex.exe`/`codex.cmd` candidates remain.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva codex::tests::linux_candidates -- --nocapture`

- [ ] **Step 3: Implement candidate generation**

Linux adds `$CODEX_HOME/bin/codex` then every `PATH` entry joined with `codex`. Windows retains npm/vendor plus `.exe`/`.cmd`. No shell/npm/Codex process is spawned.

- [ ] **Step 4: Make `find_executable()` a thin filter**

```rust
pub fn find_executable() -> Option<PathBuf> {
    let home = codex_home();
    let config = dirs::config_dir();
    let path = std::env::var_os("PATH");
    executable_candidates(home.as_deref(), config.as_deref(), path.as_deref())
        .into_iter()
        .find(|p| p.is_file())
}
```

- [ ] **Step 5: Verify GREEN and commit**

Run: `cargo test -p nyrva codex::tests -- --nocapture`

Commit: `feat(codex): support Linux executable discovery`

---

### Task 3: Remove Windows-only HTTP identity

**Files:** `nyrva/src/codex.rs`

**Interfaces:** `fn user_agent_for(os: &str) -> String`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn user_agent_is_platform_correct() {
    let linux = user_agent_for("linux");
    assert!(linux.starts_with("nyrva/"));
    assert!(linux.contains("linux"));
    assert!(!linux.contains("Windows"));
    assert!(!linux.contains("codenotch"));
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p nyrva codex::tests::user_agent_is_platform_correct -- --nocapture`

- [ ] **Step 3: Implement and wire**

```rust
fn user_agent_for(os: &str) -> String {
    format!("nyrva/{} ({os})", env!("CARGO_PKG_VERSION"))
}
```

`fetch_usage()` computes `let ua = user_agent_for(std::env::consts::OS);` and passes `&ua` to `User-Agent`.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test -p nyrva codex::tests -- --nocapture`

Commit: `fix(codex): use platform-correct Nyrva user agent`

---

### Task 4: Resolve Codex focus scope without inventing behavior

**Files:** inspect `nyrva/src/main.rs`, `nyrva/src/activity.rs`, `nyrva/src/platform/linux/focus.rs`; modify only if a Windows-only Codex dependency is found.

**Interfaces:** No new command/interface is produced. Existing `focus_session(id)` is Claude-session-specific; non-Claude `Activity` entries contain no PID and the UI has no activity-focus action.

- [ ] **Step 1: Verify the current interface boundary**

Run:

```bash
git grep -n 'focus_session\|focus_terminal\|focus_claude_desktop' -- nyrva/src nyrva/ui
```

Expected: `focus_session` is tied to Claude session IDs; Codex activity rendering does not call a Windows-only focus path.

- [ ] **Step 2: Add a code comment beside `Activity` or Codex activity construction**

Document that non-Claude activity is display/state only and that adding focus later requires a separately designed PID/window identity. Do not add fake PID inference or a new UI action in M5.

- [ ] **Step 3: Verify Linux platform abstraction remains generic**

Run: `cargo test -p nyrva platform::linux::focus::tests -- --nocapture`

Expected: existing X11 process-chain/window tests PASS.

- [ ] **Step 4: Commit only if a comment/code adjustment was needed**

Commit: `docs(codex): clarify Linux activity focus boundary`

---

### Task 5: Milestone verification and PR gate

- [ ] **Step 1:** `cargo check --workspace`
- [ ] **Step 2:** `cargo test --workspace`
- [ ] **Step 3:** `git diff --check`
- [ ] **Step 4:** `git grep -n '\.codex' -- nyrva/src/codex.rs nyrva/src/activity.rs` and verify runtime path construction is centralized.
- [ ] **Step 5:** push the M5 branch and require Windows + Linux CI, including existing Linux bundle/sidecar verification.
- [ ] **Step 6:** open PR titled `M5: add Codex Linux parity` and stop before merge.
