# Sindri: Hewn contributor notes

Read `AGENTS.md` first. Hewn is an offline-first game asset preparation studio.

## Initial architecture
- `crates/hewn-core`: portable Rust image processing, deterministic texture atlas packing, metadata.
- `apps/web`: React + TypeScript asset workbench. Atlas packing is routed through Rust via the WebAssembly bridge in PR #2; validate the build before treating that integration as complete.
- `crates/hewn-wasm`: WebAssembly adapter for the Rust core.
- Future: Tauri desktop host.
- Read `ROADMAP.md` for the full product vision, constraints, and milestones.

## Quality expectations
- Keep original images unchanged; record trim and offsets.
- Report images that cannot fit; never silently omit sprites.
- Do not claim Rust and browser outputs are identical until integrated and tested.
- Prefer accessible UI, explicit export settings, and reproducible output.
