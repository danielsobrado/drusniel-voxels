use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use crate::rendering::atlas::TextureAtlas;
use crate::rendering::triplanar_material::{TriplanarMaterial, TriplanarMaterialHandle, TriplanarUniforms};

#[derive(Resource)]
pub struct VoxelMaterial {
    pub handle: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct WaterMaterial {
    pub handle: Handle<StandardMaterial>,
}

pub fn setup_voxel_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atlas: Res<TextureAtlas>,
) {
    // Solid block material
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.handle.clone()),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        reflectance: 0.1,
        // Disable backface culling to visualize all faces while we debug missing quads
        cull_mode: None,
        // Use a mask so leaves/foliage can leverage alpha but keep opaque blocks solid
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    commands.insert_resource(VoxelMaterial {
        handle: material_handle,
    });

    // Water material - semi-transparent blue with proper depth handling
    // Use positive depth_bias to push water behind terrain, preventing visible seams
    let water_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.3, 0.8, 0.7), // Semi-transparent blue
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.05, // Very smooth surface
        metallic: 0.0,
        reflectance: 0.8, // High reflection for water look
        double_sided: true, // Visible from below
        cull_mode: None, // Render both sides
        depth_bias: 1.0, // Push water behind terrain to hide seams
        ..default()
    });

    commands.insert_resource(WaterMaterial {
        handle: water_handle,
    });
}

/// Ensure the atlas uses a repeat/mipmapped sampler so tiled terrain does not clamp or alias
pub fn configure_atlas_sampler(
    atlas: Res<TextureAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut configured: Local<bool>,
) {
    if *configured {
        return;
    }

    if let Some(image) = images.get_mut(&atlas.handle) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            ..default()
        });
        *configured = true;
    }
}

/// Setup triplanar terrain material for surface nets meshes with PBR textures
/// Loads grass, rock, sand, and dirt texture sets for multi-material terrain
pub fn setup_triplanar_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<TriplanarMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let material_handle = materials.add(TriplanarMaterial {
        uniforms: TriplanarUniforms {
            base_color: LinearRgba::WHITE,
            tex_scale: 2.0,         // Higher resolution (1 tile per 2 world units)
            blend_sharpness: 4.0,   // Moderate blend between projections
            normal_intensity: 1.0,  // Full normal map strength
            parallax_scale: 0.04,   // Subtle parallax depth
        },
        // Grass textures (for TopSoil top faces)
        grass_albedo: Some(asset_server.load("pbr/grass/albedo.png")),
        grass_normal: Some(asset_server.load("pbr/grass/normal.png")),
        // Rock textures (for Rock, Bedrock, cliffs)
        rock_albedo: Some(asset_server.load("pbr/rock/albedo.png")),
        rock_normal: Some(asset_server.load("pbr/rock/normal.png")),
        // Sand textures
        sand_albedo: Some(asset_server.load("pbr/sand/albedo.png")),
        sand_normal: Some(asset_server.load("pbr/sand/normal.png")),
        // Dirt textures (for SubSoil, sides)
        dirt_albedo: Some(asset_server.load("pbr/dirt/albedo.png")),
        dirt_normal: Some(asset_server.load("pbr/dirt/normal.png")),
    });

    commands.insert_resource(TriplanarMaterialHandle {
        handle: material_handle,
    });
}

