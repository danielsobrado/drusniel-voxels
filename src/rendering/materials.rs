use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use crate::rendering::capabilities::GraphicsCapabilities;
use crate::rendering::triplanar_material::{TriplanarMaterial, TriplanarMaterialHandle, TriplanarUniforms};

use crate::rendering::blocky_material::BlockyMaterial;

#[derive(Resource)]
pub struct VoxelMaterial {
    pub handle: Handle<BlockyMaterial>,
}

#[derive(Resource)]
pub struct WaterMaterial {
    pub handle: Handle<StandardMaterial>,
}

// setup_voxel_material is now largely superseded by array_loader which creates the BlockyMaterial
// However, we might keep this signature if we want to initialize other things or just empty.
// For now, let's essentially empty it out or remove it from plugin if not needed.
// But wait, the system logic likely expects VoxelMaterial resource to exist.
// Let's modify array_loader to insert VoxelMaterial resource instead of its own internal handle.

pub fn setup_water_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Water material - semi-transparent blue with proper depth handling
    // Use positive depth_bias to push water behind terrain, preventing visible seams
    let water_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.4, 0.7, 0.6),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.02,  // Very smooth
        metallic: 0.0,
        reflectance: 0.9,            // High reflection
        double_sided: true,
        cull_mode: None,
        depth_bias: 0.5,             // Reduce z-fighting
        ..default()
    });

    commands.insert_resource(WaterMaterial {
        handle: water_handle,
    });
}

/// Ensure the atlas uses a repeat/mipmapped sampler so tiled terrain does not clamp or alias


/// Setup triplanar terrain material for surface nets meshes with PBR textures
/// Loads grass, rock, sand, and dirt texture sets for multi-material terrain
pub fn setup_triplanar_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<TriplanarMaterial>>,
    capabilities: Option<Res<GraphicsCapabilities>>,
    asset_server: Res<AssetServer>,
) {
    let integrated = capabilities
        .as_ref()
        .map(|capabilities| capabilities.integrated_gpu)
        .unwrap_or(false);

    let material_handle = materials.add(if integrated {
        TriplanarMaterial {
            uniforms: TriplanarUniforms {
                base_color: LinearRgba::WHITE,
                tex_scale: 2.0,
                blend_sharpness: 4.0,
                normal_intensity: 1.0,
                parallax_scale: 0.0,
            },
            grass_albedo: None,
            grass_normal: None,
            rock_albedo: None,
            rock_normal: None,
            sand_albedo: None,
            sand_normal: None,
            dirt_albedo: None,
            dirt_normal: None,
        }
    } else {
        TriplanarMaterial {
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
        }
    });

    commands.insert_resource(TriplanarMaterialHandle {
        handle: material_handle,
    });
}

/// Ensure all triplanar textures use Repeat address mode for seamless tiling
pub fn configure_triplanar_textures(
    mat_handle: Option<Res<TriplanarMaterialHandle>>,
    materials: Res<Assets<TriplanarMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut configured: Local<bool>,
) {
    if *configured {
        return;
    }

    if let Some(handle) = mat_handle {
        if let Some(material) = materials.get(&handle.handle) {
            let textures = [
                &material.grass_albedo, &material.grass_normal,
                &material.rock_albedo, &material.rock_normal,
                &material.sand_albedo, &material.sand_normal,
                &material.dirt_albedo, &material.dirt_normal,
            ];

            let mut all_loaded = true;
            for tex_opt in textures {
                if let Some(tex_handle) = tex_opt {
                    if let Some(image) = images.get_mut(tex_handle) {
                        // Set sampler to Repeat for tiling
                        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            address_mode_w: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            mipmap_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    } else {
                        // Texture not loaded yet
                        all_loaded = false;
                    }
                }
            }

            // Only mark as configured if we successfully processed all textures (or at least checked them)
            // If some are not loaded, we wait for next frame
            if all_loaded {
                *configured = true;
            }
        }
    }
}
