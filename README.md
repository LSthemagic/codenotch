# Nyrva

Nyrva is a lightweight desktop usage monitor for AI coding assistants. It pins provider usage and session activity to the edge of your screen so limits stay visible without interrupting your workflow.

## MVP scope and release status

The implemented MVP targets **Windows and Linux X11**, with **Claude Code, Codex, Cursor and Antigravity**. Windows packages use NSIS (`.exe`); Linux packages use `.deb` and AppImage. Each package includes the `nyrva-hook` helper for Claude Code.

**Implementation and automated packaging are not desktop acceptance.** The first public MVP release remains blocked until the [manual smoke checklist](docs/SMOKE_TESTS.md) is executed and recorded on Windows and a real Linux X11 session. No manual pass is implied by this README or by a green CI run.

Ubuntu 22.04 is the Linux CI build baseline; Ubuntu 22.04 and Debian 12 are the intended Linux MVP baselines. Record the exact environments actually tested rather than assuming all distributions work.

Wayland, macOS, new providers, UI redesign and automatic application updates are outside this MVP.

## Installable builds

Successful CI runs upload:

- `nyrva-windows`: Windows NSIS installer.
- `nyrva-linux`: Debian package and AppImage.

For development testing, download the artifacts from the successful Actions run for the commit being tested. These are CI builds, not a declaration of a stable release.

Version tags matching `v*` run the [release workflow](.github/workflows/release.yml). It validates the tag against Cargo and Tauri, reuses the same CI build/test/package gates, requires all three package formats and creates a **draft GitHub Release** with `SHA256SUMS`. A maintainer publishes it only after manual acceptance; pushing a tag does not publish a stable release automatically.

See [release instructions](docs/RELEASING.md) for versioning, acceptance, publishing and failed-run recovery. Packages are unsigned; checksums verify integrity, not publisher identity.

## Provider requirements

Sign in using each provider's own application first. Nyrva is not an authentication manager and must not refresh or modify provider credentials. Provider-owned credentials/databases are read-only inputs; explicitly installing or removing the Claude integration changes its hook configuration.

Codex uses `CODEX_HOME` or `~/.codex`; Cursor uses its local state database; Antigravity prefers the running local language-server bridge and has credential/transcript fallbacks. Provider presence, local state, service availability and account capabilities affect what can be displayed. Missing data must not be interpreted as zero usage.

See [development and provider notes](docs/DEVELOPMENT.md) for prerequisites and limitations.

## Architecture

Nyrva uses Rust and Tauri 2. The compact edge-notch UI is retained from the existing Windows port.

```text
Cargo.toml
├── nyrva/       # Tauri desktop application
└── nyrva-hook/  # Claude Code hook launcher
```

OS-specific behavior is isolated behind the platform layer. Provider adapters live in the desktop crate.

## Development

Install the platform prerequisites in [DEVELOPMENT.md](docs/DEVELOPMENT.md). Prepare the sidecar **before** the first workspace check/test.

Windows, from the repository root:

```powershell
./scripts/prepare-windows-sidecar.ps1 debug
cargo check --workspace
cargo test --workspace
```

Linux, from the repository root:

```bash
bash scripts/prepare-linux-sidecar.sh debug
cargo check --workspace
cargo test --workspace
```

Release tooling tests use Python 3.11+ and only its standard library:

```bash
python -m unittest discover -s tests -p 'test_release_tools.py' -v
```

Build/package commands, manual test records and publication steps are documented separately so passing a build cannot be mistaken for release acceptance. Documentation in `docs/DEVELOPMENT.md`, `docs/SMOKE_TESTS.md` and `docs/RELEASING.md` is in Portuguese.

## License and attribution

Nyrva is MIT licensed and is based on MIT-licensed work from the original Codenotch project and its Windows port. Existing copyright and third-party attribution notices are preserved in `LICENSE` and provider glyph notices.
