# Nyrva Linux X11 Foundation Design

## Goal

Add the first functional Linux backend for Nyrva, targeting **Linux X11** on **Ubuntu 22.04 / Debian 12** as the supported build baseline, while preserving current Windows behavior.

This milestone provides the Linux application infrastructure needed to launch, pin, drag, open links/folders, detect locale, autostart, and install/run Claude hooks. It does **not** promise working Linux quota/activity adapters for Claude, Codex, Cursor, or Antigravity yet.

## Scope

### In scope

- compile the shared Tauri/Rust application for Linux;
- add `platform/linux/` matching the application-infrastructure facade introduced in M2;
- support X11 as the Linux display backend for the MVP;
- reuse the existing Tauri window placement algorithm for edge pinning;
- Linux shell integration via `xdg-open`;
- Linux locale detection from environment variables;
- XDG autostart through `~/.config/autostart/nyrva.desktop`;
- Linux-compatible Claude hook installation and hook launcher behavior;
- Linux-specific Tauri bundling config;
- Linux CI using the Tauri v2 system prerequisites;
- preserve Windows behavior and packaging.

### Explicitly out of scope

- Wayland support;
- GNOME extension / layer-shell support;
- making all four providers work on Linux;
- provider protocol/parsing changes;
- UI redesign;
- Linux desktop activity/focus parity beyond safe foundation behavior;
- new providers.

## Platform baseline

The Linux build baseline is **Ubuntu 22.04 / Debian 12**. Tauri v2 requires WebKitGTK 4.1 on Linux and recommends building on the oldest supported system to avoid raising the glibc floor. Required Debian/Ubuntu build packages include:

```text
libwebkit2gtk-4.1-dev
build-essential
curl
wget
file
libxdo-dev
libssl-dev
libayatana-appindicator3-dev
librsvg2-dev
```

Linux support in this milestone assumes an X11 session. Wayland sessions are not a supported runtime target yet.

## Architecture

The M2 facade becomes dual-platform:

```text
nyrva/src/platform/
├── mod.rs
├── windows/
│   └── ...
└── linux/
    ├── mod.rs
    ├── autostart.rs
    ├── focus.rs
    ├── locale.rs
    ├── shell.rs
    └── window.rs
```

`platform/mod.rs` selects the implementation at compile time:

```rust
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
```

The shared application continues to own Tauri-level window size/position logic. OS-specific operations remain behind the platform facade.

## Linux window behavior

### Edge pinning

`main.rs::place_notch` continues to use Tauri monitor geometry and `set_position`. On X11 this is the supported MVP path.

No native Wayland positioning workaround is introduced.

### Dragging

The shared drag loop needs `platform::left_button_down()`. On Linux X11 this will use the X11/XTest-compatible tooling already represented by Tauri's Linux prerequisite `libxdo` via the Rust `x11rb`/Xlib-compatible approach chosen by implementation, with the smallest dependency that can reliably query pointer button state.

If the required low-level query cannot be implemented without adding a large dependency, the accepted fallback is to keep drag controlled by Tauri/DOM pointer events for Linux only, but edge pinning must still work.

### No-activate behavior

Windows-specific `WS_EX_NOACTIVATE` has no direct Linux equivalent. `platform::apply_noactivate` on Linux is a best-effort no-op in M3 because the Tauri window is already configured with `focus: false`, `skipTaskbar: true`, `decorations: false`, and `alwaysOnTop: true`.

### Console attachment

`platform::attach_parent_console()` is a no-op on Linux.

## Linux shell integration

`platform/linux/shell.rs` uses:

```text
xdg-open <path-or-url>
```

Commands are detached from stdin/stdout/stderr and failures remain non-fatal, matching Windows convenience-operation semantics.

## Linux locale

`platform/linux/locale.rs` resolves locale in this order:

1. `LC_ALL`
2. `LC_MESSAGES`
3. `LANG`
4. fallback `en`

Only the language prefix matters for the existing app translations:

- `zh*` -> `zh`
- `ja*` -> `ja`
- `ko*` -> `ko`
- everything else -> `en`

## Linux autostart

`platform/linux/autostart.rs` implements XDG desktop autostart.

Path:

```text
~/.config/autostart/nyrva.desktop
```

Desktop entry:

