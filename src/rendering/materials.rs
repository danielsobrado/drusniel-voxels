use bevy::prelude::*;
use crate::rendering::atlas::TextureAtlas;

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

