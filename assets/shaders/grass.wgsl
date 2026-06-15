// Grass wind shader - Valheim-style swaying animation
// Self-contained shader that doesn't rely on Bevy's forward_io

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world}
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

struct GrassMaterial {
    base_color: vec4<f32>,
    tip_color: vec4<f32>,
    wind_strength: f32,
    wind_speed: f32,
    wind_scale: f32,
    time: f32,
};

@group(2) @binding(0)
var<uniform> material: GrassMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// Simple noise function for wind variation
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian Motion for more natural wind
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;

    for (var i = 0; i < 3; i++) {
        value += amplitude * noise(pos * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Get world position of vertex
    let world_from_local = get_world_from_local(vertex.instance_index);
    var world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));

    // Calculate wind displacement based on height (UV.y = 1 at bottom, 0 at top)
    let height_factor = 1.0 - vertex.uv.y; // 0 at bottom, 1 at top
    let height_factor_smooth = height_factor * height_factor; // Quadratic falloff

    // Wind calculation using world position for coherent movement
    let wind_time = material.time * material.wind_speed;
    let wind_pos = world_position.xz * material.wind_scale;

    // Primary wind wave
    let wind1 = sin(wind_pos.x * 0.5 + wind_time) * cos(wind_pos.y * 0.3 + wind_time * 0.7);

    // Secondary turbulence
    let wind2 = fbm(wind_pos * 2.0 + vec2<f32>(wind_time * 0.5, wind_time * 0.3));

    // Gusts - occasional stronger wind
    let gust = sin(wind_time * 0.2) * 0.5 + 0.5;
    let gust_strength = gust * gust * 0.5;

    // Combine wind effects
    let wind_x = (wind1 * 0.6 + wind2 * 0.4) * material.wind_strength * (1.0 + gust_strength);
    let wind_z = (wind1 * 0.4 + wind2 * 0.6) * material.wind_strength * 0.7 * (1.0 + gust_strength);

    // Apply displacement
    world_position.x += wind_x * height_factor_smooth;
    world_position.z += wind_z * height_factor_smooth;

    // Slight vertical compression when bent
    let bend_amount = abs(wind_x) + abs(wind_z);
    world_position.y -= bend_amount * height_factor_smooth * 0.1;

    out.position = view.clip_from_world * world_position;
    out.world_position = world_position;
    out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.uv = vertex.uv;

    // Interpolate color from base to tip based on height
    out.color = mix(material.base_color, material.tip_color, height_factor);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.3));
    let normal = normalize(in.world_normal);
    let ndotl = max(dot(normal, light_dir), 0.0);
    let ambient = 0.5;
    let diffuse = ndotl * 0.5;

    var color = in.color.rgb * (ambient + diffuse);

    // Add slight subsurface scattering effect (grass glows when backlit)
    let sss = (1.0 - abs(dot(normal, light_dir))) * 0.1;
    color += vec3<f32>(0.2, 0.25, 0.1) * sss;

    // Alpha test - always opaque for grass
    return vec4<f32>(color, 1.0);
}
