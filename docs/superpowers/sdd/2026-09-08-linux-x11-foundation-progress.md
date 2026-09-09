# SDD ledger — plan: docs/superpowers/plans/2026-09-08-linux-x11-foundation.md

Ruling: Linux CI is moved ahead of Task 1 execution so the remote Ubuntu 22.04 runner can serve as the only available TDD executor in this environment. Cost if wrong: task ordering differs from the plan, but scope and acceptance criteria stay unchanged.

Ruling: Task 5's Linux icon asset is moved forward before Task 1 GREEN because Tauri `generate_context!` requires `icons/icon.png` during Linux `cargo check`. Cost if wrong: task ordering changes; bundle content and branding remain unchanged.

Ruling: Tasks 1-3 must reach a compile-complete Linux platform facade before the CI GREEN can occur, because `main.rs` consumes window/shell/autostart symbols during crate compilation. Their tests are still introduced before their implementations, but the first full Linux GREEN validates the combined platform foundation. Cost if wrong: the review surface is larger than the original per-task compile checkpoints, but no milestone scope is expanded.

Ruling: Task 6 is strengthened from compile/test-only CI to an actual Linux packaging gate. Ubuntu 22.04 installs the Tauri CLI and packaging prerequisites, runs `cargo tauri build`, and verifies that both a `.deb` and an `.AppImage` exist. Cost if wrong: PR CI is slower, but it directly validates the package formats already required by the M3 spec.

Task 1 RED: Ubuntu 22.04 `cargo check --workspace` failed as expected on missing Linux platform implementations; Windows `cargo check` passed. Linux system dependencies installed successfully.

Task 1: complete — Linux platform selection, locale detection/tests, and conservative focus compatibility implemented.
Task 2: complete — `xdg-open` integration and X11 Button1 query via x11rb implemented and tested.
Task 3: complete — XDG autostart renderer/path/enable/disable behavior implemented and tested.
Verification: GitHub Actions run 34292556612 passed on both Windows and Ubuntu 22.04: `cargo check --workspace` + `cargo test --workspace`.

Task 4 RED: GitHub Actions run 34293043565 failed as expected when tests referenced the not-yet-implemented cross-platform hook/config helpers.
Task 4: complete — hook executable name, main executable name, and config path are platform-aware; Windows-only process flags remain Windows-gated.
Verification: GitHub Actions run 34293537779 passed on both Windows and Ubuntu 22.04: `cargo check --workspace` + `cargo test --workspace`.

Task 5: complete — shared Tauri config now owns common app values; Windows owns `nsis` + `.ico`; Linux owns `deb` + `appimage` + `.png`.
Verification: GitHub Actions run 34293979390 passed on both Windows and Ubuntu 22.04 after the platform-specific config split.

Task 6: complete — Ubuntu 22.04 CI installs the official Tauri compile prerequisites plus packaging tools, runs workspace check/test, installs Tauri CLI, builds Linux bundles, and verifies `.deb` + `.AppImage` output. Windows CI behavior remains check/test only and unchanged.
Verification: GitHub Actions run 34294415374 passed. Windows check/test passed; Linux check/test, `cargo tauri build`, and both bundle-existence checks passed.

Cargo.lock reproducibility gate: run 34295451932 proved that `cargo check --workspace` still mutated the checked-in lockfile because the workspace rename and Linux-only x11rb graph were not persisted. A one-run, feature-branch-only CI helper was used to let Cargo generate the authoritative lockfile. Run 34297357377 produced commit `220e6b2` containing only `Cargo.lock` (55 insertions, 24 deletions), including `gethostname 1.1.0`, `x11rb 0.13.2`, `x11rb-protocol 0.13.2`, the Nyrva workspace package names, and their dependency edges. The helper intentionally exited non-zero after pushing. The temporary write permission/self-commit mechanism must be removed before the final clean-head gate.

Task 7 scope review: branch-vs-main comparison contains only CI, SDD ledger, hook infrastructure, Linux platform files, Linux-only dependency/config, Tauri bundle config, the lockfile refresh, and the Linux PNG icon. No provider endpoint/request/parsing file and no UI file changed. Wayland/layer-shell and provider enablement remain out of scope.

Manual X11 runtime smoke testing remains intentionally manual because GitHub-hosted CI is headless and does not provide the target desktop session. Final clean-head CI gate is still required after removing the temporary Cargo.lock refresh helper before the PR is declared ready to merge.
