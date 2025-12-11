use bevy::prelude::*;
use crate::voxel::world::VoxelWorld;
use crate::voxel::types::{VoxelType, Voxel};
use super::Health;

/// Component for rabbit entities
#[derive(Component)]
pub struct Rabbit {
    pub hop_timer: f32,
    pub hop_direction: Vec3,
    pub is_hopping: bool,
    pub hop_progress: f32,
}

impl Default for Rabbit {
    fn default() -> Self {
        Self {
            hop_timer: 1.0,
            hop_direction: Vec3::ZERO,
            is_hopping: false,
            hop_progress: 0.0,
        }
    }
}

/// Resource to track if rabbits have been spawned
#[derive(Resource, Default)]
pub struct RabbitSpawned {
    pub spawned: bool,
    pub frame_counter: u32,
}

/// Resource to hold the loaded rabbit scene handle
#[derive(Resource)]
pub struct RabbitSceneHandle(pub Handle<Scene>);

/// Setup system to load the rabbit GLTF model
pub fn setup_rabbit_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let rabbit_scene: Handle<Scene> = asset_server.load("models/white_rabbit/scene.gltf#Scene0");
    commands.insert_resource(RabbitSceneHandle(rabbit_scene));
}

/// Spawn rabbits on the terrain
pub fn spawn_rabbits(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    mut spawned: ResMut<RabbitSpawned>,
    rabbit_scene: Option<Res<RabbitSceneHandle>>,
) {
    if spawned.spawned {
        return;
    }

    // Wait for rabbit scene to be loaded
    let Some(rabbit_scene) = rabbit_scene else {
        return;
    };

    // Wait for chunks to be fully generated
    spawned.frame_counter += 1;
    if spawned.frame_counter < 120 {
        return;
    }

    // Wait until world has at least one chunk loaded
    if world.get_chunk(IVec3::ZERO).is_none() {
        info!("Waiting for world chunks to load for rabbits...");
        return;
    }

    spawned.spawned = true;
    info!("Starting rabbit spawn process (after {} frames)...", spawned.frame_counter);

    let mut rabbit_count = 0;
    let max_rabbits = 20;
    let mut positions_checked = 0;
    let mut surfaces_found = 0;

    // Sample test positions to verify get_voxel works
    let test_pos = IVec3::new(24, 20, 24);
    if let Some(voxel) = world.get_voxel(test_pos) {
        info!("  Test voxel at {:?}: {:?}", test_pos, voxel);
    } else {
        info!("  Test voxel at {:?}: NONE", test_pos);
    }

    // Distribute spawns across the map with wider spacing
    // World is 512x512, so step by 25 blocks gives ~400 positions
    for x in (10..500).step_by(25) {
        for z in (10..500).step_by(25) {
            if rabbit_count >= max_rabbits {
                break;
            }

            let world_x = x as i32;
            let world_z = z as i32;
            positions_checked += 1;

            // Find surface height - search from top down
            for y in (1..64).rev() {
                let pos = IVec3::new(world_x, y, world_z);
                let above_pos = IVec3::new(world_x, y + 1, world_z);

                if let (Some(current_voxel), Some(above_voxel)) = (world.get_voxel(pos), world.get_voxel(above_pos)) {
                    // Found a solid block with air above - this is the surface!
                    if current_voxel.is_solid() && !current_voxel.is_liquid() && 
                       above_voxel == VoxelType::Air {
                        surfaces_found += 1;
                        
                        // Spawn rabbit at this valid surface
                        let hash = simple_hash(world_x * 73, world_z * 67);
                        // Removed hash filter - spawn on all valid surfaces until we reach max_rabbits
                            // Spawn rabbit
                            let rotation = hash * std::f32::consts::TAU;
                            let spawn_pos = Vec3::new(
                                world_x as f32 + 0.5,
                                y as f32 + 1.0,  // Slightly above ground
                                world_z as f32 + 0.5,
                            );

                            info!("  Spawning rabbit #{} at {:?} on {:?}", rabbit_count + 1, spawn_pos, current_voxel);

                            commands.spawn((
                                SceneRoot(rabbit_scene.0.clone()),
                                Transform::from_translation(spawn_pos)
                                    .with_rotation(Quat::from_rotation_y(rotation))
                                    .with_scale(Vec3::splat(0.5)),  // Scale down the model
                                GlobalTransform::default(),
                                Visibility::Visible,
                                InheritedVisibility::VISIBLE,
                                ViewVisibility::default(),
                                Rabbit::default(),
                                Health::new(10.0),
                            ));
                            rabbit_count += 1;
                        
                        break;  // Found surface for this column, move to next
                    }
                }
            }
        }
        if rabbit_count >= max_rabbits {
            break;
        }
    }

    info!("=== RABBIT SPAWN STATISTICS ===");
    info!("Positions checked: {}", positions_checked);
    info!("Surfaces found: {}", surfaces_found);
    info!("✓ Spawned {} rabbits in the world", rabbit_count);

    if rabbit_count == 0 {
        warn!("⚠ NO RABBITS SPAWNED! Check terrain generation.");
    }
}

