use bevy::prelude::*;
use bevy_mesh::VertexAttributeValues;
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

#[derive(Resource)]
pub struct RabbitHandles {
    pub scene: Handle<Scene>,
    pub texture: Handle<Image>,
}

pub fn setup_rabbit_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load("models/white_rabbit/scene.gltf#Scene0");
    let texture = asset_server.load("models/white_rabbit/textures/Material_0_BaseColor.jpeg");
    commands.insert_resource(RabbitHandles { scene, texture });
}

/// Marker component to track rabbits that have had their textures fixed
#[derive(Component)]
pub struct RabbitTextureFixed;

pub fn fix_rabbit_textures(
    mut commands: Commands,
    rabbit_query: Query<(Entity, Option<&Children>), (With<Rabbit>, Without<RabbitTextureFixed>)>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mesh_query: Query<&Mesh3d>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>, // CHANGED to ResMut
    images: Res<Assets<Image>>,
    handles: Option<Res<RabbitHandles>>,
) {
    let Some(handles) = handles else { return };

    // Check if texture is loaded
    let texture_state = images.get(&handles.texture).is_some();
    if !texture_state {
        // info_once!("Rabbit texture is NOT fully loaded yet.");
    }

    for (rabbit_entity, maybe_children) in rabbit_query.iter() {
        // Scene hasn't spawned children yet, skip for now
        let Some(children) = maybe_children else { continue };

        let mut stack: Vec<Entity> = children.iter().collect();
        let mut fixed_any = false;

        while let Some(curr) = stack.pop() {
            // let mut mesh_uv_missing = false;

            // Check mesh for UVs and Generate if missing
            if let Ok(mesh_handle) = mesh_query.get(curr) {
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    if !mesh.contains_attribute(Mesh::ATTRIBUTE_UV_0) {
                        // mesh_uv_missing = true;
                        info!("Mesh {:?} missing UVs. Generating procedural UVs...", mesh_handle.id());
                        
                        // Generate simple planar UVs (XZ plane)
                        if let Some(VertexAttributeValues::Float32x3(positions)) =
                            mesh.attribute(Mesh::ATTRIBUTE_POSITION).cloned()
                        {
                            let uvs: Vec<[f32; 2]> = positions.iter()
                                .map(|pos| [pos[0], pos[2]]) // Planar mapping X, Z
                                .collect();
                            
                            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                            info!("Inserted generated UVs into mesh.");
                        }
                    }
                }
            }

            // Check for material
            if let Ok(mat_handle) = material_query.get(curr) {
                let _handle_id = mat_handle.id();
                
                // Strategy Switch: Create a NEW material and replace the component
                let new_material = StandardMaterial {
                    base_color_texture: Some(handles.texture.clone()),
                    base_color: Color::WHITE, // Set back to WHITE to see texture
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    alpha_mode: AlphaMode::Opaque,
                    ..default()
                };

                let new_handle = materials.add(new_material);
                
                if !fixed_any {
                    info!("Replacing material for rabbit entity {:?}.", curr);
                    
                    // Inspect and Fix UVs
                     if let Ok(mesh_handle) = mesh_query.get(curr) {
                        if let Some(mesh) = meshes.get_mut(mesh_handle) {
                             let mut needs_uv_fix = false;

                             // Check if UVs are missing OR degenerate (all zeros)
                             if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                                let non_zero_count = uvs.iter().filter(|uv| uv[0] != 0.0 || uv[1] != 0.0).count();
                                info!("Mesh UV check: {} total, {} non-zero.", uvs.len(), non_zero_count);
                                
                                // If fewer than 10 vertices have non-zero UVs, it's definitely broken
                                if non_zero_count < 10 { 
                                    needs_uv_fix = true;
                                    info!("Detected degenerate/zero UVs. Forcing regeneration.");
                                }
                             } else {
                                 needs_uv_fix = true; // Missing attribute entirely
                                 info!("Missing UV attribute. Forcing regeneration.");
                             }

                             // Generate Cylindrical UVs (Better for animal bodies than planar)
                             if needs_uv_fix {
                                if let Some(VertexAttributeValues::Float32x3(positions)) = 
                                    mesh.attribute(Mesh::ATTRIBUTE_POSITION) 
                                {
                                    // 1. Calculate Bounds
                                    let mut min_x = f32::MAX;
                                    let mut max_x = f32::MIN;
                                    let mut min_y = f32::MAX;
                                    let mut max_y = f32::MIN;
                                    let mut min_z = f32::MAX;
                                    let mut max_z = f32::MIN;
                                    
                                    for pos in positions.iter() {
                                        min_x = min_x.min(pos[0]);
                                        max_x = max_x.max(pos[0]);
                                        min_y = min_y.min(pos[1]);
                                        max_y = max_y.max(pos[1]);
                                        min_z = min_z.min(pos[2]);
                                        max_z = max_z.max(pos[2]);
                                    }

                                    // Use Bounding Box Center (better for geometry mapping than centroid)
                                    let center_x = (min_x + max_x) / 2.0;
                                    let center_z = (min_z + max_z) / 2.0;
                                    let center_y = (min_y + max_y) / 2.0;

                                    let _height = max_y - min_y;
                                    // Use Box Radius for spherical approximation
                                    let _radius = ((max_x - min_x).max(max_z - min_z)) / 2.0;

                                    // Check Texture Image Size if possible
                                    let uv_scale = 1.0; // Reset to 1.0 to see original pattern
                                    if let Some(img) = images.get(&handles.texture) {
                                        info!("Rabbit Texture Info: {:?} (Size: {:?})", handles.texture.id(), img.size());
                                    }

                                    let uvs: Vec<[f32; 2]> = positions.iter()
                                        .map(|pos| {
                                            // Relativize position
                                            let dx = pos[0] - center_x;
                                            let dy = pos[1] - center_y; // Center Y for spherical
                                            let dz = pos[2] - center_z;

                                            // Spherical Mapping (Project from center of bounding box)
                                            // More uniform for blobby shapes like rabbits
                                            let r = (dx*dx + dy*dy + dz*dz).sqrt();
                                            
                                            // Longitude (Angle around Y) - U
                                            let angle = dz.atan2(dx); 
                                            let u = (angle / (std::f32::consts::PI * 2.0)) + 0.5;
                                            
                                            // Latitude (Angle from North Pole) - V
                                            let lat = (dy / r).acos(); // 0 to PI
                                            let v = lat / std::f32::consts::PI;

                                            [u * uv_scale, v * uv_scale]
                                        })
                                        .collect();
                                    
                                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
                                    info!("Inserted generated Spherical UVs into mesh {:?} (Bounds Center).", mesh_handle.id());
                                    
                                    if let Err(e) = mesh.generate_tangents() {
                                        warn!("Failed to generate tangents: {:?}", e);
                                    }
                                }
                             }
                        }
                     }
                }

                commands.entity(curr).insert(MeshMaterial3d(new_handle));
                fixed_any = true;
            }

            // Push children to continue traversal
            if let Ok(kids) = children_query.get(curr) {
                stack.extend(kids.iter());
            }
        }

        // Mark this rabbit as having its texture fixed
        if fixed_any {
            commands.entity(rabbit_entity).insert(RabbitTextureFixed);
            info!("Fixed texture for rabbit {:?}", rabbit_entity);
        }
    }
}

/// Spawn rabbits on the terrain
pub fn spawn_rabbits(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    mut spawned: ResMut<RabbitSpawned>,
    handles: Option<Res<RabbitHandles>>,
) {
    if spawned.spawned {
        return;
    }

    // Wait for rabbit assets to be loaded
    let Some(handles) = handles else {
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
                                SceneRoot(handles.scene.clone()),
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
