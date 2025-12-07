use bevy::prelude::*;
use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::voxel::chunk::Chunk;
use crate::voxel::meshing::generate_chunk_mesh;
use crate::voxel::types::VoxelType;
use crate::voxel::world::VoxelWorld;
use crate::rendering::materials::VoxelMaterial;
use crate::config::loader::load_config;

pub struct VoxelPlugin;

#[derive(Resource)]
pub struct WorldConfig {
    pub size_chunks: IVec3,
    pub chunk_size: i32,
    pub greedy_meshing: bool,
}

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldConfig {
                size_chunks: IVec3::new(32, 4, 32),
                chunk_size: 16,
                greedy_meshing: true,
            })
            .insert_resource(VoxelWorld::new(IVec3::new(32, 4, 32)))
            .add_systems(Startup, setup_voxel_world)
            .add_systems(Update, mesh_dirty_chunks_system);
    }
}

// Simple pseudo-random noise functions for terrain generation
fn hash(x: i32, z: i32) -> f32 {
    let n = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n ^ (n >> 16)) as u32 as f32) / u32::MAX as f32
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn value_noise(x: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - x.floor();
    let zf = z - z.floor();

    let v00 = hash(xi, zi);
    let v10 = hash(xi + 1, zi);
    let v01 = hash(xi, zi + 1);
    let v11 = hash(xi + 1, zi + 1);

    let u = smoothstep(xf);
    let v = smoothstep(zf);

    lerp(lerp(v00, v10, u), lerp(v01, v11, u), v)
}

