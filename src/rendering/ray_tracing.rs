use bevy::prelude::*;

/// Runtime toggle that mirrors the ray tracing preference from the settings menu.
#[derive(Resource, Clone, Debug)]
pub struct RayTracingSettings {
    pub enabled: bool,
}

impl Default for RayTracingSettings {
    fn default() -> Self {
        Self { enabled: false }
    }
}
