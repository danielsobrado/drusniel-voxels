use crate::interaction::palette::PlacementPaletteState;
use crate::map::MapState;
use crate::menu::PauseMenuState;
use crate::rendering::capabilities::GraphicsCapabilities;
use crate::rendering::ray_tracing::RayTracingSettings;
use crate::voxel::types::Voxel;
use crate::voxel::world::VoxelWorld;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::core_pipeline::Skybox;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Msaa;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_water::ImageReformat;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraMode {
    Fly,
    Walk,
}

#[derive(Component)]
pub struct PlayerCamera {
    // Shared settings
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub mode: CameraMode,

    // Fly mode settings
    pub fly_speed: f32,

    // Walk mode settings
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub velocity: Vec3,
    pub grounded: bool,
    pub player_height: f32,
    pub player_radius: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
            mode: CameraMode::Walk, // Start in walk mode

            fly_speed: 40.0,

            walk_speed: 8.0,
            run_speed: 16.0,
            jump_force: 12.0,
            gravity: 30.0,
            velocity: Vec3::ZERO,
            grounded: false,
            player_height: 1.8,
            player_radius: 0.3,
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    capabilities: Res<GraphicsCapabilities>,
    ray_tracing: Res<RayTracingSettings>,
) {
    // Load the skybox cubemap image with cubemap reformat
    let skybox_image = ImageReformat::cubemap(
        &mut commands,
        &asset_server,
        "textures/table_mountain_2_puresky_4k_cubemap.jpg",
    );

    let mut camera = commands.spawn((
        Camera3d::default(),
        Bloom {
            intensity: 0.06, // Subtle glow on bright highlights
            ..default()
        },
        Transform::from_xyz(256.0, 50.0, 256.0).looking_at(Vec3::new(200.0, 30.0, 200.0), Vec3::Y),
        PlayerCamera::default(),
        // Tonemapping for better HDR look
        Tonemapping::AcesFitted,
        // Skybox with the cubemap
        Skybox {
            image: skybox_image,
            brightness: 1500.0,
            rotation: Quat::IDENTITY,
        },
        // Atmospheric fog with warm/pink horizon tint
        DistanceFog {
            color: Color::srgba(0.7, 0.8, 0.95, 1.0), // Soft blue-gray base
            directional_light_color: Color::srgba(1.0, 0.85, 0.7, 1.0), // Warm golden sun scatter
            directional_light_exponent: 20.0,
            falloff: FogFalloff::ExponentialSquared { density: 0.0015 },
        },
    ));

    if capabilities.taa_supported {
        camera.insert(TemporalAntiAliasing::default());
        commands.insert_resource(Msaa::Off);
    }

    if ray_tracing.enabled && capabilities.ray_tracing_supported {
        camera.insert(ScreenSpaceReflections::default());
    }
}

pub fn update_ray_tracing_on_camera(
    capabilities: Res<GraphicsCapabilities>,
    settings: Res<RayTracingSettings>,
    mut commands: Commands,
    mut cameras: Query<(Entity, Option<&ScreenSpaceReflections>), With<PlayerCamera>>,
) {
    if !(settings.is_changed() || capabilities.is_changed()) {
        return;
    }

    let should_enable = settings.enabled && capabilities.ray_tracing_supported;

    for (entity, current) in cameras.iter_mut() {
        match (should_enable, current.is_some()) {
            (true, false) => {
                commands
                    .entity(entity)
                    .insert(ScreenSpaceReflections::default());
            }
            (false, true) => {
                commands.entity(entity).remove::<ScreenSpaceReflections>();
            }
            _ => {}
        }
    }
}

pub fn player_camera_system(
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
    mut windows: Query<(&mut Window, &mut CursorOptions)>,
    world: Res<VoxelWorld>,
    pause_menu: Res<PauseMenuState>,
    palette: Res<PlacementPaletteState>,
    map_state: Res<MapState>,
) {
    let Ok((_window, mut cursor_options)) = windows.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    if pause_menu.open || palette.open || map_state.open {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
        return;
    }

    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;

    for (mut transform, mut camera) in query.iter_mut() {
        // Toggle between fly and walk mode with Tab
        if keys.just_pressed(KeyCode::Tab) {
            camera.mode = match camera.mode {
                CameraMode::Fly => CameraMode::Walk,
                CameraMode::Walk => CameraMode::Fly,
            };
            camera.velocity = Vec3::ZERO;
            // Log mode change
            match camera.mode {
                CameraMode::Fly => info!("Switched to FLY mode"),
                CameraMode::Walk => info!("Switched to WALK mode"),
            }
        }

        // Reset position with R
        if keys.just_pressed(KeyCode::KeyR) {
            camera.yaw = -2.35;
            camera.pitch = -0.4;
            camera.velocity = Vec3::ZERO;
            *transform = Transform::from_xyz(256.0, 50.0, 256.0)
                .looking_at(Vec3::new(200.0, 30.0, 200.0), Vec3::Y);
        }

        if cursor_options.visible {
            return;
        }

        // Mouse look (both modes)
        for ev in mouse_motion.read() {
            camera.yaw -= ev.delta.x * camera.sensitivity;
            camera.pitch -= ev.delta.y * camera.sensitivity;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        }

        transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);

        // Movement based on mode
        match camera.mode {
            CameraMode::Fly => {
                fly_movement(&mut transform, &camera, &keys, dt);
            }
            CameraMode::Walk => {
                walk_movement(&mut transform, &mut camera, &keys, dt, &world);
            }
        }
    }
}

