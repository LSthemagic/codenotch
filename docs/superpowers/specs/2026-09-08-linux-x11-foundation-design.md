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
- Linux `.deb` and `.AppImage` bundle configuration;
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

The Linux build baseline is **Ubuntu 22.04 / Debian 12**. Tauri v2 requires WebKitGTK 4.1 on Linux and recommends building on the oldest supported system to avoid raising the glibc floor. Required Debian/Ubuntu build packages are:

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

The shared drag loop keeps its existing `platform::left_button_down() -> bool` contract.

Linux implements this with target-specific dependency:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.13"
```

`platform/linux/window.rs` connects to the active X11 display with `x11rb`, queries the root pointer state, and returns whether Button 1 is pressed. Connection/query failures return `false` and never crash the application.

This dependency is Linux-only and must not alter the Windows dependency graph.

### No-activate behavior

Windows-specific `WS_EX_NOACTIVATE` has no direct Linux equivalent. `platform::apply_noactivate` on Linux is a no-op in M3 because the Tauri window is already configured with `focus: false`, `skipTaskbar: true`, `decorations: false`, and `alwaysOnTop: true`.

### Console attachment

`platform::attach_parent_console()` is a no-op on Linux.

## Linux shell integration

`platform/linux/shell.rs` uses:

```text
xdg-open <path-or-url>
```

Commands use null stdin/stdout/stderr and failures remain non-fatal, matching Windows convenience-operation semantics.

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

Locale normalization is implemented as a pure function and unit-tested separately from environment lookup.

## Linux autostart

`platform/linux/autostart.rs` implements XDG desktop autostart.

Path:

```text
${XDG_CONFIG_HOME:-~/.config}/autostart/nyrva.desktop
```

Desktop entry:

```ini
[Desktop Entry]
Type=Application
Name=Nyrva
Exec="/absolute/path/to/nyrva" --silent
Terminal=false
X-GNOME-Autostart-enabled=true
```

Rules:

- use `XDG_CONFIG_HOME` when non-empty, otherwise `~/.config`;
- create the `autostart` directory when enabling;
- quote the executable path and escape `\`, `"`, backticks, and `$` for the desktop-entry Exec field;
- `is_enabled` requires the file to contain `Name=Nyrva` and an `Exec=` line;
- disable removes only `nyrva.desktop`;
- a missing file on disable is success;
- path/rendering helpers are pure and unit-tested with temporary paths rather than the real user directory.

## Claude hooks on Linux

The current hook installation is Windows-specific because it assumes `nyrva-hook.exe`.

M3 makes hook executable resolution platform-aware:

- Windows: sibling `nyrva-hook.exe`;
- Linux: sibling `nyrva-hook`.

The generated Claude hook command continues to use an absolute executable path.

The hook helper remains dependency-free and becomes cross-platform for config lookup and app startup.

### Config path

- Windows: `%APPDATA%/nyrva/config.json` as today;
- Linux: `${XDG_CONFIG_HOME:-$HOME/.config}/nyrva/config.json`.

### Main executable

- Windows: `nyrva.exe`;
- Linux: `nyrva`.

### Parent PID

The existing Unix `std::os::unix::process::parent_id()` implementation is retained.

### Spawn

Linux spawning uses `Command` with null stdio and no Windows creation flags.

Path resolution functions are split from execution so they can be unit-tested without spawning the app.

## Focus/session behavior

M3 does not attempt full Linux parity for terminal focus.

`platform/linux/focus.rs` provides the same exported API as the Windows module so shared compatibility shims compile:

```text
ProcMaps
proc_maps()
fg_pid()
chain_of()
pid_hits_chain()
focus_terminal()
focus_claude_desktop()
```

Behavior in M3:

- `ProcMaps` contains empty `ppid`/`name` maps;
- `proc_maps()` returns the empty maps;
- `fg_pid()` returns `0`;
- `chain_of()` keeps the same pure ancestry algorithm as Windows so later Linux process discovery can reuse it;
- `pid_hits_chain()` keeps the same pure matching rule;
- both focus functions return `false`.

This avoids fake focus success and defers real X11 process/window focus behavior.

## Provider behavior in M3

The app may still discover/read provider data where existing code is already portable, but **M3 does not claim Linux provider support**.

Provider-specific Windows code remains guarded by `#[cfg(windows)]`. Non-Windows fallbacks already present in provider modules remain unchanged unless Linux compilation requires a narrow compatibility fix.

No provider endpoint, payload, parsing, polling interval, or persistence format may be changed as part of M3.

## Tauri configuration and packaging

Keep shared application/window settings in `nyrva/tauri.conf.json`, but remove platform-specific bundle targets/icons from it.

Add `nyrva/tauri.windows.conf.json`:

```json
{
  "bundle": {
    "targets": ["nsis"],
    "icon": ["icons/icon.ico"]
  }
}
```

Add `nyrva/tauri.linux.conf.json`:

```json
{
  "bundle": {
    "targets": ["deb", "appimage"],
    "icon": ["icons/icon.png"]
  }
}
```

M3 creates `nyrva/icons/icon.png` by converting the existing project `icon.ico` asset, preserving the current visual identity. No new logo/design is introduced.

Platform-specific config files rely on Tauri 2's documented config merge behavior.

## CI

Extend `.github/workflows/ci.yml` with a Linux job pinned to `ubuntu-22.04`.

Before Rust checks, install:

```bash
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

Then run from repository root:

```text
cargo check --workspace
cargo test --workspace
```

Windows CI remains unchanged.

## Validation

M3 is accepted when:

1. Windows workspace still checks/tests successfully in its existing job;
2. Ubuntu 22.04 workspace checks/tests successfully;
3. Linux compilation does not depend on Windows-only executable suffixes in shared infrastructure;
4. Linux platform facade exists and is selected by `cfg(target_os = "linux")`;
5. Linux autostart path/rendering behavior is unit-tested without mutating the real user autostart directory;
6. locale normalization is unit-tested;
7. hook path/config resolution is unit-tested;
8. no provider HTTP/parsing logic changes are present in the diff;
9. Linux Tauri packaging config defines `.deb` and `.AppImage` targets with a PNG app icon.

Manual X11 smoke test checklist:

```text
Nyrva launches
window appears on right edge
tray icon/menu appears
vertical drag works while Button 1 is held
Open data folder works
provider page links open
Start at sign-in creates/removes nyrva.desktop
Claude hook install writes nyrva-hook path
nyrva-hook can launch/connect to nyrva
```

## Error handling

Linux infrastructure follows current application semantics:

- X11 pointer-query, shell and focus failures are non-fatal;
- autostart and hook installation return descriptive `Result<String, String>` errors;
- missing optional XDG variables fall back to standard paths;
- Wayland is not silently presented as supported.

## Future milestones

After M3, Linux provider enablement proceeds separately for:

1. Claude;
2. Codex;
3. Cursor;
4. Antigravity;

Wayland support remains after Linux X11/provider parity.