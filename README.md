# Voxel Sandbox (Rust + Bevy)

Open-source voxel sandbox game scaffold, built with Rust and Bevy. This repository follows incremental milestones.

**Current Status: Milestone 2** - Core chunk system, block registry, coordinate conversions, and procedural terrain generation with comprehensive tests.

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

## Features Implemented

### Milestone 1: Project Bootstrap
- ✅ Cargo workspace with modular crates
- ✅ Bevy app with 3D window, camera, and lighting
- ✅ CI/CD pipeline with format, clippy, build, and test
- ✅ MIT license and documentation

### Milestone 2: Core Chunk & Voxel System
- ✅ Block registry with palette compression
- ✅ Chunk data structures (16×16×256 with palette)
- ✅ Coordinate conversion utilities (world ↔ chunk ↔ local)
- ✅ Procedural terrain generation with Perlin noise
- ✅ Comprehensive unit tests for all core systems
- ✅ ChunkManager for loading/unloading chunks

## Workspace Layout
- **crates/core**: Shared types, math, block registry, and palette system
- **crates/world**: Chunk management and procedural terrain generation
- **crates/render**: Meshing and rendering (future milestone)
- **crates/game**: Bevy app entry point (binary)
- **assets/**: Runtime assets (currently empty)

## Testing
Run all tests with:
```bash
cargo test --workspace --all-features -- --nocapture
```

## License
MIT

