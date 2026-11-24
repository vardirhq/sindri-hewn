# Sindri: Hewn | Product vision and development plan

> **Raw assets, refined.**
>
> Status: living product plan. This is the intended direction, not a claim that every
> feature exists. Update this document as decisions change and milestones land.

## The idea

**Sindri: Hewn** (pronounced "hyoon") is a standalone, offline-first asset
preparation studio for indie game developers. It turns scattered source art into
organized, reproducible, engine-ready resources without making artists learn a
full game engine, build custom scripts, or subscribe to an AI service.

The central promise is simple: **import raw art, prepare it visually, export
clean assets and metadata**. The application should feel like a carefully
designed creative tool, not a collection of developer utilities taped together.

Hewn belongs to the Sindri ecosystem but is **not Sindri-only**. It should be
useful to people shipping with Godot, Unity, custom engines, or Sindri.

## Why build it?

Small game teams routinely spend time on tedious asset preparation:
- Removing transparent margins from sprites while preserving their placement.
- Combining hundreds of images into efficient texture atlases.
- Keeping sprite names, animations, and variants consistent.
- Assembling terrain tiles and checking that transitions work.
- Re-exporting assets after art changes without breaking references.
- Translating metadata into the format a target engine expects.

Existing options often solve only one step or bury it in a larger editor.
Hewn should connect the steps into a visual, predictable workflow.

## Product principles

1. **Non-destructive by default.** Original imports remain untouched. Edits are
   instructions in a project; exports are derived artifacts.
2. **Offline-first and private.** Images stay on the user's device. No account,
   server upload, AI model, or network connection is required to do the work.
3. **Deterministic and reproducible.** Same source bytes, ordering, settings,
   and tool version should produce the same placement and metadata.
4. **Visual and approachable.** Immediate previews, readable errors, sensible
   defaults, drag-and-drop, and a polished responsive interface.
5. **Engine-neutral core.** Rust implements processing; front ends and exporters
   consume the same underlying results.
6. **Honest failures.** Never silently discard sprites or corrupt metadata.
   Show exactly what does not fit and what the user can change.
7. **Useful without Sindri.** Engine-specific integrations are adapters, not
   requirements or special cases in the core.
8. **Portable and extensible.** Browser first, desktop later; avoid tying
   project data to a particular host.

## Audience and core jobs

- **Solo developer:** turn a folder of PNGs into an atlas and importable
  metadata in minutes.
- **Pixel artist:** inspect sprites, remove excess transparent space, and
  maintain consistent animation frames and alignment.
- **2D game developer:** build terrain/autotile sets and preview transitions
  before importing them into a game.
- **Small team:** save preparation settings, repeat exports, and track which
  generated assets came from which originals.

The first audience is developers with 2D sprite-based projects. Avoid trying
to become a general image editor, 3D modeling suite, or game engine.

## Intended workflow

1. Create or open a Hewn project, or start a disposable session.
2. Import files/folders, inspect previews, and preserve source provenance.
3. Organize assets into named groups, animations, terrain sets, or variants.
4. Choose preparation operations and inspect the result immediately.
5. Validate dimensions, transparent bounds, frame origins, collisions, and
   atlas utilization; explain problems before export.
6. Export images and machine-readable metadata for the target engine.
7. Reopen the project and regenerate identical outputs after source changes.

An import-to-export path must stay fast even as advanced features are added.

## Feature areas

### 1. Asset library and inspection
- Multi-file and folder import, search, tags, groups, sorting, rename aliases.
- Sprite previews, original dimensions, alpha bounds, offsets, and file details.
- Duplicate detection: exact hashes first; perceptual similarity later.
- Warnings for invalid images, naming conflicts, and inconsistent dimensions.
- Non-destructive batch operations and undo/redo where appropriate.
- Project-relative source references and clear missing-file recovery.