fn fly_movement(
    transform: &mut Transform,
    camera: &PlayerCamera,
    keys: &Res<ButtonInput<KeyCode>>,
    dt: f32,
) {
    let mut velocity = Vec3::ZERO;
    let local_z = transform.local_z();
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z).normalize_or_zero();
    let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize_or_zero();

    if keys.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keys.pressed(KeyCode::Space) {
        velocity += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        velocity -= Vec3::Y;
    }

    let speed = if keys.pressed(KeyCode::ControlLeft) {
        camera.fly_speed * 3.0 // Turbo fly
    } else {
        camera.fly_speed
    };

    transform.translation += velocity.normalize_or_zero() * speed * dt;
}

fn walk_movement(
    transform: &mut Transform,
    camera: &mut PlayerCamera,
    keys: &Res<ButtonInput<KeyCode>>,
    dt: f32,
    world: &VoxelWorld,
) {
    let local_z = transform.local_z();
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z).normalize_or_zero();
    let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize_or_zero();

    // Horizontal input
    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir += right;
    }

    // Determine speed (run with shift)
    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        camera.run_speed
    } else {
        camera.walk_speed
    };

    // Set horizontal velocity
    let mut horizontal_velocity = move_dir.normalize_or_zero() * speed;

    // Friction / damping if no input? (Optional, currently instant stop)
    if move_dir == Vec3::ZERO {
        horizontal_velocity = Vec3::ZERO;
    }

    camera.velocity.x = horizontal_velocity.x;
    camera.velocity.z = horizontal_velocity.z;

    // Ground check with smooth terrain height
    let terrain_height = get_terrain_height(world, transform.translation);

    // Check if we are grounded
    // We are grounded if our feet are slightly below or just above the terrain height
    if let Some(h) = terrain_height {
        let feet_y = transform.translation.y - camera.player_height;
        if feet_y <= h + 0.2 && camera.velocity.y <= 0.0 {
            camera.grounded = true;
            camera.velocity.y = 0.0;
            // Snap gently or hard? Hard snap prevents jitter.
            // But we must snap to h + player_height
            transform.translation.y = h + camera.player_height;
        } else {
            camera.grounded = false;
        }
    } else {
        camera.grounded = false;
    }

    // Jump
    if keys.just_pressed(KeyCode::Space) && camera.grounded {
        camera.velocity.y = camera.jump_force;
        camera.grounded = false;
        transform.translation.y += 0.1; // Lift off ground immediately
    }

    // Apply gravity
    if !camera.grounded {
        camera.velocity.y -= camera.gravity * dt;
        camera.velocity.y = camera.velocity.y.max(-50.0);
    }

    // Calculate intended movement
    let movement = camera.velocity * dt;

    // Collision Detection & Step-Up Logic
    let mut new_pos = transform.translation;
    let step_height = 1.1; // Max height to automatically step up

    // Separate axes to allow sliding against walls

    // --- X Axis ---
    let target_x = new_pos.x + movement.x;
    // Check collision at feet and head
    // Note: We use a slightly smaller radius for collision to allow getting close to walls
    let col_radius = camera.player_radius * 0.9;

    let can_move_x_flat = !check_collision(
        world,
        Vec3::new(target_x, new_pos.y, new_pos.z),
        col_radius,
        camera.player_height,
    );

    if can_move_x_flat {
        new_pos.x = target_x;
    } else if camera.grounded {
        // Try step up
        let can_step_x = !check_collision(
            world,
            Vec3::new(target_x, new_pos.y + step_height, new_pos.z),
            col_radius,
            camera.player_height,
        );
        if can_step_x {
            // We can move X if we go up.
            // Check the terrain height at the target X to snap exactly there
            if let Some(h) = get_terrain_height(
                world,
                Vec3::new(target_x, new_pos.y + step_height, new_pos.z),
            ) {
                if h - (new_pos.y - camera.player_height) <= step_height + 0.1 {
                    new_pos.x = target_x;
                    new_pos.y = h + camera.player_height; // Teleport up to the step
                }
            }
        } else {
            camera.velocity.x = 0.0;
        }
    } else {
        camera.velocity.x = 0.0;
    }

    // --- Z Axis ---
    let target_z = new_pos.z + movement.z;
    let can_move_z_flat = !check_collision(
        world,
        Vec3::new(new_pos.x, new_pos.y, target_z),
        col_radius,
        camera.player_height,
    );

    if can_move_z_flat {
        new_pos.z = target_z;
    } else if camera.grounded {
        // Try step up
        let can_step_z = !check_collision(
            world,
            Vec3::new(new_pos.x, new_pos.y + step_height, target_z),
            col_radius,
            camera.player_height,
        );
        if can_step_z {
            if let Some(h) = get_terrain_height(
                world,
                Vec3::new(new_pos.x, new_pos.y + step_height, target_z),
            ) {
                if h - (new_pos.y - camera.player_height) <= step_height + 0.1 {
                    new_pos.z = target_z;
                    // Only update Y if it's higher than current (in case we already stepped up in X)
                    let target_y = h + camera.player_height;
                    if target_y > new_pos.y {
                        new_pos.y = target_y;
                    }
                }
            }
        } else {
            camera.velocity.z = 0.0;
        }
    } else {
        camera.velocity.z = 0.0;
    }

    // --- Y Axis (Vertical Movement) ---
    // If not grounded or jumping, apply vertical velocity
    // (If we stepped up, new_pos.y is already updated)
    if !camera.grounded {
        new_pos.y += movement.y;
    }

    // Ceiling check
    if movement.y > 0.0 {
        if check_collision(
            world,
            new_pos + Vec3::Y * 0.1,
            col_radius,
            camera.player_height,
        ) {
            camera.velocity.y = 0.0;
            new_pos.y = transform.translation.y; // Cancel updwards move
        }
    }

    // Apply final position
    transform.translation = new_pos;

    // Safety: Prevent falling through world
    if transform.translation.y < camera.player_height + 1.0 {
        transform.translation.y = camera.player_height + 1.0;
        camera.velocity.y = 0.0;
        camera.grounded = true;
    }
}

