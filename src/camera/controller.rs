use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::Skybox;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_water::ImageReformat;
use crate::voxel::world::VoxelWorld;
use crate::voxel::types::Voxel;

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

pub fn spawn_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the skybox cubemap image with cubemap reformat 
    let skybox_image = ImageReformat::cubemap(
        &mut commands,
        &asset_server,
        "textures/table_mountain_2_puresky_4k_cubemap.jpg",
    );
    
    commands.spawn((
        Camera3d::default(),
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
}

pub fn player_camera_system(
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
    mut windows: Query<(&mut Window, &mut CursorOptions)>,
    world: Res<VoxelWorld>,
) {
    let Ok((_window, mut cursor_options)) = windows.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // Toggle cursor lock with Escape
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = !cursor_options.visible;
        cursor_options.grab_mode = if cursor_options.visible {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
    }

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
    let horizontal_velocity = move_dir.normalize_or_zero() * speed;
    camera.velocity.x = horizontal_velocity.x;
    camera.velocity.z = horizontal_velocity.z;

    // Ground check - check voxel below feet
    let feet_pos = transform.translation - Vec3::Y * camera.player_height;
    let ground_check_pos = IVec3::new(
        feet_pos.x.floor() as i32,
        (feet_pos.y - 0.1).floor() as i32,
        feet_pos.z.floor() as i32,
    );

    let was_grounded = camera.grounded;
    camera.grounded = false;

    // Check ground
    if let Some(voxel) = world.get_voxel(ground_check_pos) {
        if voxel.is_solid() {
            let ground_y = ground_check_pos.y as f32 + 1.0 + camera.player_height;
            if transform.translation.y <= ground_y + 0.1 {
                camera.grounded = true;
                if camera.velocity.y < 0.0 {
                    camera.velocity.y = 0.0;
                }
                transform.translation.y = ground_y;
            }
        }
    }

    // Also check slightly ahead for slopes
    let ahead_pos = feet_pos + move_dir.normalize_or_zero() * 0.5;
    let ahead_ground = IVec3::new(
        ahead_pos.x.floor() as i32,
        (ahead_pos.y - 0.1).floor() as i32,
        ahead_pos.z.floor() as i32,
    );

    if let Some(voxel) = world.get_voxel(ahead_ground) {
        if voxel.is_solid() {
            let ground_y = ahead_ground.y as f32 + 1.0 + camera.player_height;
            if transform.translation.y < ground_y && (ground_y - transform.translation.y) < 1.2 {
                // Step up
                transform.translation.y = ground_y;
                camera.grounded = true;
                if camera.velocity.y < 0.0 {
                    camera.velocity.y = 0.0;
                }
            }
        }
    }

    // Jump
    if keys.just_pressed(KeyCode::Space) && camera.grounded {
        camera.velocity.y = camera.jump_force;
        camera.grounded = false;
    }

    // Apply gravity
    if !camera.grounded {
        camera.velocity.y -= camera.gravity * dt;
        // Terminal velocity
        camera.velocity.y = camera.velocity.y.max(-50.0);
    }

    // Move player
    let movement = camera.velocity * dt;
    let new_pos = transform.translation + movement;

    // Collision check for walls (horizontal)
    let head_pos = new_pos;
    let body_pos = new_pos - Vec3::Y * (camera.player_height * 0.5);

    let can_move_x = !is_solid_at(world, Vec3::new(new_pos.x + camera.player_radius * movement.x.signum(), transform.translation.y, transform.translation.z))
                  && !is_solid_at(world, Vec3::new(new_pos.x + camera.player_radius * movement.x.signum(), transform.translation.y - camera.player_height * 0.5, transform.translation.z));

    let can_move_z = !is_solid_at(world, Vec3::new(transform.translation.x, transform.translation.y, new_pos.z + camera.player_radius * movement.z.signum()))
                  && !is_solid_at(world, Vec3::new(transform.translation.x, transform.translation.y - camera.player_height * 0.5, new_pos.z + camera.player_radius * movement.z.signum()));

    if can_move_x {
        transform.translation.x = new_pos.x;
    } else {
        camera.velocity.x = 0.0;
    }

    if can_move_z {
        transform.translation.z = new_pos.z;
    } else {
        camera.velocity.z = 0.0;
    }

    // Vertical movement (already handled ground, now check ceiling)
    if movement.y > 0.0 {
        let ceiling_check = transform.translation + Vec3::Y * 0.5;
        if is_solid_at(world, ceiling_check) {
            camera.velocity.y = 0.0;
        } else {
            transform.translation.y += movement.y;
        }
    } else {
        transform.translation.y += movement.y;
    }

    // Prevent falling through world
    if transform.translation.y < camera.player_height + 1.0 {
        transform.translation.y = camera.player_height + 1.0;
        camera.velocity.y = 0.0;
        camera.grounded = true;
    }
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
