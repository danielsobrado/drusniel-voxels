use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy_mesh::{Indices, PrimitiveTopology};
use crate::voxel::world::VoxelWorld;
use crate::voxel::types::VoxelType;
use super::Health;

/// Component for wolf entities
#[derive(Component)]
pub struct Wolf {
    pub wander_timer: f32,
    pub wander_direction: Vec3,
}

impl Default for Wolf {
    fn default() -> Self {
        Self {
            wander_timer: 0.0,
            wander_direction: Vec3::ZERO,
        }
    }
}

/// Resource to track if wolves have been spawned
#[derive(Resource, Default)]
pub struct WolfSpawned {
    pub spawned: bool,
    pub frame_counter: u32,
}

/// Spawn wolves on the terrain
pub fn spawn_wolves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world: Res<VoxelWorld>,
    mut spawned: ResMut<WolfSpawned>,
) {
    if spawned.spawned {
        return;
    }

    // Wait for chunks to be fully generated
    // Increment frame counter and wait for 60 frames (~1 second)
    spawned.frame_counter += 1;
    if spawned.frame_counter < 60 {
        return;
    }

    // Wait until world has at least one chunk loaded
    if world.get_chunk(IVec3::ZERO).is_none() {
        info!("Waiting for world chunks to load...");
        return;
    }

    spawned.spawned = true;
    info!("Starting wolf spawn process (after {} frames)...", spawned.frame_counter);
    info!("World bounds: 512x64x512 blocks");

    // DEBUG: Check if chunks actually exist
    info!("=== CHUNK DEBUG ===");
    let test_positions = vec![
        IVec3::new(0, 0, 0),
        IVec3::new(1, 0, 1),
        IVec3::new(10, 1, 10),
        IVec3::new(16, 1, 16),
    ];
    
    for chunk_pos in test_positions {
        if world.get_chunk(chunk_pos).is_some() {
            info!("  Chunk {:?} EXISTS", chunk_pos);
            
            // Try to get a voxel from this chunk
            let world_pos = IVec3::new(
                chunk_pos.x * 16 + 8,
                chunk_pos.y * 16 + 20,
                chunk_pos.z * 16 + 8,
            );
            if let Some(voxel) = world.get_voxel(world_pos) {
                info!("    Sample voxel at {:?}: {:?}", world_pos, voxel);
            } else {
                info!("    Sample voxel at {:?}: NONE", world_pos);
            }
        } else {
            info!("  Chunk {:?} MISSING", chunk_pos);
        }
    }
    info!("==================");

    // Create wolf mesh
    let wolf_mesh = meshes.add(create_wolf_mesh());

    // Create wolf material - Realistic gray-brown fur
    let wolf_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.40, 0.35), // Natural gray-brown
        perceptual_roughness: 0.9, // Furry texture
        metallic: 0.0,
        ..default()
    });

    let mut wolf_count = 0;
    let max_wolves = 50;
    let mut positions_checked = 0;
    let mut valid_spawn_locations = 0;
    let mut topsoil_found = 0;
    let mut sand_found = 0;
    let mut air_above_found = 0;

    // Iterate through the actual world bounds
    // Sample every 4 blocks for more coverage
    for x in (0..512).step_by(4) {
        for z in (0..512).step_by(4) {
            if wolf_count >= max_wolves {
                break;
            }

            let world_x = x as i32;
            let world_z = z as i32;
            positions_checked += 1;

            // Use hash to determine spawn chance - 50% probability for balanced distribution
            let hash = simple_hash(world_x * 41, world_z * 43);
            
            if hash > 0.50 { // 50% spawn chance
                // Find surface height - iterate from BOTTOM to TOP
                let mut surface_y = None;
                // let mut surfaces_found = 0;
                
                for y in 1..64 {
                    let pos = IVec3::new(world_x, y, world_z);
                    let above_pos = IVec3::new(world_x, y + 1, world_z);
                    
                    if let (Some(current_voxel), Some(above_voxel)) = (world.get_voxel(pos), world.get_voxel(above_pos)) {
                        // Found a solid block with air above - this is the surface!
                        if (current_voxel == VoxelType::Sand || current_voxel == VoxelType::TopSoil || 
                            current_voxel == VoxelType::SubSoil || current_voxel == VoxelType::Rock) &&
                           above_voxel == VoxelType::Air {
                            surface_y = Some((y, current_voxel));
                            // surfaces_found += 1;
                        }
                    }
                }
                
                
                if let Some((y, voxel)) = surface_y {
                    if voxel == VoxelType::TopSoil {
                        topsoil_found += 1;
                    }
                    if voxel == VoxelType::Sand {
                        sand_found += 1;
                    }
                    
                    air_above_found += 1;
                    valid_spawn_locations += 1;
                    
                    // Spawn wolf
                    let rotation = hash * std::f32::consts::TAU;
                    let spawn_pos = Vec3::new(
                        world_x as f32 + 0.5,
                        y as f32 + 1.5,
                        world_z as f32 + 0.5,
                    );

                    commands.spawn((
                        Mesh3d(wolf_mesh.clone()),
                        MeshMaterial3d(wolf_material.clone()),
                        Transform::from_translation(spawn_pos)
                            .with_rotation(Quat::from_rotation_y(rotation)),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                        Wolf::default(),
                        Health::new(30.0),
                    ));
                    wolf_count += 1;
                }
            }
        }
        if wolf_count >= max_wolves {
            break;
        }
    }

    info!("=== SPAWN STATISTICS ===");
    info!("Positions checked: {}", positions_checked);
    info!("Sand blocks found: {}", sand_found);
    info!("TopSoil blocks found: {}", topsoil_found);
    info!("Air above found: {}", air_above_found);
    info!("Valid spawn locations: {}", valid_spawn_locations);
    info!("✓ Spawned {} wolves in the world", wolf_count);
    
    if wolf_count == 0 {
        warn!("⚠ NO WOLVES SPAWNED! Check terrain generation.");
        warn!("Try flying around the world to see if there's any grass/terrain.");
    }
}

