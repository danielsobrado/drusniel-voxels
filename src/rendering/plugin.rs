use bevy::prelude::*;
use crate::rendering::atlas::load_texture_atlas;
use crate::rendering::materials::{configure_atlas_sampler, setup_voxel_material, setup_triplanar_material};
use crate::rendering::triplanar_material::TriplanarMaterial;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register TriplanarMaterial as a custom material type
            .add_plugins(MaterialPlugin::<TriplanarMaterial>::default())
            .add_systems(Startup, (
                load_texture_atlas,
                setup_voxel_material,
                setup_triplanar_material,
            ).chain())
            .add_systems(Update, configure_atlas_sampler);
    }
}