/// Returns true if the player cylinder at `pos` collides with any solid voxel
fn check_collision(world: &VoxelWorld, pos: Vec3, radius: f32, height: f32) -> bool {
    // Check feet, waist, head
    // A simple 3-point check or 4 corners check
    // For cylinder, we check boundaries: x+r, x-r, z+r, z-r

    let y_min = pos.y - height;
    let y_max = pos.y;

    // Discrete steps for Y (feet, mid, head)
    let y_steps = [y_min + 0.1, y_min + height * 0.5, y_max - 0.1];

    for y in y_steps {
        // Check center
        if is_solid_at(world, Vec3::new(pos.x, y, pos.z)) {
            return true;
        }
        // Check cardinal edges
        if is_solid_at(world, Vec3::new(pos.x + radius, y, pos.z)) {
            return true;
        }
        if is_solid_at(world, Vec3::new(pos.x - radius, y, pos.z)) {
            return true;
        }
        if is_solid_at(world, Vec3::new(pos.x, y, pos.z + radius)) {
            return true;
        }
        if is_solid_at(world, Vec3::new(pos.x, y, pos.z - radius)) {
            return true;
        }
    }
    false
}

fn is_solid_at(world: &VoxelWorld, pos: Vec3) -> bool {
    let block_pos = IVec3::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );

    if let Some(voxel) = world.get_voxel(block_pos) {
        voxel.is_solid()
    } else {
        false
    }
}

/// Calculate smoothed terrain height at a specific world position
/// Uses bilinear interpolation of the 4 nearest column heights
fn get_terrain_height(world: &VoxelWorld, pos: Vec3) -> Option<f32> {
    let x = pos.x;
    let z = pos.z;
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let fx = x - x0 as f32;
    let fz = z - z0 as f32;

    // We scan for ground near the player's feet
    let scan_start_y = pos.y;

    let h00 = get_column_height(world, x0, z0, scan_start_y);
    let h10 = get_column_height(world, x1, z0, scan_start_y);
    let h01 = get_column_height(world, x0, z1, scan_start_y);
    let h11 = get_column_height(world, x1, z1, scan_start_y);

    // If all are None, we act as void
    if h00.is_none() && h10.is_none() && h01.is_none() && h11.is_none() {
        return None;
    }

    // Fill missing values with nearest valid one (or current y - something)
    // to handle edges of cliffs smoothly(ish)
    let fallback = pos.y - 100.0;
    let v00 = h00.unwrap_or(h10.or(h01).or(h11).unwrap_or(fallback));
    let v10 = h10.unwrap_or(v00);
    let v01 = h01.unwrap_or(v00);
    let v11 = h11.unwrap_or(v00);

    let h0 = v00 * (1.0 - fx) + v10 * fx;
    let h1 = v01 * (1.0 - fx) + v11 * fx;

    Some(h0 * (1.0 - fz) + h1 * fz)
}

/// Find the height of the highest solid block in a column, starting scan from start_y downwards
fn get_column_height(world: &VoxelWorld, x: i32, z: i32, start_y: f32) -> Option<f32> {
    let mut y = (start_y + 1.0).ceil() as i32;
    let min_y = y - 6; // Scan down 6 blocks max for ground

    while y >= min_y {
        if let Some(voxel) = world.get_voxel(IVec3::new(x, y, z)) {
            if voxel.is_solid() {
                // Return Y + 1.0 (top of block)
                // For surface nets, visual surface is often a bit smoothed, but
                // walking on the block top (1.0) is the safe standard.
                return Some(y as f32 + 1.0);
            }
        }
        y -= 1;
    }
    None
}
