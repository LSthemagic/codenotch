# Nyrva Linux X11 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a functional Linux X11 application backend for Nyrva on Ubuntu 22.04 / Debian 12 without claiming Linux provider parity yet.

**Architecture:** Extend the M2 compile-time `platform` facade with a Linux implementation for window/input, shell, locale, autostart, and safe focus stubs. Keep Tauri monitor placement shared. Make Claude hook installation/launcher path handling cross-platform, split Tauri bundle configuration by OS, and add Ubuntu 22.04 CI.

**Tech Stack:** Rust 2021, Tauri 2, WebKitGTK 4.1, X11 via `x11rb`, XDG autostart, `xdg-open`, GitHub Actions Ubuntu 22.04.

**Spec:** `docs/superpowers/specs/2026-09-08-linux-x11-foundation-design.md`

## Global Constraints

- Supported Linux baseline: Ubuntu 22.04 / Debian 12.
- Linux MVP display backend: X11 only.
- Wayland is explicitly unsupported in this milestone.
- Do not change provider endpoints, request payloads, polling cadence, quota parsing, or persistence formats.
- Do not claim Claude/Codex/Cursor/Antigravity Linux support yet.
- Preserve current Windows behavior and NSIS packaging.
- Linux packages: `deb` and `appimage`.
- Linux X11 mouse-button query uses `x11rb`.
- Keep common Tauri window settings in `tauri.conf.json`; OS bundle targets live in OS-specific configs.

---

### Task 1: Add the Linux platform module and locale tests

**Files:**
- Modify: `nyrva/src/platform/mod.rs`
- Create: `nyrva/src/platform/linux/mod.rs`
- Create: `nyrva/src/platform/linux/locale.rs`
- Create: `nyrva/src/platform/linux/focus.rs`

**Interfaces:**
- `platform::system_language() -> &'static str`
- `platform::focus_terminal(u32) -> bool`
- `platform::focus_claude_desktop() -> bool`
- `platform::ProcMaps { ppid, name }`
- `platform::proc_maps() -> ProcMaps`
- `platform::fg_pid() -> u32`
- `platform::chain_of(...) -> Vec<u32>`
- `platform::pid_hits_chain(...) -> bool`

- [ ] **Step 1: Write locale parsing tests**

Add pure tests in `locale.rs` around a helper:

```rust
fn normalize_locale(raw: &str) -> &'static str {
    let l = raw.to_ascii_lowercase();
    if l.starts_with("zh") { "zh" }
    else if l.starts_with("ja") { "ja" }
    else if l.starts_with("ko") { "ko" }
    else { "en" }
}

#[cfg(test)]
mod tests {
    use super::normalize_locale;

    #[test]
    fn normalizes_supported_locales() {
        assert_eq!(normalize_locale("zh_CN.UTF-8"), "zh");
        assert_eq!(normalize_locale("ja_JP.UTF-8"), "ja");
        assert_eq!(normalize_locale("ko_KR.UTF-8"), "ko");
        assert_eq!(normalize_locale("pt_BR.UTF-8"), "en");
    }
}
```

- [ ] **Step 2: Implement Linux locale precedence**

```rust
pub fn system_language() -> &'static str {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return normalize_locale(&v);
            }
        }
    }
    "en"
}
```

- [ ] **Step 3: Implement conservative focus compatibility**

Create Linux `ProcMaps` with empty maps, `fg_pid() -> 0`, focus functions returning `false`, and pure `chain_of` / `pid_hits_chain` helpers matching Windows semantics so shared code compiles without fake focus success.

- [ ] **Step 4: Wire Linux compile-time selection**

```rust
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
```

- [ ] **Step 5: Run focused tests on Linux**

```text
cargo test -p nyrva locale
```

Expected: PASS.

---

### Task 2: Add Linux shell and X11 window/input behavior

