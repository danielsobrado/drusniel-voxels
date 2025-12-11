use bevy::prelude::*;
use crate::rendering::materials::{setup_water_material, setup_triplanar_material}; // Removed setup_voxel_material, configure_atlas_sampler
use crate::rendering::triplanar_material::TriplanarMaterial;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register TriplanarMaterial as a custom material type
            .add_plugins(MaterialPlugin::<TriplanarMaterial>::default())
            // Register BlockyMaterial
            .add_plugins(MaterialPlugin::<crate::rendering::blocky_material::BlockyMaterial>::default())
            .add_systems(Startup, (
                crate::rendering::array_loader::start_loading_texture_arrays, 
                setup_water_material,
                setup_triplanar_material,
            ).chain())
            .add_systems(Update, (
                crate::rendering::materials::configure_triplanar_textures,
                crate::rendering::array_loader::create_texture_array,
            ));
    }
}
