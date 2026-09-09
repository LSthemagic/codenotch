# M8 MVP Release Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Windows + Linux X11 provider-parity codebase into a reproducible first Nyrva release candidate with permanent CI, verified Claude hook packaging on both platforms, tag-driven release artifacts, current docs, and explicit desktop smoke validation.

**Architecture:** Keep normal PR CI focused on deterministic build/test/package checks and move publication to a separate tag workflow. Use Tauri `externalBin` for `nyrva-hook` on both Windows and Linux; prepare target-triple-named sidecars reproducibly before Tauri checks/builds.

**Tech Stack:** Rust 2021, Tauri 2, GitHub Actions, NSIS, 7-Zip, Debian packages, AppImage.

**Spec:** `docs/superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md`

## Global Constraints

- MVP platforms: Windows + Linux X11.
- Providers: Claude Code, Codex, Cursor, Antigravity.
- Packaging: NSIS, `.deb`, AppImage.
- `nyrva-hook` must be physically shipped with every package that exposes Claude hook installation.
- Wayland remains deferred.
- Build unsigned artifacts unless signing is separately configured later.
- No placeholder secrets, updater infrastructure, telemetry, or UI redesign.

---

### Task 1: Make Windows sidecar packaging symmetric with Linux

**Files:**
- Create: `scripts/prepare-windows-sidecar.ps1`
- Modify: `nyrva/tauri.windows.conf.json`
- Modify: `.github/workflows/ci.yml`

**Interfaces:** `scripts/prepare-windows-sidecar.ps1 -Profile debug|release` creates `nyrva/binaries/nyrva-hook-$TARGET_TRIPLE.exe`.

- [ ] **Step 1: Add Windows sidecar preparation script**

```powershell
param(
  [ValidateSet("debug", "release")]
  [string]$Profile = "debug"
)
$ErrorActionPreference = "Stop"
$hostLine = rustc -vV | Select-String '^host:' | Select-Object -First 1
if (-not $hostLine) { throw "cannot determine Rust host triple" }
$host = ($hostLine.ToString() -replace '^host:\s*', '').Trim()
if ($host -notlike '*-pc-windows-msvc') { throw "unsupported Windows sidecar host: $host" }

if ($Profile -eq 'release') {
  cargo build -p nyrva-hook --release --locked
  $source = 'target/release/nyrva-hook.exe'
} else {
  cargo build -p nyrva-hook --locked
  $source = 'target/debug/nyrva-hook.exe'
}
$destination = "nyrva/binaries/nyrva-hook-$host.exe"
New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
Copy-Item -Force $source $destination
Write-Host "prepared $destination from $source"
```

- [ ] **Step 2: Configure Windows Tauri external binary**

`nyrva/tauri.windows.conf.json` becomes:

```json
{
  "bundle": {
    "targets": ["nsis"],
    "icon": ["icons/icon.ico"],
    "externalBin": ["binaries/nyrva-hook"]
  }
}
```

- [ ] **Step 3: Prepare debug sidecar before Windows Rust gates**

CI Windows order:

```yaml
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@stable
- name: Prepare Windows sidecar
  shell: pwsh
  run: ./scripts/prepare-windows-sidecar.ps1 -Profile debug
- run: cargo check --workspace
- run: cargo test --workspace
```

- [ ] **Step 4: Prepare release sidecar before Tauri build**

```yaml
- run: cargo install tauri-cli --version "^2.0.0" --locked
- name: Prepare release sidecar
  shell: pwsh
  run: ./scripts/prepare-windows-sidecar.ps1 -Profile release
- name: Build Windows bundle
  working-directory: nyrva
  run: cargo tauri build
```

- [ ] **Step 5: Commit**

Commit: `build(windows): package Nyrva Claude hook sidecar`

---

### Task 2: Replace temporary CI diagnostics with permanent package verifiers

**Files:**
- Create: `scripts/verify-linux-bundles.sh`
- Create: `scripts/verify-windows-bundle.ps1`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Linux verifier fails unless `.deb`, AppImage, and `usr/bin/nyrva-hook` exist in both packages.
- Windows verifier fails unless NSIS exists and 7-Zip can see `nyrva-hook.exe` inside it.

- [ ] **Step 1: Add Linux verifier**

```bash
#!/usr/bin/env bash
set -euo pipefail
root="${1:-target/release/bundle}"
deb="$(find "$root/deb" -maxdepth 1 -type f -name '*.deb' -print -quit)"
appimage="$(find "$root/appimage" -maxdepth 1 -type f -name '*.AppImage' -print -quit)"
test -n "$deb" && test -n "$appimage"
manifest="$(mktemp)"
dpkg-deb -c "$deb" > "$manifest"
grep -Eq '[[:space:]]usr/bin/nyrva-hook$' "$manifest"
appimage="$(realpath "$appimage")"
chmod +x "$appimage"
extract="$(mktemp -d)"
(cd "$extract" && "$appimage" --appimage-extract >/dev/null)
test -x "$extract/squashfs-root/usr/bin/nyrva-hook"
```

- [ ] **Step 2: Add Windows verifier**

