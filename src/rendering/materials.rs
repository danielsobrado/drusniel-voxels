use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use crate::rendering::atlas::TextureAtlas;

#[derive(Resource)]
pub struct VoxelMaterial {
    pub handle: Handle<StandardMaterial>,
}

/// Material for surface nets smooth terrain (opaque, no alpha testing)
#[derive(Resource)]
pub struct SurfaceNetsMaterial {
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

    // Surface nets material - fully opaque for smooth terrain (no alpha testing)
    let surface_nets_handle = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.handle.clone()),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        reflectance: 0.1,
        cull_mode: None,
        alpha_mode: AlphaMode::Opaque, // No alpha testing - always render solid
        ..default()
    });

    commands.insert_resource(SurfaceNetsMaterial {
        handle: surface_nets_handle,
    });

    // Water material - semi-transparent blue with proper depth handling
    let water_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.5, 0.8, 0.7), // Brighter blue with transparency
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.05, // Very smooth surface
        metallic: 0.0,
        reflectance: 0.8, // High reflection for water look
        double_sided: true, // Visible from below
        cull_mode: None, // Render both sides
        depth_bias: 0.0,
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

