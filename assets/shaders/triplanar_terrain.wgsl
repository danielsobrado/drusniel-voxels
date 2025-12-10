// True triplanar terrain shader with 3-sample blending
// Fragment-only shader that uses Bevy's standard vertex outputs

#import bevy_pbr::forward_io::VertexOutput

// Combined triplanar uniforms - matches TriplanarUniforms struct
struct TriplanarUniforms {
    base_color: vec4<f32>,     // Base color tint
    tex_scale: f32,            // World units per texture tile
    blend_sharpness: f32,      // Controls blend falloff (higher = sharper)
    atlas_size: f32,           // Number of tiles per row/column (e.g., 4.0)
    padding: f32,              // UV padding to prevent bleeding
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> uniforms: TriplanarUniforms;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var color_sampler: sampler;

// Compute UV within an atlas tile for a given world coordinate pair
fn compute_tile_uv(world_coord: vec2<f32>, atlas_index: u32) -> vec2<f32> {
    let atlas_size = uniforms.atlas_size;
    let tex_scale = uniforms.tex_scale;
    let padding = uniforms.padding;

    let tile_size = 1.0 / atlas_size;
    let usable_size = tile_size - padding * 2.0;

    let tile_x = f32(atlas_index % u32(atlas_size));
    let tile_y = f32(atlas_index / u32(atlas_size));

    let u_base = tile_x * tile_size + padding;
    let v_base = tile_y * tile_size + padding;

    // Scale world coordinates and get fractional part for tiling
    let scaled = world_coord / tex_scale;

    // Use fract to get repeating pattern, handling negatives correctly
    var frac_u = fract(scaled.x);
    var frac_v = fract(scaled.y);

    // Clamp to avoid edge bleeding
    frac_u = clamp(frac_u, 0.01, 0.99);
    frac_v = clamp(frac_v, 0.01, 0.99);

    return vec2(
        u_base + frac_u * usable_size,
        v_base + frac_v * usable_size
    );
}

// Calculate triplanar blend weights from world normal
fn triplanar_weights(world_normal: vec3<f32>) -> vec3<f32> {
    let sharpness = uniforms.blend_sharpness;

    // Compute blend weights from absolute normal components
    var weights = pow(abs(world_normal), vec3(sharpness));

    // Normalize so weights sum to 1
    let weight_sum = weights.x + weights.y + weights.z;
    weights = weights / max(weight_sum, 0.001);

    return weights;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get atlas index from UV.x (stored by meshing code)
    let atlas_idx = u32(in.uv.x + 0.5);

    let world_pos = in.world_position.xyz;
    let world_normal = normalize(in.world_normal);

    // Calculate blend weights
    let weights = triplanar_weights(world_normal);

    // Compute UVs for each projection plane
    let uv_yz = compute_tile_uv(world_pos.yz, atlas_idx);  // X-facing (east/west)
    let uv_xz = compute_tile_uv(world_pos.xz, atlas_idx);  // Y-facing (top/bottom)
    let uv_xy = compute_tile_uv(world_pos.xy, atlas_idx);  // Z-facing (north/south)

    // Sample texture from all 3 projections
    let sample_x = textureSample(color_texture, color_sampler, uv_yz);
    let sample_y = textureSample(color_texture, color_sampler, uv_xz);
    let sample_z = textureSample(color_texture, color_sampler, uv_xy);

    // Blend samples based on weights
    var color = sample_x * weights.x + sample_y * weights.y + sample_z * weights.z;

    // Apply base color tint
    color = color * uniforms.base_color;

    // Simple directional lighting
    let light_dir = normalize(vec3(0.3, 1.0, 0.2));
    let ndotl = max(dot(world_normal, light_dir), 0.0);
    let ambient = 0.4;
    let diffuse = 0.6 * ndotl;

    let lit_color = color.rgb * (ambient + diffuse);

    return vec4(lit_color, color.a);
}
