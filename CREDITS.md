# Credits and provenance

Nyrva is a derivative work, not a from-scratch implementation of the original idea.

## Codenotch

The original project is [Codenotch](https://github.com/vinzdg/codenotch), created by **Vinz (@vinzdg)** and its contributors. It supplied the original application concept, upstream code and application artwork. Its MIT copyright and permission notice are preserved in `LICENSE`.

## Existing Windows port

Nyrva also builds on the existing **Rust/Tauri Windows port**, credited in the inherited license to **Im-Midi (NG) and contributors**. The Windows Rust/Tauri foundation and compact interface were already present before the Nyrva migration; they are not claimed as newly authored by this fork.

## Nyrva fork

The Nyrva fork is maintained by **LSthemagic (Railan Santana)**. Its work includes promoting the Rust workspace to the project root, Nyrva branding, platform separation, Linux X11 adaptations, provider parity work, cross-platform packaging, CI and release delivery.

This fork does not imply endorsement by the upstream authors or the providers it monitors.

## Third-party artwork and marks

Provider glyphs originate from `@lobehub/icons-static-svg` by LobeHub. Their original notice is retained at `nyrva/glyphs/NOTICE.md` in the source tree and shipped as `PROVIDER_GLYPH_NOTICES.md` in the application resources. Provider names and marks belong to their respective owners.

The application icon and tray artwork derived from Codenotch remain attributed to @vinzdg in `LICENSE`.

## Distributed notices

Windows NSIS, Linux `.deb` and AppImage packages include `LICENSE`, `CREDITS.md` and `PROVIDER_GLYPH_NOTICES.md` in their resource directories. The release workflow also attaches readable copies alongside the packages. Preserve these notices when redistributing the software.