### 2. Texture atlas builder
- Transparent-edge trimming with original dimensions and offsets preserved.
- Configurable atlas size, padding, power-of-two presets, and preview grid.
- Deterministic rectangle packing, with utilization metrics and collision tests.
- Multiple atlas pages and explicit handling of sprites that cannot fit.
- Optional rotation only when export metadata and engine adapters support it.
- Stable naming and frame ordering; no silent sprite omission.
- PNG atlas plus versioned JSON as the baseline output.

### 3. Sprite animation preparation
- Group frames into named sequences (idle, run, attack, etc.).
- Reorder frames, preview playback, set FPS or per-frame durations.
- Consistent pivots/origins, offsets, and optional directional variants.
- Export frame sequence metadata and references to atlas frames.
- Detect missing or mismatched frames before export.

### 4. Terrain and autotiling
- Organize terrain tiles and assign semantic roles and edge rules.
- Support common neighbor masks and clearly document bit conventions.
- Preview a painted test map to expose missing corners/transitions.
- Validate incomplete rule sets and generate reusable terrain metadata.
- Start with a small, well-defined 2D tile format; expand adapters later.

### 5. Export and integrations
- Document a versioned, engine-neutral JSON schema.
- Support atlas PNG, frame rectangles, source offsets, pivots, and animations.
- Provide dedicated Godot, Unity, and Sindri adapters after the base schema
  stabilizes. Verify formats against actual consumers, not assumptions.
- Offer reproducible export presets and eventually batch/CLI export.

### 6. Workspace and experience
- Clear three-part workbench: asset library, central visual preview, inspector.
- Dark, intentional visual language consistent with the Sindri family.
- Keyboard shortcuts, accessible labels, responsive layouts, and helpful
  empty states. Avoid hiding essential actions in tiny icon-only controls.
- Project save/open and settings persistence without a cloud dependency.
- Desktop host with native filesystem access when the browser experience
  proves the processing model.

## Architecture and boundaries

```text
Source PNGs / project files
          |
          v
   hewn-core (Rust)
   - image bounds and transforms
   - atlas packing and metadata
   - future animation / terrain operations
          |
    +-----+-------------------+
    |                         |
hewn-wasm                  future native API
(wasm-bindgen adapter)     / CLI / Tauri commands
    |                         |
React + TypeScript         Tauri desktop shell
Vite browser UI            (reuse UI where practical)
    |
Export PNG + JSON / engine adapters
```

**Core owns processing.** The browser should not maintain a second packer.
WebAssembly receives RGBA pixel buffers and a manifest, calls the Rust core,
and returns an atlas buffer plus frame metadata. UI state and drawing remain
in TypeScript. The bridge should validate buffer lengths and input sizes.

Keep exports and project data versioned. Establish golden fixtures so Rust,
WASM, and future native hosts agree on the same results. Preserve source image
bytes; never silently rewrite an imported file.

### Technology choices

- **Rust:** portable processing and tests, without UI dependencies.
- **React + TypeScript + Vite:** interactive visual workbench.
- **wasm-bindgen / wasm-pack:** browser access to the Rust core.
- **Tauri (planned, not implemented):** desktop filesystem and packaging.
- **No mandatory AI or backend.** Optional future conveniences must not
  compromise the offline, private baseline.

## Delivery roadmap

### M0: Foundation | completed in PR #1
- [x] Repository conventions, Rust workspace, and CI.
- [x] Browser workbench, PNG import, sample assets, inspector, atlas preview.
- [x] Transparent trimming, shelf packing, source offsets, PNG/JSON export.
- [x] Rust core tests and automatic formatting workflow.
- [ ] Broader integration tests and robust edge-case handling.

### M1: One shared packing implementation | PR #2, pending verification
- [x] Introduce a Rust-to-WASM bridge in the feature branch.
- [x] Route browser packing through Rust and remove duplicate TypeScript packer.
- [x] Add WASM build prerequisites to web scripts and CI.
- [ ] CI green on the PR's final commit; confirm browser behavior manually.
- [ ] Add cross-host golden fixtures, including transparency and overflow.
- [ ] Validate large inputs, memory use, and error messages.

