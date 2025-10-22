# Dungeon Platformer Scaffold

Rust + Bevy project scaffold that targets desktop and WebAssembly builds while preparing the ground for an LDtk-driven 2D platformer with responsive player movement, audio, and puzzle-friendly state management.

## Project Layout

- `src/` – Bevy application entry point and modular plugins (`app`, `level`, `movement`, `player`, `audio`, `ui`).
- `assets/levels/` – Place your LDtk project files here (e.g. `test_map_1.ldtk`).
- `assets/audio/` – Drop sound effects and music referenced in `audio.rs`.
- `assets/textures/` – Reserved for sprites and tilesets.

You can rename or expand these folders as you shape the game content.

## Getting Started

1. Install the latest stable Rust toolchain: <https://rustup.rs>
2. (Optional, for web builds) Install a WASM runner such as `trunk` or `wasm-server-runner`.

### Native Build

```bash
cargo run
```

### WebAssembly Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --no-default-features --features web
```

Use your preferred runner/bundler to serve the contents of `target/wasm32-unknown-unknown/debug/`.
Embed a canvas with the id `bevy-canvas` in your HTML shell if you target the browser.

## LDtk Integration

- Update `assets/levels/test_map_1_newres.ldtk` (or change the path in `LevelConfig` inside `src/level.rs`) with your actual LDtk project.
- Mark solid tiles in an IntGrid layer with a non-zero value so the in-game collision map can detect walkable surfaces.
- Ensure level identifiers in LDtk align with `start_level` in `LevelConfig`.

## Assets & Audio

`audio.rs` preloads placeholder handles for jump, pickup, and ambient tracks. Replace them with real audio files in `assets/audio/` and expand the resource as needed. Sprites, tilesets, and textures belong under `assets/textures/`.

## Next Steps

- Flesh out collision detection (consider Bevy Rapier or XPBD for physics).
- Wire LDtk entity layers to spawn interactable objects.
- Implement serialization hooks for save/checkpoint systems.
- Replace placeholder UI with themed menus and HUD elements.
