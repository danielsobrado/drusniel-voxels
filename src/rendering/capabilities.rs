use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureFormatFeatureFlags};
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo};
use bevy::render::view::ViewTarget;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GraphicsDetectionSet;

/// Runtime information about the active GPU's rendering capabilities.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct GraphicsCapabilities {
    pub adapter_name: Option<String>,
    pub taa_supported: bool,
}

/// Determine whether the current adapter can support temporal anti-aliasing (TAA).
pub fn detect_graphics_capabilities(
    adapter: Option<Res<RenderAdapter>>,
    adapter_info: Option<Res<RenderAdapterInfo>>,
    mut capabilities: ResMut<GraphicsCapabilities>,
) {
    if let (Some(adapter), Some(adapter_info)) = (adapter, adapter_info) {
        let hdr_features = adapter.get_texture_format_features(ViewTarget::TEXTURE_FORMAT_HDR);
        let sdr_features = adapter.get_texture_format_features(TextureFormat::bevy_default());

        let hdr_filterable = hdr_features
            .flags
            .contains(TextureFormatFeatureFlags::FILTERABLE);
        let sdr_filterable = sdr_features
            .flags
            .contains(TextureFormatFeatureFlags::FILTERABLE);

        let new_capabilities = GraphicsCapabilities {
            adapter_name: Some(adapter_info.name.clone()),
            taa_supported: hdr_filterable && sdr_filterable,
        };

        if *capabilities != new_capabilities {
            *capabilities = new_capabilities;

            info!(
                adapter = %adapter_info.name,
                backend = ?adapter_info.backend,
                taa_supported = capabilities.taa_supported,
                hdr_filterable,
                sdr_filterable,
                "Detected GPU capabilities",
            );
        }
    } else {
        warn_once!(
            "Render adapter not available yet; TAA will remain disabled until capabilities are known"
        );
    }
}

/// Copy capabilities from the render world back into the main app.
pub fn sync_capabilities_to_main(
    capabilities: Res<GraphicsCapabilities>,
    mut main_world: ResMut<bevy::render::MainWorld>,
) {
    if !capabilities.is_changed() {
        return;
    }

    let main_world = main_world.as_mut();

    if let Some(mut main_capabilities) = main_world.get_resource_mut::<GraphicsCapabilities>() {
        if *main_capabilities != *capabilities {
            *main_capabilities = capabilities.clone();
        }
    } else {
        main_world.insert_resource(capabilities.clone());
    }
}
