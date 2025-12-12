# Debugging Rabbit Texture Issues

## Issue Description
The rabbit entities were rendering as untextured white models, despite the code apparently loading and applying a texture image.

## Symptoms
- Rabbits appeared pure white.
- Logs indicated the texture image was loaded (`texture_loaded=true`).
- Manually forcing the material color to Red worked, proving we had control over the material component.
- However, applying the texture had no visual effect.

## Root Cause Analysis
Deep inspection of the `Mesh` component revealed that while the GLTF model technically had a UV attribute (`ATTRIBUTE_UV_0`), **all UV coordinates were set to `(0.0, 0.0)`**.

This "Zero UV" state meant that for every vertex on the rabbit, the renderer was sampling the exact same bottom-left pixel of the texture. Since the texture likely has a solid color at that corner, the entire rabbit appeared as a single flat color (white).

## Solution
We implemented a **Runtime Procedural UV Generator** within `src/entity/rabbit.rs`.

### The Fix Logic
1.  **Detection**: The system allows the rabbit to spawn. It then queries the mesh data.
2.  **Validation**: It counts how many UV coordinates are non-zero. If the count is effectively zero (degenerate), it triggers a fix.
3.  **Generation (Cylindrical Mapping)**:
    - We calculate the bounding box of the mesh.
    - We iterate over every vertex position.
    - We calculate `angle = atan2(z, x)` to determine the horizontal coordinate `u`.
    - We calculate `height_ratio = (y - min_y) / height` to determine the vertical coordinate `v`.
    - We apply a **2x Tiling Scale** to the UVs to make the fur texture appear denser.
4.  **Application**: The new UVs are injected back into the `Mesh` asset at runtime.
5.  **Tangents**: We call `mesh.generate_tangents()` to ensure lighting calculations remain correct with the new UVs.

### Why Cylindrical?
We initially tried **Planar Mapping** (Top-Down projection), but this caused severe stretching on the vertical sides of the rabbit. **Cylindrical Mapping** wraps the texture around the body, which is much more natural for animal shapes.

## Code Reference
See `fix_rabbit_textures` in `src/entity/rabbit.rs`.

```rust
// Simplified logic snippet
let u = (angle / (PI * 2.0)) + 0.5;
let v = dy / height;
mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, ...);
```