### M2: Dependable atlas production
- [ ] Verify Auto Optimize (implemented in feature branch): compare supported rectangular power-of-two sizes using geometry-only packing, minimize RGBA8 allocation, break ties by page count, and offer explicit application of recommendations.
- [ ] Future optimization profiles: prioritize fewer pages, balanced memory, or target-engine limits.
- [ ] Replace/augment shelf packing with deterministic efficient rectangles.
- [ ] Add multi-page atlases and utilization feedback.
- [ ] Validate padding, edge cases, overlaps, and metadata with fixtures.
- [ ] Add repeatable import/export tests and predictable asset ordering.
- [ ] Make errors actionable; support large sprite sets without UI freezes.

### M3: Real projects and organization
- [ ] Define a versioned Hewn project file and migration policy.
- [ ] Save/open projects, restore asset links, and report missing sources.
- [ ] Search, groups/tags, naming controls, exact duplicate detection.
- [ ] Add export presets and repeatable rebuilds.

### M4: Animation workbench
- [ ] Named animation sequences, ordering, playback and durations.
- [ ] Pivots, origins, directional variants, validation.
- [ ] Versioned animation export and a sample consuming game.

### M5: Terrain composer
- [ ] Terrain tile collections and neighbor-mask conventions.
- [ ] Visual rule editor, paintable map preview, missing-rule warnings.
- [ ] Export rule metadata and test it in at least one real game.

### M6: Distribution and engine adapters
- [ ] Tauri desktop packaging, filesystem workflows, and platform builds.
- [ ] Document generic JSON schema and verify Godot/Unity/Sindri adapters.
- [ ] Optional CLI/batch mode using the same core.
- [ ] Accessibility, performance, documentation, and release readiness.

Milestones are priorities, not deadlines. Do not check a box merely because
a branch exists; distinguish implementation from tested and merged delivery.

## Acceptance criteria

- A first-time user can import PNGs, configure trimming/padding, preview an
  atlas, and export usable PNG/JSON without reading a manual.
- No source image is modified or uploaded.
- Frame rectangles never overlap, and every input is represented or produces
  an explicit failure.
- Trimming retains enough metadata to reconstruct sprite placement.
- Identical inputs/settings yield identical output geometry and metadata.
- Browser and native/WASM processing agree on shared golden fixtures.
- A saved project can be reopened and regenerated after restarting the app.
- Export adapters are exercised in real target-engine projects.
- CI tests formatting, Rust correctness, and the browser production build.

## Deliberate non-goals for early versions

- Generative AI, hosted asset processing, user accounts, subscriptions.
- A Photoshop replacement, full pixel editor, or 3D asset authoring suite.
- A game engine or scene editor inside Hewn.
- Dozens of exporters before a stable, tested generic schema exists.

## Risks and questions to resolve

- **Packing:** optimize space without breaking deterministic output or
  changing frame references unexpectedly.
- **Browser memory:** decoding all images and copying RGBA into WASM may
  exhaust memory on large collections. Measure before promising scale.
- **Project portability:** define relative paths and embedded-vs-linked
  asset behavior before committing to a format.
- **Origins and trimming:** document coordinate conventions and test
  animations that share pivots across frames.
- **Autotiling:** agree on mask convention and target-engine expectations.
- **WASM toolchain:** keep developer setup and CI reliable and documented.
- **Desktop scope:** ship a useful browser workbench before adding packaging
  complexity for its own sake.

## Project identity and naming

- Name: **Sindri: Hewn**.
- Pronunciation: **hyoon**.
- Tagline: **Raw assets, refined.**
- Positioning: a focused asset-preparation studio in the Sindri family,
  deliberately compatible with other engines.
- Visual direction: a refined creative workbench, not an admin dashboard.

## Maintaining this plan

Use this file as the long-lived source of truth for the product direction.
Keep the README short and link here. Every substantial feature PR should
update the relevant milestone and describe what was actually tested.
If priorities change, record the decision and its rationale rather than
quietly rewriting the product into something unrelated.
