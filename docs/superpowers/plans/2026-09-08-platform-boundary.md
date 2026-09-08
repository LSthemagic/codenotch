# Nyrva Platform Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Windows-only OS integration behind `nyrva/src/platform/` without changing current Windows behavior or provider protocols.

**Architecture:** Shared application/provider code calls a small `platform` facade. The concrete Windows implementation lives in `platform/windows/`; pure process-tree logic remains in the facade so it is testable and reusable by the later Linux backend. No Linux backend is added in this milestone.

**Tech Stack:** Rust 2021, Tauri 2, `windows` crate 0.58, GitHub Actions Windows runner.

**Spec:** `docs/superpowers/specs/2026-09-08-platform-boundary-design.md`

## Global Constraints

- Preserve Windows runtime behavior.
- Do not change Claude, Codex, Cursor, or Antigravity protocol/parsing semantics.
- Do not redesign the UI.
- Do not implement Linux or Wayland in this milestone.
- Keep direct `windows::Win32` imports under `nyrva/src/platform/windows/`.
- Keep failure semantics unchanged: focus/shell convenience operations fail quietly; autostart returns `Result<String, String>`.

---

### Task 1: Introduce the platform facade and process-tree tests

**Files:**
- Create: `nyrva/src/platform/mod.rs`
- Create: `nyrva/src/platform/windows/mod.rs`
- Modify: `nyrva/src/main.rs`

**Interfaces:**
- Produces: `platform::ProcMaps { ppid, name }`
- Produces: `platform::chain_of(pid, &ppid) -> Vec<u32>`
- Produces: `platform::pid_hits_chain(pid, chain, maps) -> bool`
- Re-exports concrete Windows OS operations from `platform::windows`.

- [ ] **Step 1: Add unit tests for process ancestry before moving implementations**

Tests cover direct ancestry, duplicate/cycle termination, and the conhost-style parent match used by focus detection.

- [ ] **Step 2: Verify the tests initially fail because the platform module/functions do not exist**

Run on Windows:

```text
cargo test -p nyrva platform::tests
```

Expected: compile failure for missing platform API.

- [ ] **Step 3: Add `platform/mod.rs` with the minimal pure helpers and Windows module export**

The pure helpers must not import Win32 APIs.

- [ ] **Step 4: Re-run the focused tests**

Expected: PASS.

---

### Task 2: Move window, shell, locale, and autostart integration

**Files:**
- Create: `nyrva/src/platform/windows/window.rs`
- Create: `nyrva/src/platform/windows/shell.rs`
- Create: `nyrva/src/platform/windows/locale.rs`
- Create: `nyrva/src/platform/windows/autostart.rs`
- Modify: `nyrva/src/platform/windows/mod.rs`
- Modify: `nyrva/src/main.rs`
- Modify: `nyrva/src/tray.rs`
- Modify: `nyrva/src/i18n.rs`
- Delete: `nyrva/src/autostart.rs`

**Interfaces:**
- `left_button_down() -> bool`
- `apply_noactivate(&AppHandle)`
- `attach_parent_console()`
- `open_folder(&Path)`
- `open_url(&str)`
- `run_hidden(program, args) -> std::process::Output` or equivalent neutral helper
- `system_language() -> &'static str`
- `autostart::{is_enabled, enable, disable}`

- [ ] **Step 1: Move the existing Win32/window and shell code without semantic changes**
- [ ] **Step 2: Replace `main.rs` and tray direct Windows commands with facade calls**
- [ ] **Step 3: Replace `i18n.rs` direct `GetUserDefaultLocaleName` use with `platform::system_language()`**
- [ ] **Step 4: Move HKCU Run logic into `platform/windows/autostart.rs` and update tray references**
- [ ] **Step 5: Run `cargo check --workspace` on Windows**

Expected: PASS.

---

### Task 3: Move process/focus and low-level provider OS primitives

**Files:**
- Create: `nyrva/src/platform/windows/process.rs`
- Create: `nyrva/src/platform/windows/focus.rs`
- Create: `nyrva/src/platform/windows/credentials.rs`
- Create: `nyrva/src/platform/windows/icons.rs`
- Modify: `nyrva/src/platform/windows/mod.rs`
- Modify: `nyrva/src/activity.rs`
- Modify: `nyrva/src/antigravity.rs`
- Modify: `nyrva/src/glyphs.rs`
- Modify: callers of `crate::focus::*`
- Delete: `nyrva/src/focus.rs`

**Interfaces:**
- `proc_maps() -> ProcMaps`
- `foreground_pid() -> u32`
- `focus_terminal(pid) -> bool`
- `focus_claude_desktop() -> bool`
- `lower_thread_priority()`
- `process_io_bytes(pid) -> Option<(u64, u64)>`
- `read_generic_credential(target) -> Option<Vec<u8>>`
- `extract_exe_icon_rgba(path, size) -> Option<(u32, u32, Vec<u8>)>`

- [ ] **Step 1: Move ToolHelp/foreground/focus Win32 code into the platform implementation**
- [ ] **Step 2: Keep activity classification in `activity.rs`; replace only Win32 IO/priority/process primitives with platform calls**
- [ ] **Step 3: Keep Antigravity quota logic in `antigravity.rs`; replace Credential Manager access with `read_generic_credential`**
- [ ] **Step 4: Keep glyph selection/PNG encoding in `glyphs.rs`; replace executable icon extraction with the platform icon primitive**
- [ ] **Step 5: Run focused unit tests and `cargo check --workspace`**

Expected: PASS.

---

### Task 4: Remove remaining infrastructure-level Windows command leakage and add CI guard

**Files:**
- Modify: `nyrva/src/diag.rs`
- Modify: other shared files only where they directly use Windows process/shell primitives
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Shared diagnostics may call `platform::run_hidden`/process-query helpers but keep their formatting and privacy behavior unchanged.

- [ ] **Step 1: Replace diagnostic `powershell` + Windows `CommandExt` boilerplate with platform helpers**
- [ ] **Step 2: Add a CI step that scans `nyrva/src` and fails when `windows::Win32` occurs outside `nyrva/src/platform/windows/`**
- [ ] **Step 3: Run the full workspace verification**

```text
cargo check --workspace
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 4: Review the diff to ensure provider HTTP endpoints, request bodies, usage parsing, polling intervals, and UI markup were not changed**

---

### Task 5: Open a stacked PR

**Files:**
- No production files.

- [ ] **Step 1: Open `feat/nyrva-platform-boundary` against `feat/nyrva-foundation`**
- [ ] **Step 2: Record validation status truthfully; if GitHub Actions is unavailable, explicitly mark compilation/tests as unverified rather than claiming success**
