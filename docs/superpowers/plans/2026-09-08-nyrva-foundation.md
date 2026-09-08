# Nyrva Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the fork into the Windows-first Nyrva Rust/Tauri workspace at repository root while removing the macOS/Swift application and preserving current Windows behavior.

**Architecture:** Reuse the existing `windows/` Rust workspace as the canonical source. Move its two crates to root as `nyrva/` and `nyrva-hook/`, preserve provider implementation blobs unchanged where possible, and edit only workspace metadata, branding/runtime identifiers, documentation, licensing, and CI required by the rename.

**Tech Stack:** Rust 2021, Tauri 2, WebView2, GitHub Actions

**Spec:** `docs/superpowers/specs/2026-09-08-nyrva-foundation-design.md`

## Global Constraints

- Product name is `Nyrva`.
- MVP platforms are Windows + Linux X11; Milestone 1 implements Windows only.
- Keep Claude, Codex, Cursor, and Antigravity provider behavior unchanged.
- Preserve the current UI; no redesign.
- Wayland is out of scope.
- Preserve MIT attribution from the upstream macOS project and Windows port.

---

### Task 1: Promote the Rust workspace to repository root

**Files:**
- Create/replace: `Cargo.toml`, `Cargo.lock`, `.gitattributes`, `.gitignore`
- Move: `windows/codenotch/` -> `nyrva/`
- Move: `windows/codenotch-hook/` -> `nyrva-hook/`
- Remove: `windows/`

**Interfaces:**
- Produces root Cargo workspace members `nyrva` and `nyrva-hook`.

- [ ] Move the existing Rust/Tauri trees without changing provider source blobs.
- [ ] Change workspace members to `["nyrva", "nyrva-hook"]`.
- [ ] Change package names and repository metadata to Nyrva.
- [ ] Ensure Cargo metadata resolves from repository root.

### Task 2: Remove macOS application infrastructure

**Files:**
- Remove: `Sources/`, `Tests/`, `project.yml`, macOS scripts/release artifacts and macOS-only task/site material not used by Nyrva.
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Produces a repository whose build entrypoint is Cargo rather than Xcode.

- [ ] Remove Swift/Xcode application source and tests.
- [ ] Replace macOS Xcode CI with Windows Rust CI.
- [ ] CI runs `cargo check --workspace` and `cargo test --workspace` from root.

### Task 3: Rename runtime identity to Nyrva

**Files:**
- Modify: `nyrva/tauri.conf.json`
- Modify: `nyrva/src/config.rs`
- Modify additional app/hook files that contain application-owned `Codenotch`/`codenotch` identifiers.

**Interfaces:**
- Produces application product name `Nyrva`, binary/crate names `nyrva` and `nyrva-hook`, and Nyrva-owned data/config paths.

- [ ] Rename Tauri `productName` and application identifier.
- [ ] Rename app-owned config/data locations to `nyrva`.
- [ ] Rename hook executable references and user-facing app/tray text while leaving provider-owned names/protocols untouched.
- [ ] Preserve all provider request/credential logic.

### Task 4: Rebuild project documentation and legal notices

**Files:**
- Modify: `README.md`, `LICENSE`
- Keep: `docs/superpowers/specs/2026-09-08-nyrva-foundation-design.md`
- Keep: `docs/superpowers/plans/2026-09-08-nyrva-foundation.md`

**Interfaces:**
- Documents Nyrva as an independent Windows + Linux project with Linux X11 planned after Windows foundation.

- [ ] Rewrite README around Nyrva, the four providers, current Windows status, and roadmap to Linux X11/Wayland.
- [ ] Preserve prior MIT attribution and third-party provider-mark notices.

### Task 5: Verification

**Files:** No production changes unless verification identifies a migration defect.

- [ ] Run/require `cargo check --workspace` on Windows CI.
- [ ] Run/require `cargo test --workspace` on Windows CI.
- [ ] Confirm no Swift/Xcode build remains in the root project.
- [ ] Confirm provider source logic is unchanged except application-owned rename references.
- [ ] Open a PR from `feat/nyrva-foundation` to `main` for review.
