use bevy::prelude::*;
use bevy::render::{RenderApp, RenderSet};

use crate::rendering::capabilities::{
    GraphicsCapabilities, GraphicsDetectionSet, detect_graphics_capabilities,
    sync_capabilities_to_main,
};
use crate::rendering::materials::{setup_triplanar_material, setup_water_material};
use crate::rendering::triplanar_material::TriplanarMaterial;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsCapabilities>()
            // Register TriplanarMaterial as a custom material type
            .add_plugins(MaterialPlugin::<TriplanarMaterial>::default())
            // Register BlockyMaterial
            .add_plugins(MaterialPlugin::<
                crate::rendering::blocky_material::BlockyMaterial,
            >::default())
            .add_systems(
                Startup,
                (
                    crate::rendering::array_loader::start_loading_texture_arrays,
                    setup_water_material,
                    setup_triplanar_material,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    crate::rendering::materials::configure_triplanar_textures,
                    crate::rendering::array_loader::create_texture_array,
                ),
            );

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<GraphicsCapabilities>()
                .add_systems(
                    Render,
                    (
                        detect_graphics_capabilities.in_set(GraphicsDetectionSet),
                        sync_capabilities_to_main
                            .after(GraphicsDetectionSet)
                            .in_set(RenderSet::Cleanup),
                    ),
                );
        } else {
            warn!("Render sub-app not available; graphics capability detection disabled");
        }
    }
}
