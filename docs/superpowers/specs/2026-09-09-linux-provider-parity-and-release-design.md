# Nyrva M5–M8 Linux Provider Parity and MVP Release Design

## Goal

Complete the remaining Windows + Linux X11 MVP work by bringing Codex, Cursor, and Antigravity to Linux parity, then hardening the product for a first distributable release.

This design continues the existing Nyrva architecture. It does not redesign the UI, add providers, or add Wayland support.

## Product scope

- Product: **Nyrva**
- MVP platforms: **Windows + Linux X11**
- MVP providers: **Claude Code, Codex, Cursor, Antigravity**
- UI: preserve the current edge-notch interface
- Packaging:
  - Windows: NSIS
  - Linux: `.deb` + AppImage
- Wayland: deferred until after the MVP

## Delivery strategy

M5 through M8 are delivered as four sequential pull requests. Each milestone starts from the previous milestone's green `main` and must pass its own Windows and Linux CI gates before the next milestone begins.

This keeps provider-specific regressions isolated and avoids one large cross-provider diff.

---

# M5 — Codex Linux parity

## Objective

Make Codex usage discovery and activity detection behave consistently on Linux X11 while preserving current Windows behavior.

## Current reusable behavior

The existing adapter already uses Codex-owned state under `~/.codex` for:

- `auth.json` credentials;
- rollout session logs;
- persisted Nyrva usage state;
- `thread_history_1.sqlite` and `state_5.sqlite` activity data.

Those data formats remain provider-owned and must not be renamed.

## Required changes

### Codex home resolution

Introduce a single Codex home resolver with this precedence:

1. non-empty `CODEX_HOME` environment variable;
2. `$HOME/.codex` fallback.

Every Codex path must derive from this resolver, including:

- `auth.json`;
- `sessions/`;
- `thread_history_1.sqlite`;
- `state_5.sqlite`;
- optional `bin/` lookup.

### Executable discovery

Make executable discovery platform-aware without spawning Codex during normal polling.

Windows keeps its existing candidates. Linux candidates include, in practical order:

- `$CODEX_HOME/bin/codex` or `~/.codex/bin/codex`;
- `codex` found on `PATH`;
- native/package locations already discoverable without shelling out to Codex itself.

Executable discovery is diagnostic/support behavior only. Usage polling continues to borrow existing credentials and state rather than launching a Codex process.

### HTTP identity

Remove the hard-coded Windows identity from Codex HTTP requests. Nyrva may identify itself by product/version and current platform, but must not claim to be Windows when running on Linux.

### Activity parity

The activity probe must use the same resolved Codex home as the usage adapter. Existing SQLite-first behavior remains preferred, with rollout-tail fallback when the desktop state database is absent.

### Focus behavior

Codex activity entries on Linux reuse the X11 process/window primitives introduced in M3/M4. No Wayland fallback is added.

## Tests

Add deterministic tests for:

- `CODEX_HOME` precedence;
- default `~/.codex` fallback;
- Linux executable candidate generation;
- auth/session/activity paths sharing one resolver;
- platform-correct User-Agent rendering;
- existing rollout parsing and activity behavior remaining unchanged.

## Acceptance

M5 is complete when Windows behavior remains green and Linux can discover Codex state, usage, and activity through Linux-native paths without Windows-only assumptions.

---

# M6 — Cursor Linux parity

## Objective

Make Cursor usage and activity detection fully supported on Linux while preserving the current read-only credential/session borrowing model.

## Current reusable behavior

The adapter already reads Cursor's VS Code-style global state database and borrows:

- `cursorAuth/accessToken`;
- `cursorAuth/stripeMembershipAuthId`;
- non-secret plan metadata;
- `composerHeaders` activity state.

The database remains read-only.

## Required changes

### State database resolution

Use an explicit platform resolver:

