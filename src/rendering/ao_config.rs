use bevy::prelude::*;
use bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel;
use serde::Deserialize;

#[derive(Resource, Deserialize, Clone)]
pub struct AmbientOcclusionConfig {
    pub ssao: SsaoConfig,
    pub baked: BakedAoConfig,
}

#[derive(Deserialize, Clone)]
pub struct SsaoConfig {
    pub enabled: bool,
    pub quality: String,
    pub constant_object_thickness: f32,
    pub disable_on_integrated_gpu: bool,
}

#[derive(Deserialize, Clone)]
pub struct BakedAoConfig {
    pub enabled: bool,
    pub strength: f32,
    pub corner_darkness: f32,
    pub fix_anisotropy: bool,
}

impl Default for AmbientOcclusionConfig {
    fn default() -> Self {
        Self {
            ssao: SsaoConfig {
                enabled: true,
                quality: "High".to_string(),
                constant_object_thickness: 0.5,
                disable_on_integrated_gpu: true,
            },
            baked: BakedAoConfig {
                enabled: true,
                strength: 0.8,
                corner_darkness: 0.6,
                fix_anisotropy: true,
            },
        }
    }
}

impl SsaoConfig {
    pub fn quality_level(&self) -> ScreenSpaceAmbientOcclusionQualityLevel {
        match self.quality.to_lowercase().as_str() {
            "low" => ScreenSpaceAmbientOcclusionQualityLevel::Low,
            "medium" => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
            "high" => ScreenSpaceAmbientOcclusionQualityLevel::High,
            "ultra" => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
            _ => ScreenSpaceAmbientOcclusionQualityLevel::High,
        }
    }
}

pub fn load_ambient_occlusion_config() -> Result<AmbientOcclusionConfig, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct AoConfigFile {
        ambient_occlusion: AmbientOcclusionConfig,
    }

    let config_str = std::fs::read_to_string("assets/config/ambient_occlusion.yaml")?;
    let config_file: AoConfigFile = serde_yaml::from_str(&config_str)?;
    Ok(config_file.ambient_occlusion)
}
