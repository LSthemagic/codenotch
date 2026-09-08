# SDD ledger — plan: docs/superpowers/plans/2026-09-08-linux-x11-foundation.md

Ruling: Linux CI is moved ahead of Task 1 execution so the remote Ubuntu 22.04 runner can serve as the only available TDD executor in this environment. Cost if wrong: task ordering differs from the plan, but scope and acceptance criteria stay unchanged.

Ruling: Task 5's Linux icon asset is moved forward before Task 1 GREEN because Tauri `generate_context!` requires `icons/icon.png` during Linux `cargo check`. Cost if wrong: task ordering changes; bundle content and branding remain unchanged.

Ruling: Tasks 1-3 must reach a compile-complete Linux platform facade before the CI GREEN can occur, because `main.rs` consumes window/shell/autostart symbols during crate compilation. Their tests are still introduced before their implementations, but the first full Linux GREEN validates the combined platform foundation. Cost if wrong: the review surface is larger than the original per-task compile checkpoints, but no milestone scope is expanded.

Task 1 RED: Ubuntu 22.04 `cargo check --workspace` failed as expected on missing Linux platform implementations; Windows `cargo check` passed. Linux system dependencies installed successfully.

Task 1: complete — Linux platform selection, locale detection/tests, and conservative focus compatibility implemented.
Task 2: complete — `xdg-open` integration and X11 Button1 query via x11rb implemented and tested.
Task 3: complete — XDG autostart renderer/path/enable/disable behavior implemented and tested.
Verification: GitHub Actions run 34292556612 passed on both Windows and Ubuntu 22.04: `cargo check --workspace` + `cargo test --workspace`.

Task 6 (partial): Linux CI job added early; final implementation gate still pending.
