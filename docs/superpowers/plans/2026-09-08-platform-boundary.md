# Nyrva Platform Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Windows-specific application infrastructure behind `nyrva/src/platform/` without changing Windows behavior or provider protocols.

**Architecture:** Shared app code calls a small `platform` facade. The Windows implementation lives in `platform/windows/`. Provider-specific Windows adapters remain in their provider files until the Linux implementation of each provider is added.

**Tech Stack:** Rust 2021, Tauri 2, `windows` crate 0.58.

**Spec:** `docs/superpowers/specs/2026-09-08-platform-boundary-design.md`

## Global Constraints

- Preserve Windows runtime behavior.
- Do not change Claude, Codex, Cursor, or Antigravity protocol/parsing semantics.
- Do not redesign the UI.
- Do not implement Linux or Wayland in this milestone.
- Remove direct Win32/shell integration from `main.rs`, `tray.rs`, and `i18n.rs`.
- Keep failure semantics unchanged.

---

### Task 1: Add the platform facade

**Files:**
- Create: `nyrva/src/platform/mod.rs`
- Create: `nyrva/src/platform/windows/mod.rs`

**Interfaces:**
- Re-export Windows application-infrastructure operations through `crate::platform`.

- [ ] Create the module structure.
- [ ] Keep the facade compile-time selected with `#[cfg(windows)]`; do not invent a Linux stub yet.

---

### Task 2: Move window, shell, locale, and autostart infrastructure

**Files:**
- Create: `nyrva/src/platform/windows/window.rs`
- Create: `nyrva/src/platform/windows/shell.rs`
- Create: `nyrva/src/platform/windows/locale.rs`
- Create: `nyrva/src/platform/windows/autostart.rs`
- Modify: `nyrva/src/platform/windows/mod.rs`
- Modify: `nyrva/src/main.rs`
- Modify: `nyrva/src/tray.rs`
- Modify: `nyrva/src/i18n.rs`

**Interfaces:**
- `left_button_down() -> bool`
- `apply_noactivate(&AppHandle)`
- `attach_parent_console()`
- `open_folder(&Path)`
- `open_url(&str)`
- `system_language() -> &'static str`
- `platform::autostart::{is_enabled, enable, disable}`

- [ ] Move direct Win32 window/input/console calls from `main.rs` into `window.rs`.
- [ ] Replace `explorer` and `cmd /C start` call sites in app infrastructure with `shell.rs` helpers.
- [ ] Move `GetUserDefaultLocaleName` from `i18n.rs` into `locale.rs`.
- [ ] Move the HKCU `Run` implementation into `platform/windows/autostart.rs` and update callers.

---

### Task 3: Move the existing Windows focus implementation as one cohesive unit

**Files:**
- Create: `nyrva/src/platform/windows/focus.rs`
- Modify: `nyrva/src/platform/windows/mod.rs`
- Replace: `nyrva/src/focus.rs` with a compatibility re-export shim.

**Interfaces:**
- `focus_terminal(pid) -> bool`
- `focus_claude_desktop() -> bool`
- `proc_maps() -> ProcMaps`
- `fg_pid() -> u32`
- `chain_of(...)`
- `pid_hits_chain(...)`

- [ ] Move the current focus/process-window implementation without changing its algorithm.
- [ ] Keep existing callers working through the thin shim so this refactor does not mix architecture changes with behavior changes.

---

### Task 4: Review and verify

**Files:**
- Review: `nyrva/src/main.rs`
- Review: `nyrva/src/tray.rs`
- Review: `nyrva/src/i18n.rs`
- Review: `nyrva/src/platform/windows/*`

- [ ] Confirm `main.rs` has no direct `windows::Win32` imports.
- [ ] Confirm `main.rs` / `tray.rs` do not directly spawn Explorer or `cmd /C start`.
- [ ] Confirm `i18n.rs` does not import Win32 globalization APIs.
- [ ] Confirm provider endpoints/request/parsing code is untouched.
- [ ] Run, when an actual Windows runner is available:

```text
cargo check --workspace
cargo test --workspace
```

If no workflow runner is available, report build/test status as unverified.

---

### Task 5: Open a stacked PR

- [ ] Open `feat/nyrva-platform-boundary` against `feat/nyrva-foundation`.
- [ ] Document that provider-specific Windows adapters are intentionally deferred to the Linux-provider milestones.
