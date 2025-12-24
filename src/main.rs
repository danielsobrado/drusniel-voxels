use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use voxel_builder::camera::plugin::CameraPlugin;
use voxel_builder::chat::ChatPlugin;
use voxel_builder::entity::EntityPlugin;
use voxel_builder::environment::AtmospherePlugin;
use voxel_builder::atmosphere::FogPlugin;
use voxel_builder::interaction::InteractionPlugin;
use voxel_builder::map::MapPlugin;
use voxel_builder::menu::PauseMenuPlugin;
use voxel_builder::props::PropsPlugin;
use voxel_builder::rendering::plugin::RenderingPlugin;
use voxel_builder::vegetation::VegetationPlugin;
use voxel_builder::viewmodel::PickaxePlugin;
use voxel_builder::voxel::plugin::VoxelPlugin;
use voxel_builder::debug_ui::DebugUiPlugin;
use voxel_builder::particles::ParticlePlugin;

fn main() {
    let plugins = {
        #[cfg(target_os = "windows")]
        {
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::DX12),
                        ..default()
                    }),
                    ..default()
                })
        }
        #[cfg(not(target_os = "windows"))]
        {
            DefaultPlugins.set(ImagePlugin::default_nearest())
        }
    };

    App::new()
        .add_plugins(plugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(VoxelPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(PickaxePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(VegetationPlugin)
        .add_plugins(ChatPlugin)
        .add_plugins(PauseMenuPlugin)
        .add_plugins(PropsPlugin)
        .add_plugins(AtmospherePlugin)
        .add_plugins(FogPlugin)
        .add_plugins(EntityPlugin)
        .add_plugins(DebugUiPlugin)
        .add_plugins(ParticlePlugin)
        .run();
}
