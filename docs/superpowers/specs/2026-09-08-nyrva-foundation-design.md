# Nyrva Foundation Design

## Goal

Turn this fork into an independent Windows + Linux project named **Nyrva**, using the existing Rust/Tauri Windows port as the canonical application codebase and removing the macOS/Swift application from the repository.

## Product scope

- Product name: **Nyrva**
- Platforms for MVP: **Windows + Linux X11**
- Wayland: deferred to a later milestone
- Providers in MVP: **Claude, Codex, Cursor, Antigravity**
- UI: preserve the current Windows-port UI for the first release; only rebrand and make platform-required adjustments
- Upstream strategy: independent development; no requirement to stay merge-compatible with `vinzdg/codenotch`

## Milestone 1 scope

Milestone 1 is structural only. It must not add Linux support or change provider behavior.

### Repository transformation

Use the Rust/Tauri application currently under `windows/` as the project root:

- move the Rust workspace to repository root;
- rename crate `codenotch` to `nyrva`;
- rename crate `codenotch-hook` to `nyrva-hook`;
- remove macOS-only Swift/Xcode application files;
- remove macOS-only CI and release configuration;
- rewrite root documentation for Nyrva;
- retain legal attribution required by the MIT-licensed source and Windows port.

Target shape:

```text
repo/
├── Cargo.toml
├── Cargo.lock
├── nyrva/
│   ├── Cargo.toml
│   ├── src/
│   ├── ui/
│   └── tauri.conf.json
├── nyrva-hook/
│   ├── Cargo.toml
│   └── src/
├── docs/
├── .github/
├── README.md
└── LICENSE
```

## Branding rules

Change application-owned identifiers from Codenotch to Nyrva where doing so does not break provider protocols or external compatibility:

- package/workspace member names;
- executable names;
- app/window/tray labels;
- installer/product names;
- application data/config directory names;
- documentation and user-facing text.

Do **not** rename provider-owned files, endpoints, environment variables, protocol fields, or third-party paths just because they contain provider names.

## Preservation rules

Milestone 1 must preserve current Windows behavior:

- the edge-pinned window still launches;
- tray behavior still works;
- Claude, Codex, Cursor, and Antigravity integrations retain their existing logic;
- hooks retain their existing behavior after the executable/crate rename;
- no UI redesign;
- no Linux implementation yet.

## Licensing

Keep the MIT license and preserve copyright/attribution notices from reused upstream code. Nyrva branding does not erase prior authorship.

## Validation

The milestone is complete only when the root Rust workspace can be built/tested for Windows and the project no longer depends on the removed macOS application structure.

Required automated validation:

```text
cargo check --workspace
cargo test --workspace
```

Windows CI should run those commands from repository root.

## Deferred work

Explicitly out of scope for Milestone 1:

- Linux/X11 implementation;
- Wayland support;
- provider behavior changes;
- new providers;
- UI redesign;
- architecture extraction into `platform/windows` / `platform/linux` (Milestone 2).
