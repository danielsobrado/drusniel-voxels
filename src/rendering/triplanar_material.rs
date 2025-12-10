use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};
use bevy_shader::ShaderRef;

/// All triplanar material uniforms in a single struct for proper GPU alignment
#[derive(Clone, Copy, ShaderType, Debug)]
pub struct TriplanarUniforms {
    /// Base color tint (vec4)
    pub base_color: LinearRgba,
    /// World units per texture repeat (lower = higher resolution, e.g., 2.0)
    pub tex_scale: f32,
    /// How sharply to blend between projections (higher = sharper transitions)
    pub blend_sharpness: f32,
    /// Normal map intensity (1.0 = full strength)
    pub normal_intensity: f32,
    /// Padding for alignment
    pub _padding: f32,
}

impl Default for TriplanarUniforms {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            tex_scale: 2.0,
            blend_sharpness: 4.0,
            normal_intensity: 1.0,
            _padding: 0.0,
        }
    }
}

/// Custom triplanar PBR terrain material with multiple terrain types
/// Supports grass (0), dirt (1), rock (2), sand (4) based on atlas index in UV.x
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct TriplanarMaterial {
    #[uniform(0)]
    pub uniforms: TriplanarUniforms,

    // Grass textures (mat 0)
    #[texture(1)]
    #[sampler(2)]
    pub grass_albedo: Option<Handle<Image>>,
    #[texture(3)]
    pub grass_normal: Option<Handle<Image>>,

    // Rock textures (mat 1)
    #[texture(4)]
    pub rock_albedo: Option<Handle<Image>>,
    #[texture(5)]
    pub rock_normal: Option<Handle<Image>>,

    // Sand textures (mat 2)
    #[texture(6)]
    pub sand_albedo: Option<Handle<Image>>,
    #[texture(7)]
    pub sand_normal: Option<Handle<Image>>,

    // Dirt textures (mat 3)
    #[texture(8)]
    pub dirt_albedo: Option<Handle<Image>>,
    #[texture(9)]
    pub dirt_normal: Option<Handle<Image>>,
}

impl Default for TriplanarMaterial {
    fn default() -> Self {
        Self {
            uniforms: TriplanarUniforms::default(),
            grass_albedo: None,
            grass_normal: None,
            rock_albedo: None,
            rock_normal: None,
            sand_albedo: None,
            sand_normal: None,
            dirt_albedo: None,
            dirt_normal: None,
        }
    }
}

impl Material for TriplanarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/triplanar_terrain.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

/// Resource holding the triplanar terrain material handle
#[derive(Resource)]
pub struct TriplanarMaterialHandle {
    pub handle: Handle<TriplanarMaterial>,
}