- Windows: `%APPDATA%/Cursor/User/globalStorage/state.vscdb`;
- Linux: `${XDG_CONFIG_HOME:-~/.config}/Cursor/User/globalStorage/state.vscdb`.

Do not rely on a path transformation that assumes Windows drive syntax on Linux.

### SQLite URI handling

Create a tested file-URI helper for the immutable fallback. It must correctly escape paths on both Windows and Linux.

Opening order remains:

1. plain read-only connection so a live WAL is visible;
2. immutable read-only URI fallback for closed/checkpointed databases.

Nyrva must never write to Cursor's database or copy secrets into its own logs.

### Activity parity

Cursor usage and activity must share the same state-database resolver. Existing `composerHeaders` semantics remain unchanged.

### Diagnostics

`present()` and doctor output must report Linux paths accurately and avoid Windows-specific wording such as `%APPDATA%` when running on Linux.

## Tests

Add deterministic tests for:

- Windows and Linux state path rendering;
- `XDG_CONFIG_HOME` precedence;
- correct immutable file URI generation on Unix and Windows path shapes;
- read-only/WAL-first behavior through small test databases;
- credential redaction/no secret leakage in diagnostic strings;
- activity resolver using the same database path as usage.

## Acceptance

M6 is complete when Cursor usage and activity work from the Linux state database with no Windows-only path or URI assumptions and all provider state remains read-only.

---

# M7 — Antigravity Linux parity

## Objective

Complete Linux support for Antigravity while preserving the existing source-priority model and without inventing an unsupported public quota API.

## Source priority

The Linux adapter uses the same truth order as Windows where possible:

1. running local Antigravity language-server bridge;
2. last valid bridge reading, marked stale if the IDE closes;
3. local credential/keyring path when available;
4. transcript-count fallback under `~/.gemini/antigravity`.

## Required changes

### Language-server discovery

Keep the existing Unix process-table discovery for the language server and CSRF token.

Port discovery must not require `lsof` as the only implementation. Add a Linux-native resolver based on `/proc` data, including `/proc/<pid>/fd` and `/proc/net/tcp*` socket information, with `lsof` retained only as a compatibility fallback when present.

All local bridge calls remain loopback-only and may relax TLS verification only for `127.0.0.1`/localhost connections to the locally running Antigravity server.

### Linux credential storage

Implement Linux Secret Service lookup for the same logical `gemini:antigravity` credential identity used by the existing keyring model.

The adapter must:

- read only;
- never persist the provider token in Nyrva state;
- never log token contents;
- gracefully continue to transcript fallback when Secret Service is unavailable, locked, or has no matching item.

The implementation should minimize new runtime dependencies. If a small Rust Secret Service/keyring dependency is required, it must be isolated behind Linux `cfg` and tested independently from live desktop services.

### Transcript fallback

Keep the existing transcript fallback and ensure Linux path resolution uses `$HOME/.gemini/antigravity` consistently.

### Activity parity

Activity probing keeps the existing recent-transcript heuristic on Linux. Provider presence must be true when either Antigravity state exists or a valid local/keyring source is discoverable.

## Tests

Add deterministic tests for:

- language-server command-line flag parsing;
- `/proc` socket inode/port mapping with fixtures or pure parsers;
- duplicate port elimination;
- Secret Service result decoding separated from live service access;
- graceful no-keyring fallback;
- transcript fallback path and parsing;
- no token content in logs/diagnostics.

## Acceptance

M7 is complete when Linux can obtain Antigravity quota from the local bridge when running, fall back safely when it is not, and no longer has a hard-coded `read_credential_raw() -> None` Linux path.

---

# M8 — MVP release hardening

## Objective

Turn the now-parity codebase into a reproducible first Windows + Linux X11 release candidate.

## CI cleanup

The normal CI workflow must contain only permanent checks. Temporary diagnostics introduced during M3/M4 packaging work must be removed unless they provide lasting failure evidence at low maintenance cost.

Required CI gates:

### Windows

