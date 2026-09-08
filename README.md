# Nyrva

Nyrva is a lightweight desktop usage monitor for AI coding assistants. It pins provider usage and session activity to the edge of your screen so limits stay visible without interrupting your workflow.

## Providers

The first Nyrva release keeps the four providers from the existing Windows port:

- Claude Code
- Codex
- Cursor
- Antigravity

## Platforms

### Windows

Windows is the current working baseline and the focus of the foundation milestone.

### Linux

Linux support is planned next, starting with X11. Wayland support is intentionally deferred because edge-pinned global window positioning requires compositor-specific work.

## Architecture

Nyrva is built with Rust and Tauri 2. The UI remains the compact edge-notch interface from the existing Windows port for the first release.

```text
Cargo.toml
├── nyrva/       # Tauri desktop application
└── nyrva-hook/  # Claude Code hook launcher
```

Provider logic lives in the desktop crate and currently supports Claude, Codex, Cursor, and Antigravity.

## Development

Requirements for the current Windows build include Rust and the normal Tauri 2 Windows prerequisites/WebView2 environment.

From the repository root:

```powershell
cargo check --workspace
cargo test --workspace
cargo run -p nyrva
```

Release build:

```powershell
cargo build --workspace --release
```

## Roadmap

1. Nyrva foundation and Windows baseline
2. Platform abstraction for OS-specific behavior
3. Linux X11 support
4. Linux provider/activity parity
5. Wayland support

## License and attribution

Nyrva is MIT licensed and is based on MIT-licensed work from the original Codenotch project and its Windows port. Existing copyright and third-party attribution notices are preserved in `LICENSE` and provider glyph notices.