```powershell
param([string]$TargetDir = "target/release/bundle/nsis")
$ErrorActionPreference = "Stop"
$installer = Get-ChildItem -Path $TargetDir -Filter *.exe -File | Select-Object -First 1
if (-not $installer) { throw "NSIS installer not found in $TargetDir" }
$listing = & 7z l $installer.FullName | Out-String
if ($LASTEXITCODE -ne 0) { throw "7z could not inspect $($installer.FullName)" }
if ($listing -notmatch 'nyrva-hook\.exe') { throw "nyrva-hook.exe not found inside NSIS installer" }
Write-Host "verified $($installer.FullName) with nyrva-hook.exe"
```

- [ ] **Step 3: Simplify Linux CI**

Remove normal-success-path diagnostic artifact generation and retain:

```yaml
- name: Verify Linux bundles and hook sidecar
  run: bash scripts/verify-linux-bundles.sh target/release/bundle
```

- [ ] **Step 4: Add Windows verification**

```yaml
- name: Verify Windows NSIS bundle
  shell: pwsh
  run: ./scripts/verify-windows-bundle.ps1 -TargetDir target/release/bundle/nsis
```

- [ ] **Step 5: Validate and commit**

Run: `bash -n scripts/verify-linux-bundles.sh`

Commit: `ci: make cross-platform package verification permanent`

---

### Task 3: Add tag-driven release workflow

**Files:** Create `.github/workflows/release.yml`

**Interfaces:** Push of `v*` tag builds tagged commit only and publishes NSIS, `.deb`, AppImage to matching GitHub Release.

- [ ] **Step 1: Add trigger and permission**

```yaml
name: Release
on:
  push:
    tags: ['v*']
permissions:
  contents: write
```

- [ ] **Step 2: Add Windows build job**

Use checkout + stable Rust, `prepare-windows-sidecar.ps1 -Profile debug`, workspace tests, install Tauri CLI, `prepare-windows-sidecar.ps1 -Profile release`, `cargo tauri build`, Windows verifier, then upload `target/release/bundle/nsis/*.exe` as `nyrva-windows`.

- [ ] **Step 3: Add Linux build job**

Install the same permanent Ubuntu 22.04 dependencies as CI (including M7 dbus build packages), run `prepare-linux-sidecar.sh release`, workspace tests, install Tauri CLI, build, Linux verifier, then upload `.deb` and AppImage as `nyrva-linux`.

- [ ] **Step 4: Add publish job**

```yaml
publish:
  needs: [windows, linux]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/download-artifact@v4
      with:
        path: dist
        merge-multiple: true
    - uses: softprops/action-gh-release@v2
      with:
        files: dist/*
        generate_release_notes: true
```

No signing placeholders.

- [ ] **Step 5: Commit**

Commit: `ci: add tag-driven Nyrva release workflow`

---

### Task 4: Update documentation and manual smoke gates

**Files:** Modify `README.md`; create `docs/DEVELOPMENT.md`, `docs/SMOKE_TESTS.md`

- [ ] **Step 1: Update README state**

Must say Windows supported, Linux X11 supported, Wayland unsupported/deferred; list Claude Code, Codex, Cursor, Antigravity; explain provider-owned auth/state is borrowed read-only.

- [ ] **Step 2: Add developer build/package instructions**

Linux:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/prepare-linux-sidecar.sh release
cd nyrva
cargo tauri build
```

Windows:

```powershell
./scripts/prepare-windows-sidecar.ps1 -Profile release
cargo check --workspace
cargo test --workspace
cd nyrva
cargo tauri build
```

Document CI-equivalent system prerequisites.

- [ ] **Step 3: Add provider constraints**

Document Claude persistent hook behavior, Codex `CODEX_HOME`, Cursor read-only `state.vscdb`, Antigravity bridge/keyring/transcript fallback, and that Nyrva never refreshes provider credentials.

- [ ] **Step 4: Add checked-in smoke checklist**

Linux X11 checklist includes `.deb`, AppImage, edge placement, drag persistence, tray, autostart, Claude hook install/uninstall/relaunch, Codex usage/activity, Cursor usage/activity, Antigravity bridge/fallback, provider links, clean shutdown/relaunch.

Windows checklist includes NSIS install/uninstall, verification that Claude hook install succeeds from installed files, provider behaviors, tray/autostart, and relaunch.

- [ ] **Step 5: Remove stale future-Linux wording**

Run:

```bash
git grep -n 'Linux support is planned\|Linux support is planned next\|Windows is the current working baseline' -- README.md docs || true
```

Expected: no stale product-state statements.

- [ ] **Step 6: Commit**

Commit: `docs: document Windows and Linux X11 MVP`

---

### Task 5: Final release-candidate verification and PR gate

- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `git diff --check`
- [ ] Require Windows CI: sidecar prep, check, tests, NSIS build, hook-inside-installer verification.
- [ ] Require Linux CI: dependencies, sidecar prep, check, clean lock, tests, `.deb` + AppImage build, hook-inside-both verification.
- [ ] Review `release.yml` uses the same successful build commands and tagged commit; do not create a fake public tag just for testing.
- [ ] Open PR titled `M8: harden Nyrva MVP release pipeline` and stop before merge.
- [ ] After merge, first public MVP release remains blocked until `docs/SMOKE_TESTS.md` is manually executed on at least one supported Windows environment and one real Linux X11 environment.
