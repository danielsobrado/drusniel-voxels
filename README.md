# Drusniel Voxels

Rust voxel game prototype based on the Drusniel world.

![v0.3 Preview](assets/images/V0.3.jpg)

## v0.3

This version improves terrain rendering, materials, and movement over uneven voxel terrain:

- PBR material blending for terrain
- Parallax-style rock texture detail
- Smooth triplanar material blending with vertex weights
- Surface Nets seam and UV mapping improvements
- Repeat sampler fixes for terrain textures
- Material and mesh generation updates
- Smoother slope movement
- Bilinear terrain height detection for the character controller
- Step-up movement logic for better traversal

## Setup

Install Rust, then build from the repository root:

```bash
cargo build
```

Run the game:

```bash
cargo run
```

## Project Layout

| Path | Role |
|---|---|
| `src/main.rs` | Runtime entry point |
| `src/lib.rs` | Shared module root |
| `src/camera/` | Camera controller and plugin |
| `src/config/` | YAML config loading |
| `src/environment.rs` | Environment and sky setup |
| `src/rendering/` | Materials, atlas loading, triplanar terrain, and rendering plugin |
| `src/vegetation/` | Grass material setup |
| `src/voxel/` | Voxel world, chunks, meshing, persistence, and types |
| `assets/config/` | World and voxel type settings |
| `assets/models/` | Runtime model assets |
| `assets/pbr/` | PBR terrain texture sets |
| `assets/shaders/` | WGSL shaders |
| `assets/textures/atlas.png` | Runtime texture atlas |