**Files:**
- Create: `nyrva/src/platform/linux/shell.rs`
- Create: `nyrva/src/platform/linux/window.rs`
- Modify: `nyrva/src/platform/linux/mod.rs`
- Modify: `nyrva/Cargo.toml`

**Interfaces:**
- `open_folder(&Path)`
- `open_url(&str)`
- `left_button_down() -> bool`
- `apply_noactivate(&tauri::AppHandle)`
- `attach_parent_console()`

- [ ] **Step 1: Add Linux-only `x11rb` dependency**

```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.13"
```

- [ ] **Step 2: Implement `xdg-open` helper**

```rust
fn open_target(target: &std::ffi::OsStr) {
    let _ = std::process::Command::new("xdg-open")
        .arg(target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
```

`open_folder` passes the path; `open_url` passes the URL.

- [ ] **Step 3: Implement X11 button query with `x11rb`**

Use `x11rb::connect(None)` and `query_pointer(root)`; return whether Button1 mask is set. Any connection/query error returns `false`.

- [ ] **Step 4: Implement Linux no-op window compatibility**

```rust
pub fn apply_noactivate(_app: &tauri::AppHandle) {}
pub fn attach_parent_console() {}
```

- [ ] **Step 5: Run Linux `cargo check -p nyrva`**

Expected: PASS.

---

### Task 3: Add XDG autostart with isolated tests

**Files:**
- Create: `nyrva/src/platform/linux/autostart.rs`
- Modify: `nyrva/src/platform/linux/mod.rs`

**Interfaces:**
- `autostart::is_enabled() -> bool`
- `autostart::enable() -> Result<String, String>`
- `autostart::disable() -> Result<String, String>`

- [ ] **Step 1: Add a pure desktop-entry renderer test**

```rust
#[test]
fn renders_nyrva_desktop_entry() {
    let s = desktop_entry(std::path::Path::new("/opt/Nyrva App/nyrva"));
    assert!(s.contains("Name=Nyrva"));
    assert!(s.contains("Terminal=false"));
    assert!(s.contains("--silent"));
}
```

- [ ] **Step 2: Add path resolution helper**

Use `${XDG_CONFIG_HOME}/autostart/nyrva.desktop` when `XDG_CONFIG_HOME` is set; otherwise `dirs::config_dir()/autostart/nyrva.desktop`.

- [ ] **Step 3: Implement safe Exec escaping**

Render the absolute executable path as one desktop-entry argument, escaping backslashes, quotes, backticks, and dollar signs according to Desktop Entry `Exec` quoting rules; append `--silent` outside the quoted executable.

- [ ] **Step 4: Implement enable/disable/is_enabled**

Enable creates the parent directory then writes exactly one Nyrva desktop entry. Disable removes only `nyrva.desktop`; missing file is success. `is_enabled` requires the file to contain both `Name=Nyrva` and an `Exec=` line containing `nyrva`.

- [ ] **Step 5: Test without touching the real user config**

Refactor internal functions to accept a base/config path for tests and use a temporary directory created under `std::env::temp_dir()` with a unique process/timestamp suffix.

- [ ] **Step 6: Run focused tests**

```text
cargo test -p nyrva autostart
```

Expected: PASS.

---

### Task 4: Make Claude hook installation and launcher cross-platform

**Files:**
- Modify: `nyrva/src/hooks_install.rs`
- Modify: `nyrva-hook/src/main.rs`
- Modify: `nyrva-hook/Cargo.toml` only if needed for `dirs`

**Interfaces:**
- Main app resolves sibling hook binary by OS.
- Hook resolves config path by OS.
- Hook resolves sibling main binary by OS.

- [ ] **Step 1: Add hook executable-name helper tests in `hooks_install.rs`**

```rust
#[test]
fn hook_binary_name_matches_platform() {
    #[cfg(windows)]
    assert_eq!(hook_binary_name(), "nyrva-hook.exe");
    #[cfg(target_os = "linux")]
    assert_eq!(hook_binary_name(), "nyrva-hook");
}
```

