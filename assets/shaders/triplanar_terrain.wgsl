// Triplanar PBR terrain shader with multiple material support
// Selects grass/rock/sand/dirt based on atlas index passed through UV.x

#import bevy_pbr::forward_io::VertexOutput

// Triplanar uniforms - matches TriplanarUniforms struct in Rust
struct TriplanarUniforms {
    base_color: vec4<f32>,     // Base color tint
    tex_scale: f32,            // World units per texture tile (lower = higher res)
    blend_sharpness: f32,      // Controls blend falloff (higher = sharper)
    normal_intensity: f32,     // Normal map strength
    _padding: f32,             // Padding for alignment
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> uniforms: TriplanarUniforms;

// Grass textures (material 0)
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var grass_albedo: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tex_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var grass_normal: texture_2d<f32>;

// Rock textures (material 1)
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var rock_albedo: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var rock_normal: texture_2d<f32>;

// Sand textures (material 2)
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var sand_albedo: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var sand_normal: texture_2d<f32>;

// Dirt textures (material 3)
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var dirt_albedo: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var dirt_normal: texture_2d<f32>;

// Compute tiled UV from world coordinates
fn compute_uv(world_coord: vec2<f32>) -> vec2<f32> {
    let tex_scale = uniforms.tex_scale;
    let scaled = world_coord / tex_scale;
    return fract(scaled);
}

// Calculate triplanar blend weights from world normal
fn triplanar_weights(world_normal: vec3<f32>) -> vec3<f32> {
    let sharpness = uniforms.blend_sharpness;
    var weights = pow(abs(world_normal), vec3(sharpness));
    let weight_sum = weights.x + weights.y + weights.z;
    weights = weights / max(weight_sum, 0.001);
    return weights;
}

// Unpack normal from texture (0-1 range to -1 to 1 range)
fn unpack_normal(sampled: vec3<f32>) -> vec3<f32> {
    return normalize(sampled * 2.0 - 1.0);
}

// Reorient tangent-space normal to world space for a triplanar projection
fn reorient_normal(tangent_normal: vec3<f32>, world_normal: vec3<f32>, axis: i32) -> vec3<f32> {
    var n = tangent_normal;
    let intensity = uniforms.normal_intensity;
    n = vec3(n.xy * intensity, n.z);
    n = normalize(n);
    
    var world_n: vec3<f32>;
    if (axis == 0) {
        world_n = vec3(n.z * sign(world_normal.x), n.y, n.x);
    } else if (axis == 1) {
        world_n = vec3(n.x, n.z * sign(world_normal.y), n.y);
    } else {
        world_n = vec3(n.x, n.y, n.z * sign(world_normal.z));
    }
    
    return normalize(world_n);
}

// Sample albedo for a given material index from 3 projections
fn sample_albedo_triplanar(mat_idx: i32, uv_yz: vec2<f32>, uv_xz: vec2<f32>, uv_xy: vec2<f32>, weights: vec3<f32>) -> vec4<f32> {
    var col_x: vec4<f32>;
    var col_y: vec4<f32>;
    var col_z: vec4<f32>;
    
    if (mat_idx == 0) {
        // Grass
        col_x = textureSample(grass_albedo, tex_sampler, uv_yz);
        col_y = textureSample(grass_albedo, tex_sampler, uv_xz);
        col_z = textureSample(grass_albedo, tex_sampler, uv_xy);
    } else if (mat_idx == 2) {
        // Rock
        col_x = textureSample(rock_albedo, tex_sampler, uv_yz);
        col_y = textureSample(rock_albedo, tex_sampler, uv_xz);
        col_z = textureSample(rock_albedo, tex_sampler, uv_xy);
    } else if (mat_idx == 4) {
        // Sand
        col_x = textureSample(sand_albedo, tex_sampler, uv_yz);
        col_y = textureSample(sand_albedo, tex_sampler, uv_xz);
        col_z = textureSample(sand_albedo, tex_sampler, uv_xy);
    } else {
        // Dirt (default for 1, 7, and others)
        col_x = textureSample(dirt_albedo, tex_sampler, uv_yz);
        col_y = textureSample(dirt_albedo, tex_sampler, uv_xz);
        col_z = textureSample(dirt_albedo, tex_sampler, uv_xy);
    }
    
    return col_x * weights.x + col_y * weights.y + col_z * weights.z;
}

// Sample normal map for a given material index from 3 projections
fn sample_normal_triplanar(mat_idx: i32, uv_yz: vec2<f32>, uv_xz: vec2<f32>, uv_xy: vec2<f32>, weights: vec3<f32>, world_normal: vec3<f32>) -> vec3<f32> {
    var norm_x_raw: vec3<f32>;
    var norm_y_raw: vec3<f32>;
    var norm_z_raw: vec3<f32>;
    
    if (mat_idx == 0) {
        // Grass
        norm_x_raw = textureSample(grass_normal, tex_sampler, uv_yz).rgb;
        norm_y_raw = textureSample(grass_normal, tex_sampler, uv_xz).rgb;
        norm_z_raw = textureSample(grass_normal, tex_sampler, uv_xy).rgb;
    } else if (mat_idx == 2) {
        // Rock
        norm_x_raw = textureSample(rock_normal, tex_sampler, uv_yz).rgb;
        norm_y_raw = textureSample(rock_normal, tex_sampler, uv_xz).rgb;
        norm_z_raw = textureSample(rock_normal, tex_sampler, uv_xy).rgb;
    } else if (mat_idx == 4) {
        // Sand
        norm_x_raw = textureSample(sand_normal, tex_sampler, uv_yz).rgb;
        norm_y_raw = textureSample(sand_normal, tex_sampler, uv_xz).rgb;
        norm_z_raw = textureSample(sand_normal, tex_sampler, uv_xy).rgb;
    } else {
        // Dirt
        norm_x_raw = textureSample(dirt_normal, tex_sampler, uv_yz).rgb;
        norm_y_raw = textureSample(dirt_normal, tex_sampler, uv_xz).rgb;
        norm_z_raw = textureSample(dirt_normal, tex_sampler, uv_xy).rgb;
    }
    
    let normal_x = reorient_normal(unpack_normal(norm_x_raw), world_normal, 0);
    let normal_y = reorient_normal(unpack_normal(norm_y_raw), world_normal, 1);
    let normal_z = reorient_normal(unpack_normal(norm_z_raw), world_normal, 2);
    
    return normalize(normal_x * weights.x + normal_y * weights.y + normal_z * weights.z);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position.xyz;
    let world_normal = normalize(in.world_normal);

    // Get material index from UV.x (stored by meshing code)
    // Atlas indices: 0=grass top, 1=dirt, 2=rock, 3=bedrock, 4=sand, 5=clay, 7=grass side
    let atlas_idx = i32(floor(in.uv.x + 0.5));
    
    // Calculate blend weights based on normal
    let weights = triplanar_weights(world_normal);

    // Compute UVs for each projection plane
    let uv_yz = compute_uv(world_pos.yz);
    let uv_xz = compute_uv(world_pos.xz);
    let uv_xy = compute_uv(world_pos.xy);

    // Sample albedo and normal based on material
    var albedo = sample_albedo_triplanar(atlas_idx, uv_yz, uv_xz, uv_xy, weights);
    albedo = albedo * uniforms.base_color;
    
    let blended_normal = sample_normal_triplanar(atlas_idx, uv_yz, uv_xz, uv_xy, weights, world_normal);

    // PBR-style lighting
    let light_dir = normalize(vec3(0.4, 0.8, 0.3));
    let view_dir = normalize(-in.world_position.xyz);
    let half_dir = normalize(light_dir + view_dir);
    
    // Diffuse (Lambert)
    let ndotl = max(dot(blended_normal, light_dir), 0.0);
    let ambient = 0.35;
    let diffuse = ndotl * 0.65;
    
    // Specular (subtle)
    let ndoth = max(dot(blended_normal, half_dir), 0.0);
    let specular = pow(ndoth, 32.0) * 0.15;
    
    // Combine lighting
    let lit_color = albedo.rgb * (ambient + diffuse) + vec3(specular);

    return vec4(lit_color, albedo.a);
}