```ini
[Desktop Entry]
Type=Application
Name=Nyrva
Exec=/absolute/path/to/nyrva --silent
Terminal=false
X-GNOME-Autostart-enabled=true
```

Rules:

- create `~/.config/autostart` when enabling;
- quote/escape the executable path safely for the desktop entry;
- `is_enabled` checks for a valid Nyrva entry;
- disable removes only `nyrva.desktop`;
- missing file on disable is treated as success.

## Claude hooks on Linux

The current hook installation is Windows-specific because it assumes `nyrva-hook.exe`.

M3 makes hook executable resolution platform-aware:

- Windows: sibling `nyrva-hook.exe`;
- Linux: sibling `nyrva-hook`.

The generated Claude hook command continues to use an absolute executable path.

The hook helper becomes cross-platform for config lookup and app startup:

### Config path

- Windows: current APPDATA-compatible path remains supported;
- Linux: `${XDG_CONFIG_HOME:-~/.config}/nyrva/config.json`.

### Main executable

- Windows: `nyrva.exe`;
- Linux: `nyrva`.

### Parent PID

The existing Unix `std::os::unix::process::parent_id()` path is retained.

### Spawn

Linux spawning uses `Command` with null stdio and no Windows creation flags.

## Focus/session behavior

M3 does not attempt full Linux parity for terminal focus.

`platform/linux/focus.rs` supplies safe compatibility operations so shared code compiles and behaves conservatively:

- `focus_terminal` -> `false`;
- `focus_claude_desktop` -> `false`;
- foreground/process-map helpers return safe empty/default values where needed by shared session acknowledgement logic.

This avoids fake success and defers real X11 focus/process-tree behavior to the later activity/provider milestone.

## Provider behavior in M3

The app may still discover/read provider data where existing code is already portable, but **M3 does not claim Linux provider support**.

Provider-specific Windows code remains guarded by `#[cfg(windows)]`. Non-Windows fallbacks already present in provider modules remain unchanged unless compilation requires a narrow compatibility fix.

No provider endpoint, payload, parsing, polling interval, or persistence format may be changed as part of M3.

## Tauri configuration

Keep common UI/window settings in `nyrva/tauri.conf.json`.

Move Windows-only bundle target to `nyrva/tauri.windows.conf.json`:

```json
{
  "bundle": {
    "targets": ["nsis"],
    "icon": ["icons/icon.ico"]
  }
}
```

Add `nyrva/tauri.linux.conf.json` for Linux bundle settings. The initial Linux targets are `deb` and `appimage` if the repository already contains compatible PNG icons; otherwise M3 may limit itself to build/check configuration and add packaging only once Linux icon assets are present.

The common config must not force an `.ico` icon on Linux.

## CI

Extend `.github/workflows/ci.yml` with a Linux job pinned to Ubuntu 22.04.

The job installs the official Tauri v2 Debian/Ubuntu prerequisites and runs:

```text
cargo check --workspace
cargo test --workspace
```

Windows CI remains unchanged.

Linux build CI is the primary compile-time acceptance gate for this milestone.

## Validation

M3 is accepted when:

1. Windows workspace still checks/tests successfully in its existing job;
2. Ubuntu 22.04 workspace checks/tests successfully;
3. Linux compilation does not depend on Windows-only imports or executable suffixes in shared infrastructure;
4. Linux platform facade exists and is selected by `cfg(target_os = "linux")`;
5. Linux autostart logic is unit-tested without mutating the real user autostart directory;
6. locale parsing is unit-tested;
7. hook path/config resolution is unit-tested where practical;
8. no provider HTTP/parsing logic changes are present in the diff.

Manual X11 smoke test checklist:

```text
Nyrva launches
window appears on right edge
tray icon/menu appears
Open data folder works
provider page links open
Start at sign-in creates/removes nyrva.desktop
Claude hook install writes nyrva-hook path
nyrva-hook can launch/connect to nyrva
```

## Error handling

Linux infrastructure follows current application semantics:

- shell/focus/window convenience failures are non-fatal;
- autostart and hook installation return descriptive `Result<String, String>` errors;
- missing optional XDG environment variables fall back to standard paths;
- Wayland is not silently presented as supported.

## Future milestones

After M3, Linux provider enablement proceeds separately for:

1. Claude;
2. Codex;
3. Cursor;
4. Antigravity;

Wayland support remains after Linux X11/provider parity.