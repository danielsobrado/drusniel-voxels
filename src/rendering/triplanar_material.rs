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
    /// World units per texture repeat (e.g., 4.0 = 1 tile per 4 world units)
    pub tex_scale: f32,
    /// How sharply to blend between projections (higher = sharper transitions)
    pub blend_sharpness: f32,
    /// Number of tiles per row/column in atlas (e.g., 4.0 for 4x4 atlas)
    pub atlas_size: f32,
    /// UV padding to prevent bleeding at tile edges
    pub padding: f32,
}

impl Default for TriplanarUniforms {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            tex_scale: 4.0,
            blend_sharpness: 4.0,
            atlas_size: 4.0,
            padding: 0.03,
        }
    }
}

/// Custom triplanar terrain material with true 3-sample blending
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct TriplanarMaterial {
    #[uniform(0)]
    pub uniforms: TriplanarUniforms,

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
}

impl Default for TriplanarMaterial {
    fn default() -> Self {
        Self {
            uniforms: TriplanarUniforms::default(),
            color_texture: None,
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