/// Create a simple wolf mesh (box-based model)
fn create_wolf_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Body (main torso) - 1.2 x 0.8 x 0.6
    add_box(&mut positions, &mut normals, &mut indices, 
        Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.2, 0.8, 0.6));

    // Head - 0.6 x 0.6 x 0.6
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(0.7, 0.6, 0.0), Vec3::new(0.6, 0.6, 0.6));

    // Legs (4 legs) - 0.2 x 0.5 x 0.2 each
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(0.4, 0.0, 0.2), Vec3::new(0.2, 0.5, 0.2));
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(0.4, 0.0, -0.2), Vec3::new(0.2, 0.5, 0.2));
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(-0.4, 0.0, 0.2), Vec3::new(0.2, 0.5, 0.2));
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(-0.4, 0.0, -0.2), Vec3::new(0.2, 0.5, 0.2));

    // Tail - 0.6 x 0.2 x 0.2
    add_box(&mut positions, &mut normals, &mut indices,
        Vec3::new(-0.7, 0.7, 0.0), Vec3::new(0.6, 0.2, 0.2));

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Helper to add a box to the mesh
fn add_box(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    size: Vec3,
) {
    let _base_index = positions.len() as u32;
    let half = size / 2.0;

    // 8 vertices of the box
    let verts = [
        center + Vec3::new(-half.x, -half.y, -half.z),
        center + Vec3::new(half.x, -half.y, -half.z),
        center + Vec3::new(half.x, half.y, -half.z),
        center + Vec3::new(-half.x, half.y, -half.z),
        center + Vec3::new(-half.x, -half.y, half.z),
        center + Vec3::new(half.x, -half.y, half.z),
        center + Vec3::new(half.x, half.y, half.z),
        center + Vec3::new(-half.x, half.y, half.z),
    ];

    // 6 faces, each with 4 vertices
    let faces = [
        // Front
        ([verts[0], verts[1], verts[2], verts[3]], [0.0, 0.0, -1.0]),
        // Back
        ([verts[5], verts[4], verts[7], verts[6]], [0.0, 0.0, 1.0]),
        // Left
        ([verts[4], verts[0], verts[3], verts[7]], [-1.0, 0.0, 0.0]),
        // Right
        ([verts[1], verts[5], verts[6], verts[2]], [1.0, 0.0, 0.0]),
        // Bottom
        ([verts[4], verts[5], verts[1], verts[0]], [0.0, -1.0, 0.0]),
        // Top
        ([verts[3], verts[2], verts[6], verts[7]], [0.0, 1.0, 0.0]),
    ];

    for (face_verts, normal) in faces.iter() {
        let start_idx = positions.len() as u32;
        
        for vert in face_verts.iter() {
            positions.push(vert.to_array());
            normals.push(*normal);
        }

        // Two triangles per face
        indices.extend_from_slice(&[
            start_idx, start_idx + 1, start_idx + 2,
            start_idx, start_idx + 2, start_idx + 3,
        ]);
    }
}

/// Simple hash function for deterministic randomness
fn simple_hash(x: i32, z: i32) -> f32 {
    let n = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    let n = n ^ (n >> 16);
    (n as u32 as f32) / (u32::MAX as f32)
}

/// Animate wolves with simple idle behavior
pub fn animate_wolves(
    time: Res<Time>,
    mut wolves: Query<(&mut Wolf, &mut Transform), Without<super::Dead>>,
) {
    let dt = time.delta_secs();

    for (mut wolf, mut transform) in wolves.iter_mut() {
        wolf.wander_timer -= dt;

        // Pick new wander direction every few seconds
        if wolf.wander_timer <= 0.0 {
            wolf.wander_timer = 2.0 + simple_hash(
                (transform.translation.x * 100.0) as i32,
                (transform.translation.z * 100.0) as i32,
            ) * 3.0;

            let angle = simple_hash(
                (time.elapsed_secs() * 100.0) as i32,
                (transform.translation.x * 50.0) as i32,
            ) * std::f32::consts::TAU;

            wolf.wander_direction = Vec3::new(angle.cos(), 0.0, angle.sin());
        }

        // Move slowly in wander direction
        transform.translation += wolf.wander_direction * dt * 0.5;

        // Rotate to face movement direction
        if wolf.wander_direction.length() > 0.01 {
            let target_rotation = Quat::from_rotation_y(
                wolf.wander_direction.z.atan2(wolf.wander_direction.x) - std::f32::consts::FRAC_PI_2
            );
            transform.rotation = transform.rotation.slerp(target_rotation, dt * 2.0);
        }
    }
}