fn fbm(x: f32, z: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * value_noise(x * frequency, z * frequency);
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

fn get_terrain_height(world_x: i32, world_z: i32) -> i32 {
    let x = world_x as f32;
    let z = world_z as f32;

    // Base terrain with multiple noise layers
    let base = fbm(x * 0.008, z * 0.008, 4) * 25.0 + 15.0;

    // Hills - larger features
    let hills = fbm(x * 0.02, z * 0.02, 3) * 12.0;

    // Mountains - occasional tall peaks
    let mountain_mask = fbm(x * 0.005, z * 0.005, 2);
    let mountains = if mountain_mask > 0.6 {
        (mountain_mask - 0.6) * 60.0
    } else {
        0.0
    };

    // River valleys - carve into terrain
    let river_noise = (fbm(x * 0.015, z * 0.015, 2) * 6.28).sin();
    let river_factor = if river_noise.abs() < 0.15 {
        -8.0 * (1.0 - river_noise.abs() / 0.15)
    } else {
        0.0
    };

    (base + hills + mountains + river_factor).max(1.0).min(58.0) as i32
}

fn get_biome(world_x: i32, world_z: i32) -> u8 {
    // 0 = normal, 1 = sandy/beach, 2 = rocky, 3 = clay deposits
    let x = world_x as f32;
    let z = world_z as f32;

    let biome_noise = fbm(x * 0.01, z * 0.01, 2);
    let detail_noise = fbm(x * 0.05, z * 0.05, 2);

    if biome_noise < 0.25 {
        1 // Sandy areas
    } else if biome_noise > 0.75 && detail_noise > 0.5 {
        2 // Rocky outcrops
    } else if biome_noise > 0.4 && biome_noise < 0.5 && detail_noise > 0.6 {
        3 // Clay deposits
    } else {
        0 // Normal terrain
    }
}

fn is_cave(world_x: i32, world_y: i32, world_z: i32) -> bool {
    let x = world_x as f32;
    let y = world_y as f32;
    let z = world_z as f32;

    // 3D noise for caves
    let cave_noise = fbm(x * 0.05 + y * 0.03, z * 0.05 + y * 0.02, 3);
    let cave_threshold = 0.65 + (y / 64.0) * 0.1; // Caves more common at lower depths

    cave_noise > cave_threshold && world_y > 2 && world_y < 45
}

fn is_dungeon_wall(world_x: i32, world_y: i32, world_z: i32) -> bool {
    // Create dungeon structures at specific locations
    let dungeon_spacing = 128;
    let dungeon_size = 24;

    let dx = ((world_x % dungeon_spacing) + dungeon_spacing) % dungeon_spacing;
    let dz = ((world_z % dungeon_spacing) + dungeon_spacing) % dungeon_spacing;

    // Check if we're in a dungeon area
    if dx < dungeon_size && dz < dungeon_size && world_y >= 5 && world_y <= 20 {
        let local_x = dx;
        let local_z = dz;
        let local_y = world_y - 5;

        // Create room walls
        let is_outer_wall = local_x == 0 || local_x == dungeon_size - 1 ||
                           local_z == 0 || local_z == dungeon_size - 1;

        // Create inner walls forming corridors
        let corridor_width = 4;
        let wall_at_x = (local_x % 8 == 0 || local_x % 8 == 1) && local_x > 0 && local_x < dungeon_size - 1;
        let wall_at_z = (local_z % 8 == 0 || local_z % 8 == 1) && local_z > 0 && local_z < dungeon_size - 1;

        // Doorways in inner walls
        let doorway_x = local_z >= 3 && local_z <= 5 || local_z >= 11 && local_z <= 13 || local_z >= 19 && local_z <= 21;
        let doorway_z = local_x >= 3 && local_x <= 5 || local_x >= 11 && local_x <= 13 || local_x >= 19 && local_x <= 21;

        let is_inner_wall = (wall_at_x && !doorway_x) || (wall_at_z && !doorway_z);

        // Floor and ceiling
        let is_floor = local_y == 0;
        let is_ceiling = local_y == 10;

        // Pillars at intersections
        let is_pillar = (local_x % 8 <= 1) && (local_z % 8 <= 1) &&
                       local_x > 0 && local_x < dungeon_size - 1 &&
                       local_z > 0 && local_z < dungeon_size - 1;

        return (is_outer_wall || is_inner_wall || is_floor || is_ceiling || is_pillar) && local_y <= 10;
    }

    false
}

/// Check if a tree should spawn at this location
fn should_spawn_tree(world_x: i32, world_z: i32, terrain_height: i32) -> bool {
    // Trees only spawn above water level on grass
    if terrain_height <= WATER_LEVEL + 2 {
        return false;
    }
    
    // Use hash to determine tree placement - sparse distribution
    let tree_noise = hash(world_x.wrapping_mul(7), world_z.wrapping_mul(13));
    
    // About 2% chance per block
    tree_noise > 0.98
}

/// Get tree height at this location (for consistent tree generation)
fn get_tree_height(world_x: i32, world_z: i32) -> i32 {
    let h = hash(world_x.wrapping_add(1000), world_z.wrapping_add(2000));
    3 + (h * 3.0) as i32 // Height between 3 and 5
}

/// Check if a position is part of a tree trunk
fn is_tree_trunk(world_x: i32, world_y: i32, world_z: i32, terrain_height: i32) -> bool {
    if !should_spawn_tree(world_x, world_z, terrain_height) {
        return false;
    }
    
    let trunk_height = get_tree_height(world_x, world_z);
    let trunk_bottom = terrain_height + 1;
    let trunk_top = trunk_bottom + trunk_height;
    
    world_y >= trunk_bottom && world_y < trunk_top
}

/// Check if a position is part of tree leaves
fn is_tree_leaves(world_x: i32, world_y: i32, world_z: i32) -> bool {
    // Check nearby positions for tree trunks
    let radius = 3;
    
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let check_x = world_x + dx;
            let check_z = world_z + dz;
            
            let check_height = get_terrain_height(check_x, check_z);
            
            if should_spawn_tree(check_x, check_z, check_height) {
                let trunk_height = get_tree_height(check_x, check_z);
                let trunk_top = check_height + 1 + trunk_height;
                let leaf_center_y = trunk_top - 1;
                
                // Spherical leaf shape
                let dx_f = dx as f32;
                let dz_f = dz as f32;
                let dy_f = (world_y - leaf_center_y) as f32;
                
                let dist_sq = dx_f * dx_f + dy_f * dy_f * 1.5 + dz_f * dz_f;
                let leaf_radius = 2.5;
                
                if dist_sq < leaf_radius * leaf_radius {
                    // Don't place leaves where trunk is
                    if !(dx == 0 && dz == 0 && world_y < trunk_top) {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

// Water level constant - areas below this height will be filled with water
// Set to 0 to effectively disable water (terrain starts at ~15+)
const WATER_LEVEL: i32 = 0;

fn setup_voxel_world(
    mut world: ResMut<VoxelWorld>,
) {
    // Generate extensive procedural terrain
    let chunk_positions: Vec<IVec3> = world.all_chunk_positions().collect();

    for chunk_pos in chunk_positions {
        let mut chunk = Chunk::new(chunk_pos);
        let chunk_world_x = chunk_pos.x * CHUNK_SIZE_I32;
        let chunk_world_z = chunk_pos.z * CHUNK_SIZE_I32;
        let chunk_world_y = chunk_pos.y * CHUNK_SIZE_I32;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = chunk_world_x + x as i32;
                let world_z = chunk_world_z + z as i32;

                let terrain_height = get_terrain_height(world_x, world_z);
                let biome = get_biome(world_x, world_z);

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_world_y + y as i32;

                    // Check for dungeon structures first - DISABLED FOR DEBUGGING
                    // if is_dungeon_wall(world_x, world_y, world_z) {
                    //     chunk.set(UVec3::new(x as u32, y as u32, z as u32), VoxelType::Rock);
                    //     continue;
                    // }

                    // Check for caves
                    if is_cave(world_x, world_y, world_z) && world_y < terrain_height - 3 {
                        // Fill caves below water level with water
                        let voxel = if world_y <= WATER_LEVEL {
                            VoxelType::Water
                        } else {
                            VoxelType::Air
                        };
                        chunk.set(UVec3::new(x as u32, y as u32, z as u32), voxel);
                        continue;
                    }

                    // Check for tree trunks
                    if is_tree_trunk(world_x, world_y, world_z, terrain_height) {
                        chunk.set(UVec3::new(x as u32, y as u32, z as u32), VoxelType::Wood);
                        continue;
                    }

                    // Check for tree leaves
                    if world_y > terrain_height && is_tree_leaves(world_x, world_y, world_z) {
                        chunk.set(UVec3::new(x as u32, y as u32, z as u32), VoxelType::Leaves);
                        continue;
                    }

                    let voxel = if world_y > terrain_height {
                        // Above terrain - check if below water level
                        if world_y <= WATER_LEVEL {
                            VoxelType::Water
                        } else {
                            VoxelType::Air
                        }
                    } else if world_y == 0 {
                        VoxelType::Bedrock
                    } else if world_y <= 3 {
                        // Deep bedrock layer with some rock
                        if hash(world_x, world_z + world_y * 1000) > 0.3 {
                            VoxelType::Bedrock
                        } else {
                            VoxelType::Rock
                        }
                    } else {
                        // Determine block based on depth from surface and biome
                        let depth = terrain_height - world_y;

                        // Near water, use sand instead of topsoil
                        let near_water = terrain_height <= WATER_LEVEL + 2;

                        match biome {
                            1 => {
                                // Sandy biome
                                if depth <= 4 {
                                    VoxelType::Sand
                                } else if depth <= 8 {
                                    VoxelType::SubSoil
                                } else {
                                    VoxelType::Rock
                                }
                            }
                            2 => {
                                // Rocky biome
                                if depth <= 1 {
                                    VoxelType::Rock
                                } else if depth <= 3 {
                                    VoxelType::SubSoil
                                } else {
                                    VoxelType::Rock
                                }
                            }
                            3 => {
                                // Clay deposits
                                if depth <= 2 {
                                    VoxelType::TopSoil
                                } else if depth <= 6 {
                                    VoxelType::Clay
                                } else if depth <= 10 {
                                    VoxelType::SubSoil
                                } else {
                                    VoxelType::Rock
                                }
                            }
                            _ => {
                                // Normal terrain - use sand near water (beaches)
                                if near_water && depth <= 2 {
                                    VoxelType::Sand
                                } else if depth == 0 {
                                    VoxelType::TopSoil
                                } else if depth <= 4 {
                                    VoxelType::SubSoil
                                } else {
                                    VoxelType::Rock
                                }
                            }
                        }
                    };

                    chunk.set(UVec3::new(x as u32, y as u32, z as u32), voxel);
                }
            }
        }

        chunk.mark_dirty();
        world.insert_chunk(chunk);
    }
}

fn mesh_dirty_chunks_system(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<VoxelMaterial>,
    water_material: Res<crate::rendering::materials::WaterMaterial>,
) {
    // Collect dirty chunks first to avoid borrowing issues
    let dirty_chunks: Vec<IVec3> = world.dirty_chunks().collect();
    
    for chunk_pos in dirty_chunks {
        // Step 1: Generate mesh data using immutable borrow
        let mesh_result = if let Some(chunk) = world.get_chunk(chunk_pos) {
            generate_chunk_mesh(chunk, &world)
        } else {
            continue;
        };

        // Step 2: Update chunk state using mutable borrow
        if let Some(chunk) = world.get_chunk_mut(chunk_pos) {
            // Clear dirty flag
            chunk.clear_dirty();
            
            let world_pos = VoxelWorld::chunk_to_world(chunk_pos);
            
            // Handle solid mesh
            if mesh_result.solid.is_empty() {
                if let Some(entity) = chunk.mesh_entity() {
                    commands.entity(entity).despawn();
                    chunk.clear_mesh_entity();
                }
            } else {
                let mesh = mesh_result.solid.into_mesh();
                let mesh_handle = meshes.add(mesh);
                
                if let Some(entity) = chunk.mesh_entity() {
                    commands.entity(entity).insert(Mesh3d(mesh_handle));
                } else {
                    let entity = commands.spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material.handle.clone()),
                        Transform::from_xyz(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32),
                        crate::voxel::meshing::ChunkMesh { chunk_position: chunk_pos },
                    )).id();
                    chunk.set_mesh_entity(entity);
                }
            }
            
            // Handle water mesh - DISABLED FOR DEBUGGING
            // Skip water mesh creation entirely
            if let Some(entity) = chunk.water_mesh_entity() {
                commands.entity(entity).despawn();
                chunk.clear_water_mesh_entity();
            }
        }
    }
}

