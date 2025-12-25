use bevy::prelude::*;

use crate::rendering::building_material::BuildingMaterial;
use crate::rendering::capabilities::{
    GraphicsCapabilities, GraphicsDetectionSet, detect_graphics_capabilities,
};
use crate::rendering::materials::{setup_triplanar_material, setup_water_material, setup_building_material, setup_props_material};
use crate::rendering::props_material::PropsMaterial;
use crate::rendering::ray_tracing::RayTracingSettings;
use crate::rendering::ssao::SsaoPlugin;
use crate::rendering::triplanar_material::TriplanarMaterial;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsCapabilities>()
            .init_resource::<RayTracingSettings>()
            .add_systems(
                Startup,
                detect_graphics_capabilities.in_set(GraphicsDetectionSet),
            )
            .add_plugins(SsaoPlugin)
            .add_plugins(crate::rendering::cinematic::CinematicPlugin)
            .add_plugins(crate::rendering::photo_mode::PhotoModePlugin)
            // ScreenSpaceReflectionsPlugin is already included by DefaultPlugins via PbrPlugin.
            // Register TriplanarMaterial as a custom material type
            .add_plugins(MaterialPlugin::<TriplanarMaterial>::default())
            // Register BlockyMaterial
            .add_plugins(MaterialPlugin::<
                crate::rendering::blocky_material::BlockyMaterial,
            >::default())
            // Register BuildingMaterial (Full PBR for RTX 40xx)
            .add_plugins(MaterialPlugin::<BuildingMaterial>::default())
            // Register PropsMaterial (Medium PBR)
            .add_plugins(MaterialPlugin::<PropsMaterial>::default())
            .add_systems(
                Startup,
                (
                    crate::rendering::array_loader::start_loading_texture_arrays,
                    setup_water_material,
                    setup_triplanar_material,
                    setup_building_material,
                    setup_props_material,
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
    }
}
