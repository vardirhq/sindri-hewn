# Sindri: Hewn

**Raw assets, refined.**

Hewn is an offline-first, non-destructive game asset preparation studio. Organize loose PNG sprites, trim transparent edges, and build texture atlases with source offsets preserved.

## Current state

The initial foundation includes:
- **Browser workbench** built with React, TypeScript, Vite, and Rust WebAssembly. Import and inspect PNGs, preview an atlas, adjust trimming, atlas dimensions and padding, and export a PNG plus JSON metadata.
- **Shared Rust core library** with deterministic shelf packing, alpha trimming, offsets, overflow reporting, and unit tests.
- Sample sprites in the web UI for exploring the workflow without supplying files.
- GitHub Actions builds for Rust and web.

**Important:** The browser now packs through the Rust core via WebAssembly. No Tauri wrapper exists yet.

## Run locally

### Web workbench

```bash
cd apps/web
npm install
cargo install wasm-pack --locked
rustup target add wasm32-unknown-unknown
npm run dev
```

Open the local Vite URL. Files are processed locally in your browser, never uploaded to a server.

### Rust

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Architecture

```text
crates/hewn-core/  Portable image + atlas processing
crates/hewn-wasm/  WebAssembly adapter for hewn-core
apps/web/          Browser UI and initial local atlas workflow
```

## Next milestones

1. Add Rust/WASM parity fixtures and improve browser error reporting.
2. Multiple pages / smarter rectangle packing with collision-free atlas placement.
3. Project saving, semantic grouping, duplicate detection, and animation sequences.
4. Terrain composer, bitmask rules, and map preview.
5. Desktop packaging with Tauri and additional engine export formats.

Hewn is part of the Sindri ecosystem but is designed for use with **any game engine**, including Godot, Unity, and Sindri itself.
