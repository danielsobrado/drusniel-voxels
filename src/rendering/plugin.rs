use bevy::prelude::*;
use crate::rendering::atlas::load_texture_atlas;
use crate::rendering::materials::{configure_atlas_sampler, setup_voxel_material};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                load_texture_atlas,
                setup_voxel_material,
            ).chain())
            .add_systems(Update, configure_atlas_sampler);
    }
}
