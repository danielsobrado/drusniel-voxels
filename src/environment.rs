use bevy::prelude::*;
use bevy_water::*;

/// Water level constant - matches terrain generation
pub const SEA_LEVEL: f32 = 18.0;

/// Settings that drive the sky and sun animation
#[derive(Resource)]
pub struct AtmosphereSettings {
    /// Length of a full day/night cycle in seconds
    pub day_length: f32,
    /// Current time within the cycle
    pub time: f32,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            day_length: 1800.0, // 30 minutes for a full cycle
            // Start during the day (slightly past sunrise)
            time: 1800.0 * 0.25,
        }
    }
}

#[derive(Component)]
pub struct Sun;

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AtmosphereSettings::default())
            // Soft initial sky tint
            .insert_resource(ClearColor(Color::srgba(0.50, 0.64, 0.84, 1.0)))
            // bevy_water for dynamic ocean waves
            .insert_resource(WaterSettings {
                height: SEA_LEVEL,
                amplitude: 0.5,
                clarity: 0.4,
                deep_color: Color::srgba(0.05, 0.15, 0.35, 0.95).into(),
                shallow_color: Color::srgba(0.2, 0.5, 0.7, 0.85).into(),
                edge_color: Color::srgba(0.6, 0.8, 0.9, 0.7).into(),
                ..default()
            })
            .add_plugins((WaterPlugin, ImageUtilsPlugin))
            .add_systems(Startup, setup_atmosphere)
            .add_systems(Update, animate_atmosphere);
    }
}

fn setup_atmosphere(
    mut commands: Commands,
) {
    // Sun directional light with extended shadow range
    commands.spawn((
        DirectionalLight {
            color: Color::srgba(1.0, 0.93, 0.82, 1.0),
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(-0.3, -1.0, -0.2).normalize(), Vec3::Y),
        Sun,
    ));
}

fn animate_atmosphere(
    time: Res<Time>,
    mut settings: ResMut<AtmosphereSettings>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut ambient: ResMut<AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
) {
    // Advance time
    settings.time = (settings.time + time.delta_secs()) % settings.day_length;
    let phase = settings.time / settings.day_length; // 0..1

    // Sun position: overhead at noon, gentle arc for sunrise/sunset
    let theta = phase * std::f32::consts::TAU;
    let altitude = theta.sin(); // 1 at noon, -1 at midnight
    let azimuth = theta.cos();  // horizontal movement
    let sun_dir = Vec3::new(azimuth * 0.35, -altitude.max(0.2), 0.45).normalize_or_zero();

    // Lighting strength based on altitude
    let day_factor = saturate((altitude + 0.4) * 1.0).max(0.65); // keep a higher floor for nights
    let horizon_warmth = (1.0 - altitude.abs()).clamp(0.0, 1.0);
    // Warm key light + cooler ambient fill for Valheim-like grading
    let sun_strength = lerp(4000.0, 10_000.0, day_factor);
    let ambient_strength = lerp(2800.0, 6000.0, day_factor);
    let sun_tint = (Vec3::new(1.0, 0.78, 0.62)
        .lerp(Vec3::new(1.0, 0.93, 0.80), day_factor)
        + Vec3::splat(horizon_warmth * 0.04))
        .min(Vec3::splat(1.0));
    let ambient_tint = Vec3::new(0.10, 0.16, 0.26)
        .lerp(Vec3::new(0.24, 0.36, 0.52), day_factor);
    let sky_tint = Vec3::new(0.14, 0.20, 0.34)
        .lerp(Vec3::new(0.50, 0.68, 0.88), day_factor);

    // Update sun
    if let Ok((mut transform, mut light)) = sun_query.single_mut() {
        transform.look_to(sun_dir, Vec3::Y);
        light.illuminance = sun_strength;
        light.color = Color::srgba(sun_tint.x, sun_tint.y, sun_tint.z, 1.0);
    }

    // Update ambient and sky tint
    ambient.brightness = ambient_strength;
    ambient.color = Color::srgba(ambient_tint.x, ambient_tint.y, ambient_tint.z, 1.0);
    clear_color.0 = Color::srgba(sky_tint.x, sky_tint.y, sky_tint.z, 1.0);

}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
