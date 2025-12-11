use bevy::prelude::*;
use voxel_builder::camera::plugin::CameraPlugin;
use voxel_builder::environment::AtmospherePlugin;
use voxel_builder::interaction::InteractionPlugin;
use voxel_builder::props::PropsPlugin;
use voxel_builder::rendering::plugin::RenderingPlugin;
use voxel_builder::vegetation::VegetationPlugin;
use voxel_builder::viewmodel::PickaxePlugin;
use voxel_builder::voxel::plugin::VoxelPlugin;
use voxel_builder::entity::EntityPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(VoxelPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(PickaxePlugin)
        .add_plugins(VegetationPlugin)
        .add_plugins(PropsPlugin)
        .add_plugins(AtmospherePlugin)
        .add_plugins(EntityPlugin)
        .run();
}