- `cargo check --workspace`;
- `cargo test --workspace`;
- release package build using the Windows Tauri config;
- verify expected installer output exists.

### Linux

- install documented Tauri system prerequisites;
- prepare the `nyrva-hook` sidecar;
- `cargo check --workspace`;
- verify `Cargo.lock` remains clean;
- `cargo test --workspace`;
- `cargo tauri build`;
- verify `.deb` exists;
- verify AppImage exists;
- verify `nyrva-hook` is physically present in both Linux packages.

## Release workflow

Add a tag-driven GitHub Actions release workflow for version tags matching the chosen release convention, initially `v*`.

The release workflow must:

- build from the tagged commit only;
- produce Windows NSIS, Linux `.deb`, and Linux AppImage artifacts;
- keep the Linux sidecar preparation reproducible;
- fail if expected package outputs are missing;
- publish artifacts to the GitHub Release for that tag.

Signing/notarization is not invented in this milestone. If signing credentials are not configured, the workflow must build unsigned artifacts rather than contain placeholder secrets.

## Documentation

Update README and developer instructions so they describe the actual product state:

- Windows supported;
- Linux X11 supported;
- Claude Code, Codex, Cursor, and Antigravity supported on the MVP platforms according to their documented local-state constraints;
- Wayland explicitly unsupported/deferred;
- development prerequisites for both Windows and Ubuntu/Debian Linux;
- packaging commands;
- hook installation behavior;
- known limitations.

## Runtime smoke checklist

Maintain a checked-in manual smoke checklist for a real Linux X11 desktop because headless GitHub runners cannot validate desktop interaction.

The checklist must cover at least:

- application launch;
- edge placement;
- drag persistence;
- tray interaction;
- autostart enable/disable;
- Claude hook install/uninstall and relaunch;
- Codex usage/activity discovery;
- Cursor usage/activity discovery;
- Antigravity bridge/fallback discovery;
- provider click-through links;
- clean shutdown/relaunch.

A corresponding Windows smoke checklist must cover the same product-level behaviors that are applicable there.

## Release acceptance

M8 is complete when:

- permanent CI is green on Windows and Linux;
- release packaging is reproducible from a tag;
- expected distributable artifacts are produced;
- the checked-in smoke checklist is executed manually on at least one supported Windows environment and one supported Linux X11 environment before declaring the first public MVP release;
- README no longer describes Linux as future work.

---

# Cross-milestone constraints

## Preserve Windows

Every provider milestone must keep current Windows behavior. Platform-specific changes should be isolated with explicit helpers or `cfg` boundaries rather than branching throughout parsing/business logic.

## Read-only provider integrations

Nyrva borrows provider-owned credentials/state. It does not become an authentication manager for Codex, Cursor, or Antigravity.

Provider credentials must never be:

- logged;
- emitted to the UI;
- copied into Nyrva persisted snapshots;
- refreshed or modified by Nyrva.

## Error handling

Provider failures are isolated. A missing provider, locked keyring, stale database, malformed local state, or unavailable network endpoint must not crash Nyrva or block other providers.

Where a previous valid reading exists, stale is preferred over invented data.

## Testing strategy

Each milestone uses TDD for new path/platform behavior.

Pure path, parser, URI, `/proc`, and configuration behavior must be unit tested without requiring a live provider installation.

Live provider services are validated by manual smoke checks and must not be required for ordinary CI.

## Explicitly deferred

The following remain outside M5–M8:

- Wayland/layer-shell support;
- macOS restoration;
- UI redesign;
- new providers;
- Nyrva-managed provider login flows;
- cloud synchronization;
- telemetry/analytics;
- automatic application update infrastructure unless separately designed later.

# Completion definition

After M8, Nyrva's first MVP consists of one Rust/Tauri codebase supporting Windows and Linux X11, with Claude Code, Codex, Cursor, and Antigravity provider integrations, reproducible installers/packages, permanent CI gates, and documented manual desktop smoke validation.
