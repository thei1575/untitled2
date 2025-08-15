# Voxel Sandbox (Rust + Bevy)

Open-source voxel sandbox game scaffold, built with Rust and Bevy. This repository follows incremental milestones. Milestone 1 provides a compiling workspace and a Bevy window with a camera and light.

## Requirements
- Rust (stable toolchain)
- Git (for CI syncing)

## Build & Run

Fast path:

```
cargo run -p voxel_game --release
```

Or build only:

```
cargo build --release
```

Format and lint:

```
cargo fmt --all
cargo clippy --all-targets --all-features -p voxel_game -- -D warnings
```

## Controls (future milestones)
- WASD + mouse look
- Space to jump, Shift to crouch
- Left click remove, Right click place

## Workspace Layout
- crates/core: shared types and math
- crates/world: chunks, generation, save
- crates/render: meshing and rendering
- crates/game: Bevy app entry (binary)
- assets/: runtime assets (currently empty)

## License
MIT

