# Sindri: Hewn contributor notes

Read `AGENTS.md` first. Hewn is an offline-first game asset preparation studio.

## Initial architecture
- `crates/hewn-core`: portable Rust image processing, deterministic texture atlas packing, metadata.
- `apps/web`: React + TypeScript asset workbench. Its initial atlas preview/export is client-only, while the Rust crate provides the reusable native foundation.
- Future: Tauri desktop host and WASM bindings for one shared packing implementation.

## Quality expectations
- Keep original images unchanged; record trim and offsets.
- Report images that cannot fit; never silently omit sprites.
- Do not claim Rust and browser outputs are identical until integrated and tested.
- Prefer accessible UI, explicit export settings, and reproducible output.
