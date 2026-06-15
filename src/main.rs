use bevy::prelude::*;
use voxel_builder::voxel::plugin::VoxelPlugin;
use voxel_builder::rendering::plugin::RenderingPlugin;
use voxel_builder::camera::plugin::CameraPlugin;
use voxel_builder::interaction::InteractionPlugin;
use voxel_builder::viewmodel::PickaxePlugin;
use voxel_builder::vegetation::VegetationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Pixel art look
        .add_plugins(VoxelPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(PickaxePlugin)
        .add_plugins(VegetationPlugin)
        // Warm sky with slight pink/purple tint (Valheim style)
        .insert_resource(ClearColor(Color::srgb(0.65, 0.75, 0.9)))
        // Warm ambient light for that golden hour feel
        .insert_resource(AmbientLight {
            color: Color::srgb(0.9, 0.85, 0.75),
            brightness: 600.0,
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, setup_environment)
        .add_systems(Update, animate_sun)
        .run();
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct SunVisual;

fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun - Directional light with warm color and shadows
    let sun_direction = Vec3::new(-0.4, -0.6, -0.5).normalize();

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85), // Warm sunlight
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ZERO).looking_to(sun_direction, Vec3::Y),
        Sun,
    ));

    // Visual sun sphere in the sky
    let sun_position = Vec3::new(300.0, 200.0, 250.0);
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.7),
            emissive: LinearRgba::new(5.0, 4.5, 3.0, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(sun_position),
        SunVisual,
    ));

    // Sun glow (larger transparent sphere)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(35.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.9, 0.6, 0.15),
            emissive: LinearRgba::new(1.0, 0.9, 0.5, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(sun_position),
        SunVisual,
    ));

    // Horizon haze and clouds disabled for debugging
}

// Animate sun position for day/night cycle (slow)
fn animate_sun(
    time: Res<Time>,
    mut sun_query: Query<&mut Transform, With<Sun>>,
    mut sun_visual_query: Query<&mut Transform, (With<SunVisual>, Without<Sun>)>,
) {
    let cycle_speed = 0.005; // Slower day cycle
    let angle = time.elapsed_secs() * cycle_speed;

    // Sun direction rotates around the world
    let sun_dir = Vec3::new(
        angle.cos() * 0.5,
        -0.6 - angle.sin().abs() * 0.3, // Sun higher at noon
        angle.sin() * 0.5,
    ).normalize();

    for mut transform in sun_query.iter_mut() {
        *transform = Transform::from_translation(Vec3::ZERO).looking_to(sun_dir, Vec3::Y);
    }

    // Move visual sun sphere to match
    let sun_distance = 300.0;
    let sun_pos = Vec3::new(256.0, 50.0, 256.0) - sun_dir * sun_distance;

    for mut transform in sun_visual_query.iter_mut() {
        transform.translation = sun_pos;
    }
}