- [ ] **Step 2: Replace hard-coded `nyrva-hook.exe`**

```rust
fn hook_binary_name() -> &'static str {
    if cfg!(windows) { "nyrva-hook.exe" } else { "nyrva-hook" }
}
```

- [ ] **Step 3: Make `nyrva-hook` config-path lookup cross-platform**

Add `dirs = "5"` to `nyrva-hook` and resolve:

```rust
fn config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("nyrva").join("config.json"))
}
```

This preserves `%APPDATA%` behavior on Windows and becomes `${XDG_CONFIG_HOME:-~/.config}/nyrva/config.json` on Linux.

- [ ] **Step 4: Resolve main executable by OS**

```rust
fn main_binary_name() -> &'static str {
    if cfg!(windows) { "nyrva.exe" } else { "nyrva" }
}
```

Keep null stdio on both platforms and keep Windows creation flags under `#[cfg(windows)]` only.

- [ ] **Step 5: Run hook tests/check**

```text
cargo check -p nyrva-hook
cargo test -p nyrva
```

Expected: PASS.

---

### Task 5: Split Tauri bundle configuration for Windows and Linux

**Files:**
- Modify: `nyrva/tauri.conf.json`
- Create: `nyrva/tauri.windows.conf.json`
- Create: `nyrva/tauri.linux.conf.json`
- Create: `nyrva/icons/icon.png`

**Interfaces:**
- Common config owns product/window/security values.
- Windows override owns NSIS + `.ico`.
- Linux override owns `deb` + `appimage` + PNG icon.

- [ ] **Step 1: Remove OS-specific target/icon from common config**

Common `bundle` becomes:

```json
{ "active": true }
```

- [ ] **Step 2: Add Windows override**

```json
{
  "bundle": {
    "targets": ["nsis"],
    "icon": ["icons/icon.ico"]
  }
}
```

- [ ] **Step 3: Add Linux override**

```json
{
  "bundle": {
    "targets": ["deb", "appimage"],
    "icon": ["icons/icon.png"]
  }
}
```

- [ ] **Step 4: Add PNG icon derived from the existing project icon**

Generate a 512x512 PNG from the existing application artwork; do not introduce new branding/design work in M3.

- [ ] **Step 5: Validate config parsing during Linux and Windows checks**

Expected: Tauri build scripts accept the OS-specific config files.

---

### Task 6: Add Ubuntu 22.04 CI and Linux build gate

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Windows job remains unchanged.
- New Linux job runs on `ubuntu-22.04`.

- [ ] **Step 1: Add Linux prerequisite installation**

```yaml
- name: Install Tauri Linux dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev \
      build-essential \
      curl \
      wget \
      file \
      libxdo-dev \
      libssl-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev
```

- [ ] **Step 2: Add Linux workspace checks**

```yaml
- name: Cargo check
  run: cargo check --workspace
- name: Cargo test
  run: cargo test --workspace
```

- [ ] **Step 3: Run local/static validation available in the execution environment**

Check that shared files no longer assume `.exe` for hooks/app startup and Linux modules contain no Windows imports.

- [ ] **Step 4: Use GitHub Actions as the authoritative Linux compile/test gate**

Expected: Ubuntu 22.04 job exits 0 for both commands.

---

### Task 7: Review scope and open stacked PR

**Files:**
- No new production files.

- [ ] **Step 1: Compare against `feat/nyrva-platform-boundary`**

Verify no provider endpoint/request/parsing files changed except narrow compile guards if strictly required.

- [ ] **Step 2: Verify explicit non-goals**

No Wayland/layer-shell code, no provider feature claims, no UI redesign.

- [ ] **Step 3: Open `feat/nyrva-linux-x11-foundation` against `feat/nyrva-platform-boundary`**

PR body must state that Linux provider support is intentionally deferred.

- [ ] **Step 4: Report verification truthfully**

Only claim build/test success if the actual Windows/Linux commands or CI runs have exited 0.
