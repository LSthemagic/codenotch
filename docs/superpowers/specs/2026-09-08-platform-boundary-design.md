# Nyrva Platform Boundary Design

## Goal

Prepare the Nyrva Rust/Tauri application for Linux X11 without adding Linux behavior yet by moving **application infrastructure** that is directly tied to Windows behind a single `platform` boundary.

## Scope

This milestone is a behavior-preserving refactor. Windows behavior must remain unchanged. Provider protocols, parsing, polling, UI behavior, and persistence formats are not redesigned.

The implementation is stacked on top of `feat/nyrva-foundation`.

During repository inspection, the Windows coupling separated naturally into two groups:

1. application infrastructure: window behavior, shell/opening, focus, locale, autostart and console attachment;
2. provider-specific adapters: Antigravity Credential Manager access, EXE icon extraction and activity IO counters.

This milestone extracts group 1. Group 2 stays with each provider until its Linux provider implementation is introduced, avoiding a premature move of provider logic into a generic platform package.

## Chosen approach

Use a small module facade rather than introducing a trait/object graph now.

```text
nyrva/src/
├── platform/
│   ├── mod.rs
│   └── windows/
│       ├── mod.rs
│       ├── autostart.rs
│       ├── focus.rs
│       ├── locale.rs
│       ├── shell.rs
│       └── window.rs
├── main.rs
├── tray.rs
├── i18n.rs
└── ...
```

`platform/mod.rs` is the shared OS-facing facade. On Windows it delegates to `platform::windows`. Linux modules are deliberately not added in this milestone.

This is preferred over a trait-heavy design because there is currently one concrete implementation and these operations are process-global functions. Runtime polymorphism would add indirection without useful capability yet.

## Platform responsibilities

### `platform/windows/window.rs`

Own direct Win32 window/input details:

- left mouse button state used by notch dragging;
- `WS_EX_NOACTIVATE` / `WS_EX_TOOLWINDOW` application;
- parent-console attachment.

Tauri monitor sizing and `set_position` remain in shared `main.rs` because the same high-level API is expected to be reused by Linux X11.

### `platform/windows/shell.rs`

Own Windows shell integration:

- open a directory with Explorer;
- open a URL through the Windows shell;
- Windows-only `CommandExt::creation_flags` needed for these commands.

Shared application infrastructure must no longer invoke `explorer` or `cmd /C start` directly.

### `platform/windows/autostart.rs`

Move the current HKCU `Run` implementation here unchanged in behavior. Shared code calls `platform::autostart::{is_enabled, enable, disable}`.

### `platform/windows/focus.rs`

Own the existing Windows terminal/Claude Desktop focus code and its process/window snapshot helpers. This file is moved as one cohesive Windows implementation to avoid changing its runtime algorithm in the same commit as the architecture refactor.

### `platform/windows/locale.rs`

Own `GetUserDefaultLocaleName`. `i18n.rs` remains responsible for mapping the resulting language to application translations.

## Shared-code rules

After this milestone:

- `main.rs` contains no direct `windows::Win32::*` imports;
- `main.rs` and `tray.rs` do not directly spawn `explorer` or `cmd /C start`;
- application focus/autostart/locale/window integration is reached through `platform`;
- provider files are not moved wholesale into the platform layer;
- cross-platform paths based on `dirs` stay shared;
- no Linux implementation or fake Linux stubs are added solely to make the architecture look complete.

Provider-specific Windows adapters that remain are documented debt for the Linux-provider milestones, not hidden as completed work.

## Error behavior

Platform operations preserve the current failure model:

- open URL/folder and focus operations fail quietly or return `false` as today;
- autostart retains `Result<String, String>`;
- focus/process discovery returns empty/default data on OS inspection failure;
- no new user-visible error states are introduced.

## Verification

Required Windows verification from repository root:

```text
cargo check --workspace
cargo test --workspace
```

Review additionally verifies that `main.rs`, `tray.rs`, and `i18n.rs` no longer contain direct Win32/shell integration.

Because GitHub Actions has not produced a run for the foundation PR yet, verification status must be reported truthfully; no passing-build claim is allowed without actual output.

## Non-goals

- Linux X11 implementation;
- Wayland support;
- provider-specific Credential Manager / EXE icon / IO-counter extraction;
- changing the notch placement algorithm;
- changing providers or provider endpoints;
- UI redesign;
- dynamic runtime platform selection;
- dependency-injection framework.
