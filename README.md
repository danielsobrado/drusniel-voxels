# Drusniel Voxels

Rust voxel game prototype based on the Drusniel world.

![v0.4 Preview](assets/images/v0.4-props1.jpg)

## v0.4

This version expands the project into a larger terrain, rendering, building, and world
interaction prototype.

Highlights:

- HDR rendering pipeline with tonemapping, bloom, debanding, and color grading
- Radiance-cascade global illumination experiments
- Adaptive GI quality presets
- GTAO ambient occlusion
- PCSS soft shadows
- Distance fog and volumetric fog
- Volumetric clouds
- Gerstner-style water with foam and caustic effects
- Weather particles for rain, snow, and dust
- Multi-layer vegetation wind animation
- Grass subsurface scattering and contact-shadow support
- Environment-map lighting for PBR reflections and ambient lighting
- Texture arrays with mipmaps and anisotropic filtering
- Expanded PBR materials for terrain, props, and buildings
- Chunk LOD with skirts and GPU fallback paths
- Prop LOD with billboards and mesh decimation
- Extended prop view distances
- Snap-point building system with ghost previews
- Terrain sculpting tools for raise, lower, level, and smooth operations
- Terrain conforming for placed props and buildings
- Save/load persistence
- Prop placement persistence
- Minimap, inventory, hotbar, chat, settings, debug overlays, and photo mode
- Config-driven tuning for rendering, weather, water, terrain, props, camera, and input

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
| `src/atmosphere/` | Atmosphere, fog, and sky integration |
| `src/building/` | Snap-point building system |
| `src/camera/` | Camera controller and configuration |
| `src/config/` | YAML and JSON config loading |
| `src/input/` | Input bindings and manager |
| `src/interaction/` | Targeting, editing, palette, and debug interaction |
| `src/menu/` | Menus, settings, multiplayer screen, and UI |
| `src/particles/` | Weather particle systems |
| `src/physics/` | Physics plugin, layers, and terrain colliders |
| `src/player/` | Player controller, input, spawning, and plugin |
| `src/props/` | Prop loading, placement, LOD, persistence, and rendering |
| `src/rendering/` | Rendering pipeline, materials, shadows, GI, water, clouds, and effects |
| `src/terrain/` | Terrain generation and terrain editing tools |
| `src/vegetation/` | Grass and wind animation |
| `src/voxel/` | Voxel world, chunks, meshing, LOD, occlusion, and persistence |
| `assets/config/` | Runtime configuration files |
| `assets/models/` | Runtime model assets |
| `assets/pbr/` | PBR texture sets |
| `assets/shaders/` | WGSL shaders |
| `assets/textures/` | Texture atlas, skybox, billboards, and terrain textures |
| `benches/` | Benchmark targets |
| `tests/` | Rust test suite |