/// Simple hash function for deterministic randomness
fn simple_hash(x: i32, z: i32) -> f32 {
    let n = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    let n = n ^ (n >> 16);
    (n as u32 as f32) / (u32::MAX as f32)
}

/// Animate rabbits with hopping behavior
pub fn animate_rabbits(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    mut rabbits: Query<(&mut Rabbit, &mut Transform), Without<super::Dead>>,
) {
    let dt = time.delta_secs();

    for (mut rabbit, mut transform) in rabbits.iter_mut() {
        rabbit.hop_timer -= dt;

        // Apply horizontal movement if hopping
        if rabbit.is_hopping {
            rabbit.hop_progress += dt * 3.0; // Hop speed

            if rabbit.hop_progress >= 1.0 {
                rabbit.is_hopping = false;
                rabbit.hop_progress = 0.0;
                rabbit.hop_timer = 0.5 + simple_hash(
                    (transform.translation.x * 100.0) as i32,
                    (transform.translation.z * 100.0) as i32,
                ) * 2.0;
            } else {
                let forward_motion = rabbit.hop_direction * dt * 2.0;
                transform.translation.x += forward_motion.x;
                transform.translation.z += forward_motion.z;
            }
        } else if rabbit.hop_timer <= 0.0 {
            // Start new hop
            rabbit.is_hopping = true;
            rabbit.hop_progress = 0.0;

            let angle = simple_hash(
                (time.elapsed_secs() * 100.0) as i32,
                (transform.translation.x * 50.0) as i32,
            ) * std::f32::consts::TAU;

            rabbit.hop_direction = Vec3::new(angle.cos(), 0.0, angle.sin());

            // Rotate to face hop direction
            if rabbit.hop_direction.length() > 0.01 {
                let target_rotation = Quat::from_rotation_y(
                    rabbit.hop_direction.z.atan2(rabbit.hop_direction.x) - std::f32::consts::FRAC_PI_2
                );
                transform.rotation = target_rotation;
            }
        }

        // Apply Gravity / Snap to Floor
        let x = transform.translation.x.floor() as i32;
        let z = transform.translation.z.floor() as i32;
        let start_y = (transform.translation.y + 1.0).floor() as i32; // Search from slightly above
        
        // Find ground
        let mut ground_y = 0.0; // Fallback to 0 if no ground found (void)
        for y in (0..=start_y).rev() {
            if let Some(voxel) = world.get_voxel(IVec3::new(x, y, z)) {
                if voxel.is_solid() {
                    ground_y = y as f32 + 1.0;
                    break;
                }
            }
        }

        // Calculate vertical position (Ground + Hop Arc)
        let hop_height = if rabbit.is_hopping {
            (rabbit.hop_progress * std::f32::consts::PI).sin() * 0.5
        } else {
            0.0
        };

        transform.translation.y = ground_y + hop_height;
    }
}
