use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};
use bevy_shader::ShaderRef;

// Same structure as Triplanar for convenience, though we might not use all fields in Blocky
#[derive(Clone, Copy, ShaderType, Debug)]
pub struct BlockyUniforms {
    pub base_color: LinearRgba,
    pub tex_scale: f32, // Unused? Or for detail?
    pub blend_sharpness: f32, // Unused
    pub normal_intensity: f32,
    pub parallax_scale: f32,
}

impl Default for BlockyUniforms {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            tex_scale: 1.0,
            blend_sharpness: 1.0,
            normal_intensity: 1.0,
            parallax_scale: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BlockyMaterial {
    #[uniform(0)]
    pub uniforms: BlockyUniforms,

    // Array Texture (Albedo)
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub diffuse_texture: Option<Handle<Image>>,

    // Array Texture (Normal)
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub normal_texture: Option<Handle<Image>>,
}

impl Material for BlockyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blocky_terrain.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/blocky_terrain.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

#[derive(Resource)]
pub struct BlockyMaterialHandle {
    pub handle: Handle<BlockyMaterial>,
}
