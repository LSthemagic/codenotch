# Nyrva

Nyrva is a lightweight desktop usage monitor for AI coding assistants. It pins provider usage and session activity to the edge of your screen so limits stay visible without interrupting your workflow.

## Providers

The MVP supports the same four providers on the supported desktop platforms:

- Claude Code
- Codex
- Cursor
- Antigravity

Provider-owned credentials and state are read-only from Nyrva's point of view.

## Platforms

### Windows

Windows is supported and distributed as an NSIS installer. The installer includes the `nyrva-hook` helper used by the Claude Code integration.

### Linux

Linux X11 is supported. Ubuntu 22.04 and Debian 12 are the MVP baseline. Builds are distributed as both `.deb` and AppImage packages, including the `nyrva-hook` helper.

Wayland is intentionally deferred because reliable global edge positioning and focus behavior are compositor-specific.

## Installable builds

Every successful `main` CI run publishes downloadable artifacts:

- `nyrva-windows` — Windows NSIS installer (`.exe`)
- `nyrva-linux` — Debian package (`.deb`) and AppImage

Open the latest successful GitHub Actions **CI** run and download the artifact for your platform.

## Architecture

Nyrva is built with Rust and Tauri 2. The UI remains the compact edge-notch interface from the existing Windows port for the MVP.

```text
Cargo.toml
├── nyrva/       # Tauri desktop application
└── nyrva-hook/  # Claude Code hook launcher
```

OS-specific behavior is isolated behind the platform layer, while provider adapters live in the desktop crate.

## Development

From the repository root:

```bash
cargo check --workspace
cargo test --workspace
```

### Windows package

With the normal Tauri 2 Windows prerequisites/WebView2 environment installed:

```powershell
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/prepare-windows-sidecar.ps1 release
cd nyrva
cargo tauri build
```

### Linux X11 packages

Install the normal Tauri 2 Linux prerequisites for your distribution, then:

```bash
cargo install tauri-cli --version '^2.0.0' --locked
bash scripts/prepare-linux-sidecar.sh release
cd nyrva
cargo tauri build
```

The Linux Tauri config generates both `.deb` and AppImage packages.

## MVP status

- Nyrva foundation ✅
- Cross-platform OS boundary ✅
- Linux X11 foundation ✅
- Claude Code Linux parity ✅
- Codex Linux parity ✅
- Cursor Linux parity ✅
- Antigravity Linux parity ✅
- Windows/Linux packaging and downloadable CI artifacts ✅

Wayland remains post-MVP work.

## License and attribution

Nyrva is MIT licensed and is based on MIT-licensed work from the original Codenotch project and its Windows port. Existing copyright and third-party attribution notices are preserved in `LICENSE` and provider glyph notices.
