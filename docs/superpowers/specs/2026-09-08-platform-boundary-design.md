# Nyrva Platform Boundary Design

## Goal

Prepare the Nyrva Rust/Tauri application for Linux X11 without adding Linux behavior yet by moving Windows-only operating-system code behind a single `platform` boundary.

## Scope

This milestone is a refactor only. Windows behavior must remain unchanged. Provider protocols, parsing, polling, UI behavior, and persistence formats are not to be redesigned.

The implementation is stacked on top of `feat/nyrva-foundation`.

## Chosen approach

Use a small module facade rather than introducing a large trait/object graph now.

```text
nyrva/src/
├── platform/
│   ├── mod.rs
│   └── windows/
│       ├── mod.rs
│       ├── autostart.rs
│       ├── focus.rs
│       ├── process.rs
│       ├── shell.rs
│       └── window.rs
├── main.rs
├── activity.rs
├── antigravity.rs
├── diag.rs
├── glyphs.rs
└── ...
```

`platform/mod.rs` is the only OS-facing API consumed by shared code. On Windows it delegates to `platform::windows`. Linux modules are deliberately not added in this milestone.

This is preferred over a trait-heavy design because the application currently has one concrete platform implementation and many operations are process-global functions. A trait layer would add indirection without yet providing useful runtime polymorphism. If Linux later needs injectable behavior for tests, traits can be introduced around the specific seams that need them.

## Platform responsibilities

### `platform/windows/window.rs`

Own Win32 window/input details:

- left mouse button state used by notch dragging;
- `WS_EX_NOACTIVATE` / `WS_EX_TOOLWINDOW` application;
- direct Windows window/input calls.

Tauri-level monitor sizing and `set_position` remain in shared `main.rs` for now because the same API is expected to be reused by the Linux X11 implementation. Only direct Win32 calls move.

### `platform/windows/shell.rs`

Own Windows shell integration:

- open a directory with Explorer;
- open a URL using the Windows shell;
- hide console windows for spawned helper commands where required.

Shared code must no longer invoke `explorer`, `cmd /C start`, or `CommandExt::creation_flags` directly.

### `platform/windows/autostart.rs`

Move the current HKCU `Run` implementation here unchanged in behavior. Shared tray/command code calls a neutral `platform::autostart::{is_enabled, enable, disable}` facade.

### `platform/windows/process.rs`

Own Windows process inspection primitives that are currently spread across focus/activity/diagnostic helpers:

- process snapshot (`pid`, `ppid`, executable name);
- foreground PID;
- ancestor-chain helper;
- Windows process/window lookup primitives needed by focus and activity logic;
- low-priority/thread/process helpers that require Win32 APIs.

Pure provider classification remains outside this module.

### `platform/windows/focus.rs`

Own Windows-specific terminal and Claude Desktop focus behavior. Shared code calls neutral platform functions such as `focus_terminal(pid)` and `focus_claude_desktop()`.

## Shared-code rules

After this milestone:

- `main.rs` contains no direct `windows::Win32::*` imports;
- `main.rs` does not directly spawn `explorer` or `cmd /C start`;
- top-level `autostart.rs` and `focus.rs` are removed in favor of the platform facade;
- providers are not moved wholesale into the platform layer;
- cross-platform paths based on `dirs` stay shared unless their actual path differs by OS;
- no Linux implementation or fake Linux stubs are added solely to make the architecture look complete.

## Error behavior

Platform operations preserve the current failure model:

- UI convenience operations (open URL/folder, focus) fail quietly or return `false` as they do today;
- autostart retains `Result<String, String>` responses;
- process discovery returns empty/default data when OS inspection fails;
- no new user-visible error states are introduced.

## Testing and verification

Because this is behavior-preserving refactoring, tests focus on pure logic extracted from Win32 calls plus build validation.

Add unit tests for the ancestor-chain/process matching helper so that logic can be exercised without Win32.

Required Windows verification from repo root:

```text
cargo check --workspace
cargo test --workspace
```

CI also checks that direct `windows::Win32` imports do not appear outside `nyrva/src/platform/windows/`.

## Non-goals

- Linux X11 implementation;
- Wayland support;
- changing the notch placement algorithm;
- changing providers or provider endpoints;
- UI redesign;
- dynamic runtime platform selection;
- adding a general-purpose dependency-injection framework.
