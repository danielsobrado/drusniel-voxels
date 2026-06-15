use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy_mesh::{Indices, PrimitiveTopology};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::constants::VOXEL_SIZE;
use crate::voxel::chunk::Chunk;
use crate::voxel::types::{VoxelType, Voxel};
use crate::voxel::world::VoxelWorld;

// Surface nets imports for smooth meshing
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};
use ndshape::{ConstShape, ConstShape3u32};

// Debug helper: log if a solid face ends up using the water atlas tile
const DEBUG_LOG_WATER_TILE_ON_SOLIDS: bool = true;
const DEBUG_MAX_LOGS: usize = 64;
static DEBUG_WATER_SOLID_LOGS: AtomicUsize = AtomicUsize::new(0);

#[derive(Component)]
pub struct ChunkMesh {
    pub chunk_position: IVec3,
}

#[derive(Copy, Clone, Debug)]
pub enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>, // Vertex colors for AO
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Result of chunk meshing containing separate meshes for solid and water blocks
pub struct ChunkMeshResult {
    pub solid: MeshData,
    pub water: MeshData,
}

pub fn generate_chunk_mesh(
    chunk: &Chunk,
    world: &VoxelWorld,
) -> ChunkMeshResult {
    let mut solid_mesh = MeshData::new();
    let mut water_mesh = MeshData::new();
    
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let local = UVec3::new(x, y, z);
                let voxel = chunk.get(local);
                
                if voxel == VoxelType::Air {
                    continue;
                }

                if voxel.is_liquid() {
                    // Generate water mesh faces (only visible against air)
                    check_water_face(chunk, world, local, Face::Top, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::Bottom, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::North, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::South, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::East, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::West, &mut water_mesh, voxel);
                } else if voxel.is_solid() {
                    // Solid blocks - render faces adjacent to air or water (transparent)
                    check_face(chunk, world, local, Face::Top, &mut solid_mesh, voxel);
                    check_face(chunk, world, local, Face::Bottom, &mut solid_mesh, voxel);
                    check_face(chunk, world, local, Face::North, &mut solid_mesh, voxel);
                    check_face(chunk, world, local, Face::South, &mut solid_mesh, voxel);
                    check_face(chunk, world, local, Face::East, &mut solid_mesh, voxel);
                    check_face(chunk, world, local, Face::West, &mut solid_mesh, voxel);
                }
            }
        }
    }

    ChunkMeshResult {
        solid: solid_mesh,
        water: water_mesh,
    }
}

fn check_face(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
    mesh_data: &mut MeshData,
    voxel: VoxelType,
) {
    if is_face_visible(chunk, world, local, face) {
        add_face_with_ao(mesh_data, chunk, world, local, face, voxel);
    }
}

fn check_water_face(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
    mesh_data: &mut MeshData,
    voxel: VoxelType,
) {
    if is_water_face_visible(chunk, world, local, face) {
        // Water doesn't need AO - use full brightness
        add_face_no_ao(mesh_data, local, face, voxel);
    }
}

fn is_face_visible(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
) -> bool {
    let (dx, dy, dz) = match face {
        Face::Top => (0, 1, 0),
        Face::Bottom => (0, -1, 0),
        Face::North => (0, 0, -1),
        Face::South => (0, 0, 1),
        Face::East => (1, 0, 0),
        Face::West => (-1, 0, 0),
    };

    let neighbor_x = local.x as i32 + dx;
    let neighbor_y = local.y as i32 + dy;
    let neighbor_z = local.z as i32 + dz;

    // If neighbor is within chunk
    if neighbor_x >= 0 && neighbor_x < 16 &&
       neighbor_y >= 0 && neighbor_y < 16 &&
       neighbor_z >= 0 && neighbor_z < 16 {
        let neighbor_voxel = chunk.get(UVec3::new(neighbor_x as u32, neighbor_y as u32, neighbor_z as u32));
        return neighbor_voxel.is_transparent(); // Visible if neighbor is transparent (air or water)
    }

    // If neighbor is outside chunk, check world
    let chunk_pos = chunk.position();
    let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);
    let current_world_pos = chunk_origin + IVec3::new(local.x as i32, local.y as i32, local.z as i32);
    let neighbor_world_pos = current_world_pos + IVec3::new(dx, dy, dz);
    
    if let Some(neighbor_voxel) = world.get_voxel(neighbor_world_pos) {
        neighbor_voxel.is_transparent()
    } else {
        // Outside world bounds - never render faces into the void
        false
    }
}

/// Water faces are only visible when adjacent to air (not other water)
fn is_water_face_visible(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
) -> bool {
    let (dx, dy, dz) = match face {
        Face::Top => (0, 1, 0),
        Face::Bottom => (0, -1, 0),
        Face::North => (0, 0, -1),
        Face::South => (0, 0, 1),
        Face::East => (1, 0, 0),
        Face::West => (-1, 0, 0),
    };

    let neighbor_x = local.x as i32 + dx;
    let neighbor_y = local.y as i32 + dy;
    let neighbor_z = local.z as i32 + dz;

    // If neighbor is within chunk
    if neighbor_x >= 0 && neighbor_x < 16 &&
       neighbor_y >= 0 && neighbor_y < 16 &&
       neighbor_z >= 0 && neighbor_z < 16 {
        let neighbor_voxel = chunk.get(UVec3::new(neighbor_x as u32, neighbor_y as u32, neighbor_z as u32));
        return neighbor_voxel == VoxelType::Air; // Water only visible against air
    }

    // If neighbor is outside chunk, check world
    let chunk_pos = chunk.position();
    let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);
    let current_world_pos = chunk_origin + IVec3::new(local.x as i32, local.y as i32, local.z as i32);
    let neighbor_world_pos = current_world_pos + IVec3::new(dx, dy, dz);
    
    if let Some(neighbor_voxel) = world.get_voxel(neighbor_world_pos) {
        neighbor_voxel == VoxelType::Air
    } else {
        // If outside world bounds, assume visible (edge of world)
        true
    }
}

/// Calculate vertex ambient occlusion (0-3 scale, 0 = fully occluded, 3 = not occluded)
fn calculate_vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let ao = if side1 && side2 {
        0 // Fully occluded
    } else {
        3 - (side1 as i32 + side2 as i32 + corner as i32)
    };
    // Convert to brightness (0.4 to 1.0 range for visible difference)
    0.4 + (ao as f32 / 3.0) * 0.6
}

/// Check if a world position contains a solid block (for AO calculation)
fn is_solid_at_offset(chunk: &Chunk, world: &VoxelWorld, local: UVec3, offset: IVec3) -> bool {
    let local_pos = IVec3::new(local.x as i32, local.y as i32, local.z as i32) + offset;
    
    // Check within chunk first
    if local_pos.x >= 0 && local_pos.x < 16 &&
       local_pos.y >= 0 && local_pos.y < 16 &&
       local_pos.z >= 0 && local_pos.z < 16 {
        let v = chunk.get(UVec3::new(local_pos.x as u32, local_pos.y as u32, local_pos.z as u32));
        return v.is_solid();
    }
    
    // Check world
    let chunk_pos = chunk.position();
    let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);
    let world_pos = chunk_origin + local_pos;
    
    if let Some(v) = world.get_voxel(world_pos) {
        v.is_solid()
    } else {
        false
    }
}

/// Get AO values for the 4 vertices of a face
fn get_face_ao(chunk: &Chunk, world: &VoxelWorld, local: UVec3, face: Face) -> [f32; 4] {
    // For each face, we need to check the 8 neighbors in the plane of the face
    // and calculate AO for each of the 4 vertices
    
    let offsets = match face {
        Face::Top => {
            // Vertices: v0(0,1,1), v1(1,1,1), v2(1,1,0), v3(0,1,0)
            [
                (IVec3::new(-1, 1, 0), IVec3::new(0, 1, 1), IVec3::new(-1, 1, 1)),   // v0
                (IVec3::new(1, 1, 0), IVec3::new(0, 1, 1), IVec3::new(1, 1, 1)),     // v1
                (IVec3::new(1, 1, 0), IVec3::new(0, 1, -1), IVec3::new(1, 1, -1)),   // v2
                (IVec3::new(-1, 1, 0), IVec3::new(0, 1, -1), IVec3::new(-1, 1, -1)), // v3
            ]
        }
        Face::Bottom => {
            [
                (IVec3::new(-1, -1, 0), IVec3::new(0, -1, -1), IVec3::new(-1, -1, -1)),
                (IVec3::new(1, -1, 0), IVec3::new(0, -1, -1), IVec3::new(1, -1, -1)),
                (IVec3::new(1, -1, 0), IVec3::new(0, -1, 1), IVec3::new(1, -1, 1)),
                (IVec3::new(-1, -1, 0), IVec3::new(0, -1, 1), IVec3::new(-1, -1, 1)),
            ]
        }
        Face::North => {
            [
                (IVec3::new(1, 0, -1), IVec3::new(0, -1, -1), IVec3::new(1, -1, -1)),
                (IVec3::new(-1, 0, -1), IVec3::new(0, -1, -1), IVec3::new(-1, -1, -1)),
                (IVec3::new(-1, 0, -1), IVec3::new(0, 1, -1), IVec3::new(-1, 1, -1)),
                (IVec3::new(1, 0, -1), IVec3::new(0, 1, -1), IVec3::new(1, 1, -1)),
            ]
        }
        Face::South => {
            [
                (IVec3::new(-1, 0, 1), IVec3::new(0, -1, 1), IVec3::new(-1, -1, 1)),
                (IVec3::new(1, 0, 1), IVec3::new(0, -1, 1), IVec3::new(1, -1, 1)),
                (IVec3::new(1, 0, 1), IVec3::new(0, 1, 1), IVec3::new(1, 1, 1)),
                (IVec3::new(-1, 0, 1), IVec3::new(0, 1, 1), IVec3::new(-1, 1, 1)),
            ]
        }
        Face::East => {
            [
                (IVec3::new(1, 0, 1), IVec3::new(1, -1, 0), IVec3::new(1, -1, 1)),
                (IVec3::new(1, 0, -1), IVec3::new(1, -1, 0), IVec3::new(1, -1, -1)),
                (IVec3::new(1, 0, -1), IVec3::new(1, 1, 0), IVec3::new(1, 1, -1)),
                (IVec3::new(1, 0, 1), IVec3::new(1, 1, 0), IVec3::new(1, 1, 1)),
            ]
        }
        Face::West => {
            [
                (IVec3::new(-1, 0, -1), IVec3::new(-1, -1, 0), IVec3::new(-1, -1, -1)),
                (IVec3::new(-1, 0, 1), IVec3::new(-1, -1, 0), IVec3::new(-1, -1, 1)),
                (IVec3::new(-1, 0, 1), IVec3::new(-1, 1, 0), IVec3::new(-1, 1, 1)),
                (IVec3::new(-1, 0, -1), IVec3::new(-1, 1, 0), IVec3::new(-1, 1, -1)),
            ]
        }
    };
    
    let mut ao = [1.0; 4];
    for (i, (side1_off, side2_off, corner_off)) in offsets.iter().enumerate() {
        let side1 = is_solid_at_offset(chunk, world, local, *side1_off);
        let side2 = is_solid_at_offset(chunk, world, local, *side2_off);
        let corner = is_solid_at_offset(chunk, world, local, *corner_off);
        ao[i] = calculate_vertex_ao(side1, side2, corner);
    }
    ao
}

/// Get the atlas index for a voxel face (supports face-specific textures)
fn get_face_atlas_index(voxel: VoxelType, face: Face) -> u8 {
    match voxel {
        VoxelType::TopSoil => {
            match face {
                Face::Top => 0,    // Grass top texture
                Face::Bottom => 1, // Dirt texture
                _ => 7,            // Grass side texture (uses slot 7)
            }
        }
        _ => voxel.atlas_index(),
    }
}

/// Get UV rotation (0-3) based on world position to break up tiling patterns
/// Returns rotation: 0=0°, 1=90°, 2=180°, 3=270°
fn get_uv_rotation(world_x: i32, world_y: i32, world_z: i32) -> u8 {
    // Use a simple hash of position to get pseudo-random rotation
    let hash = world_x.wrapping_mul(73856093)
        ^ world_y.wrapping_mul(19349663)
        ^ world_z.wrapping_mul(83492791);
    (hash as u8) & 3 // Returns 0, 1, 2, or 3
}

/// Apply UV rotation to break up tiling patterns
/// rotation: 0=0°, 1=90°, 2=180°, 3=270°
fn rotate_uvs(uvs: [[f32; 2]; 4], rotation: u8) -> [[f32; 2]; 4] {
    match rotation {
        0 => uvs,                                    // No rotation
        1 => [uvs[3], uvs[0], uvs[1], uvs[2]],      // 90° CW
        2 => [uvs[2], uvs[3], uvs[0], uvs[1]],      // 180°
        3 => [uvs[1], uvs[2], uvs[3], uvs[0]],      // 270° CW
        _ => uvs,
    }
}

fn add_face_with_ao(
    mesh_data: &mut MeshData,
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
    voxel: VoxelType,
) {
    let x = local.x as f32 * VOXEL_SIZE;
    let y = local.y as f32 * VOXEL_SIZE;
    let z = local.z as f32 * VOXEL_SIZE;
    let s = VOXEL_SIZE;

    let (v0, v1, v2, v3, normal) = match face {
        Face::Top => (
            [x, y + s, z + s], [x + s, y + s, z + s], [x + s, y + s, z], [x, y + s, z],
            [0.0, 1.0, 0.0]
        ),
        Face::Bottom => (
            [x, y, z], [x + s, y, z], [x + s, y, z + s], [x, y, z + s],
            [0.0, -1.0, 0.0]
        ),
        Face::North => (
            [x + s, y, z], [x, y, z], [x, y + s, z], [x + s, y + s, z],
            [0.0, 0.0, -1.0]
        ),
        Face::South => (
            [x, y, z + s], [x + s, y, z + s], [x + s, y + s, z + s], [x, y + s, z + s],
            [0.0, 0.0, 1.0]
        ),
        Face::East => (
            [x + s, y, z + s], [x + s, y, z], [x + s, y + s, z], [x + s, y + s, z + s],
            [1.0, 0.0, 0.0]
        ),
        Face::West => (
            [x, y, z], [x, y, z + s], [x, y + s, z + s], [x, y + s, z],
            [-1.0, 0.0, 0.0]
        ),
    };

    // Calculate AO for each vertex
    let ao = get_face_ao(chunk, world, local, face);

    let start_idx = mesh_data.positions.len() as u32;
    
    mesh_data.positions.push(v0);
    mesh_data.positions.push(v1);
    mesh_data.positions.push(v2);
    mesh_data.positions.push(v3);
    
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    
    // Add vertex colors for AO (grayscale)
    mesh_data.colors.push([ao[0], ao[0], ao[0], 1.0]);
    mesh_data.colors.push([ao[1], ao[1], ao[1], 1.0]);
    mesh_data.colors.push([ao[2], ao[2], ao[2], 1.0]);
    mesh_data.colors.push([ao[3], ao[3], ao[3], 1.0]);
    
    // Face-specific texture
    let atlas_idx = get_face_atlas_index(voxel, face);

    if DEBUG_LOG_WATER_TILE_ON_SOLIDS && atlas_idx == VoxelType::Water.atlas_index() {
        let count = DEBUG_WATER_SOLID_LOGS.fetch_add(1, Ordering::Relaxed);
        if count < DEBUG_MAX_LOGS {
            let chunk_origin = VoxelWorld::chunk_to_world(chunk.position());
            let world_pos = chunk_origin + IVec3::new(local.x as i32, local.y as i32, local.z as i32);
            info!(
                "Solid face using water tile at {:?}, voxel {:?}, face {:?}",
                world_pos, voxel, face
            );
        }
    }
    let cols = 4.0;
    let rows = 4.0;
    let col = (atlas_idx % 4) as f32;
    let row = (atlas_idx / 4) as f32;
    
    // UV padding to prevent texture bleeding from adjacent tiles
    let padding = 0.02;
    
    let u_min = col / cols + padding;
    let u_max = (col + 1.0) / cols - padding;
    let v_min = row / rows + padding;
    let v_max = (row + 1.0) / rows - padding;
    
    mesh_data.uvs.push([u_min, v_max]);
    mesh_data.uvs.push([u_max, v_max]);
    mesh_data.uvs.push([u_max, v_min]);
    mesh_data.uvs.push([u_min, v_min]);
    
    // Use flipped winding for proper AO interpolation when needed
    // Check if we should flip the quad diagonal based on AO values
    if ao[0] + ao[2] > ao[1] + ao[3] {
        // Normal winding
        mesh_data.indices.push(start_idx);
        mesh_data.indices.push(start_idx + 2);
        mesh_data.indices.push(start_idx + 1);
        
        mesh_data.indices.push(start_idx);
        mesh_data.indices.push(start_idx + 3);
        mesh_data.indices.push(start_idx + 2);
    } else {
        // Flipped diagonal for better AO interpolation
        // Triangle 1: v1, v0, v3 (CCW)
        mesh_data.indices.push(start_idx + 1);
        mesh_data.indices.push(start_idx);
        mesh_data.indices.push(start_idx + 3);
        
        // Triangle 2: v1, v3, v2 (CCW)
        mesh_data.indices.push(start_idx + 1);
        mesh_data.indices.push(start_idx + 3);
        mesh_data.indices.push(start_idx + 2);
    }
}

fn add_face_no_ao(
    mesh_data: &mut MeshData,
    local: UVec3,
    face: Face,
    voxel: VoxelType,
) {
    let x = local.x as f32 * VOXEL_SIZE;
    let y = local.y as f32 * VOXEL_SIZE;
    let z = local.z as f32 * VOXEL_SIZE;
    let s = VOXEL_SIZE;

    // Inset water faces slightly to prevent them showing through terrain gaps
    // The smooth terrain mesh may not perfectly align with blocky water mesh
    let inset = 0.05;

    let (v0, v1, v2, v3, normal) = match face {
        Face::Top => (
            [x + inset, y + s - inset, z + s - inset], [x + s - inset, y + s - inset, z + s - inset],
            [x + s - inset, y + s - inset, z + inset], [x + inset, y + s - inset, z + inset],
            [0.0, 1.0, 0.0]
        ),
        Face::Bottom => (
            [x + inset, y + inset, z + inset], [x + s - inset, y + inset, z + inset],
            [x + s - inset, y + inset, z + s - inset], [x + inset, y + inset, z + s - inset],
            [0.0, -1.0, 0.0]
        ),
        Face::North => (
            [x + s - inset, y + inset, z + inset], [x + inset, y + inset, z + inset],
            [x + inset, y + s - inset, z + inset], [x + s - inset, y + s - inset, z + inset],
            [0.0, 0.0, -1.0]
        ),
        Face::South => (
            [x + inset, y + inset, z + s - inset], [x + s - inset, y + inset, z + s - inset],
            [x + s - inset, y + s - inset, z + s - inset], [x + inset, y + s - inset, z + s - inset],
            [0.0, 0.0, 1.0]
        ),
        Face::East => (
            [x + s - inset, y + inset, z + s - inset], [x + s - inset, y + inset, z + inset],
            [x + s - inset, y + s - inset, z + inset], [x + s - inset, y + s - inset, z + s - inset],
            [1.0, 0.0, 0.0]
        ),
        Face::West => (
            [x + inset, y + inset, z + inset], [x + inset, y + inset, z + s - inset],
            [x + inset, y + s - inset, z + s - inset], [x + inset, y + s - inset, z + inset],
            [-1.0, 0.0, 0.0]
        ),
    };

    let start_idx = mesh_data.positions.len() as u32;
    
    mesh_data.positions.push(v0);
    mesh_data.positions.push(v1);
    mesh_data.positions.push(v2);
    mesh_data.positions.push(v3);
    
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    
    // Full brightness for water
    mesh_data.colors.push([1.0, 1.0, 1.0, 1.0]);
    mesh_data.colors.push([1.0, 1.0, 1.0, 1.0]);
    mesh_data.colors.push([1.0, 1.0, 1.0, 1.0]);
    mesh_data.colors.push([1.0, 1.0, 1.0, 1.0]);
    
    let atlas_idx = voxel.atlas_index();
    let cols = 4.0;
    let rows = 4.0;
    let col = (atlas_idx % 4) as f32;
    let row = (atlas_idx / 4) as f32;
    
    // UV padding to prevent texture bleeding from adjacent tiles
    let padding = 0.02;
    
    let u_min = col / cols + padding;
    let u_max = (col + 1.0) / cols - padding;
    let v_min = row / rows + padding;
    let v_max = (row + 1.0) / rows - padding;
    
    mesh_data.uvs.push([u_min, v_max]);
    mesh_data.uvs.push([u_max, v_max]);
    mesh_data.uvs.push([u_max, v_min]);
    mesh_data.uvs.push([u_min, v_min]);
    
    mesh_data.indices.push(start_idx);
    mesh_data.indices.push(start_idx + 2);
    mesh_data.indices.push(start_idx + 1);

    mesh_data.indices.push(start_idx);
    mesh_data.indices.push(start_idx + 3);
    mesh_data.indices.push(start_idx + 2);
}

// =============================================================================
// Surface Nets Smooth Meshing
// =============================================================================

/// Padded chunk shape for surface nets (18x18x18 for 16x16x16 chunk + 1 padding)
type PaddedChunkShape = ConstShape3u32<18, 18, 18>;

/// Sample voxel from world or chunk, returns true if solid OR water
/// Water is treated as solid for SDF purposes to prevent surface nets from generating
/// surfaces at solid-water boundaries (which would create visible seams with the blocky water mesh)
fn sample_voxel_solid(chunk: &Chunk, world: &VoxelWorld, chunk_origin: IVec3, px: u32, py: u32, pz: u32) -> bool {
    let world_pos = chunk_origin + IVec3::new(px as i32 - 1, py as i32 - 1, pz as i32 - 1);

    let voxel = if px >= 1 && px <= 16 && py >= 1 && py <= 16 && pz >= 1 && pz <= 16 {
        chunk.get(UVec3::new(px - 1, py - 1, pz - 1))
    } else {
        world.get_voxel(world_pos).unwrap_or(VoxelType::Air)
    };

    // Treat water as solid for SDF so we don't generate surfaces at solid-water boundaries
    voxel.is_solid() || voxel.is_liquid()
}

/// Generate an SDF array from voxel data with 1-voxel padding for neighbor sampling
/// Uses distance-based SDF for smoother surfaces at chunk boundaries
fn generate_sdf(chunk: &Chunk, world: &VoxelWorld) -> [f32; 5832] { // 18^3 = 5832
    let mut sdf = [1.0f32; PaddedChunkShape::USIZE];
    let chunk_pos = chunk.position();
    let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);

    // First pass: set binary solid/air values
    for i in 0..PaddedChunkShape::USIZE {
        let [px, py, pz] = PaddedChunkShape::delinearize(i as u32);
        let is_solid = sample_voxel_solid(chunk, world, chunk_origin, px, py, pz);
        // SDF: negative inside solid, positive in air
        sdf[i] = if is_solid { -1.0 } else { 1.0 };
    }

    // Second pass: smooth SDF values at boundaries by averaging with neighbors
    // This creates smoother transitions and helps with chunk boundary alignment
    let mut smoothed = sdf;
    for i in 0..PaddedChunkShape::USIZE {
        let [px, py, pz] = PaddedChunkShape::delinearize(i as u32);

        // Only smooth interior cells (not at array edges)
        if px > 0 && px < 17 && py > 0 && py < 17 && pz > 0 && pz < 17 {
            let current = sdf[i];

            // Check if this is a boundary cell (sign changes with any neighbor)
            let neighbors = [
                sdf[PaddedChunkShape::linearize([px - 1, py, pz]) as usize],
                sdf[PaddedChunkShape::linearize([px + 1, py, pz]) as usize],
                sdf[PaddedChunkShape::linearize([px, py - 1, pz]) as usize],
                sdf[PaddedChunkShape::linearize([px, py + 1, pz]) as usize],
                sdf[PaddedChunkShape::linearize([px, py, pz - 1]) as usize],
                sdf[PaddedChunkShape::linearize([px, py, pz + 1]) as usize],
            ];

            let has_sign_change = neighbors.iter().any(|&n| (n > 0.0) != (current > 0.0));

            if has_sign_change {
                // At surface boundary, use a value between -0.5 and 0.5 for smoother interpolation
                let neighbor_avg: f32 = neighbors.iter().sum::<f32>() / 6.0;
                smoothed[i] = (current + neighbor_avg) * 0.5;
            }
        }
    }

    smoothed
}

/// Sample the voxel type at a world position for texture lookup
fn sample_voxel_for_texture(chunk: &Chunk, world: &VoxelWorld, local_pos: Vec3) -> VoxelType {
    // Convert local position to world position for accurate sampling
    // This handles positions outside the [0,15] range due to Surface Nets padding
    let chunk_origin = VoxelWorld::chunk_to_world(chunk.position());
    let world_pos = IVec3::new(
        chunk_origin.x + local_pos.x.round() as i32,
        chunk_origin.y + local_pos.y.round() as i32,
        chunk_origin.z + local_pos.z.round() as i32,
    );

    // First try exact position
    if let Some(voxel) = world.get_voxel(world_pos) {
        if voxel.is_solid() {
            return voxel;
        }
    }

    // If we hit air/water, search nearby for a solid voxel
    // Prioritize searching upward first (more likely to find terrain surface)
    for dy in [0i32, -1, 1, -2, 2] {
        for dx in [-1i32, 0, 1] {
            for dz in [-1i32, 0, 1] {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue; // Already checked
                }
                let check_pos = world_pos + IVec3::new(dx, dy, dz);
                if let Some(v) = world.get_voxel(check_pos) {
                    if v.is_solid() {
                        return v;
                    }
                }
            }
        }
    }
    VoxelType::TopSoil // Default fallback
}

/// Compute planar UV coordinates in world space that tile within an atlas tile
fn compute_triplanar_uv(world_pos: Vec3, normal: [f32; 3], atlas_idx: u8) -> [f32; 2] {
    let cols = 4.0f32;
    let rows = 4.0f32;
    let col = (atlas_idx % 4) as f32;
    let row = (atlas_idx / 4) as f32;

    // UV padding to prevent bleeding
    let padding = 0.03;
    let tile_size = 1.0 / cols - padding * 2.0;
    let u_base = col / cols + padding;
    let v_base = row / rows + padding;

    // Use normal to determine dominant axis for projection
    let abs_normal = [normal[0].abs(), normal[1].abs(), normal[2].abs()];

    let (u_world, v_world) = if abs_normal[1] > abs_normal[0] && abs_normal[1] > abs_normal[2] {
        // Top/bottom face - use X and Z
        (world_pos.x, world_pos.z)
    } else if abs_normal[0] > abs_normal[2] {
        // East/west face - use Z and Y
        (world_pos.z, world_pos.y)
    } else {
        // North/south face - use X and Y
        (world_pos.x, world_pos.y)
    };

    // World-space scale so the pattern stays continuous across chunks
    let tex_scale = 1.0 / 4.0; // 1 tile per 4 world units

    // Get fractional part, handling negative values correctly
    // rem_euclid ensures we always get a positive value in [0, 1)
    let u_frac = (u_world * tex_scale).rem_euclid(1.0);
    let v_frac = (v_world * tex_scale).rem_euclid(1.0);

    // Clamp to ensure we stay within tile bounds (avoid edge sampling issues)
    let u_clamped = u_frac.clamp(0.01, 0.99);
    let v_clamped = v_frac.clamp(0.01, 0.99);

    let u = u_base + u_clamped * tile_size;
    let v = v_base + v_clamped * tile_size;

    // Final safety clamp to ensure valid UV coordinates
    [
        u.clamp(0.001, 0.999),
        v.clamp(0.001, 0.999),
    ]
}

/// Generate mesh using Surface Nets algorithm for smooth terrain
pub fn generate_chunk_mesh_surface_nets(
    chunk: &Chunk,
    world: &VoxelWorld,
) -> ChunkMeshResult {
    let mut solid_mesh = MeshData::new();
    let mut water_mesh = MeshData::new();
    let chunk_origin = VoxelWorld::chunk_to_world(chunk.position());

    // Scale factor to slightly enlarge chunks so they overlap at boundaries
    // This prevents gaps (sky showing through) at chunk seams caused by
    // independent SDF smoothing per chunk producing slightly different vertex positions
    const CHUNK_SCALE: f32 = 1.01; // 1.0% larger overlap
    let chunk_center = Vec3::new(8.0, 8.0, 8.0) * VOXEL_SIZE; // Center of 16x16x16 chunk

    // Generate SDF from voxel data
    let sdf = generate_sdf(chunk, world);

    // Run surface nets on the SDF
    // Extract the full padded region [0,0,0] to [17,17,17)
    // Including the padding lets the mesh extend half a voxel past each edge,
    // so neighboring chunks meet without leaving a one-voxel gap.
    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
        &sdf,
        &PaddedChunkShape {},
        [0; 3],  // Start at 0 (include negative padding)
        [17; 3], // End at 17 (include positive padding)
        &mut buffer,
    );

    // Convert surface nets output to MeshData
    // Use per-triangle vertices to ensure consistent material indices (no interpolation artifacts)
    if !buffer.positions.is_empty() && !buffer.indices.is_empty() {
        // Process triangles one at a time, duplicating vertices per triangle
        // This prevents atlas index interpolation between different materials
        for tri_idx in (0..buffer.indices.len()).step_by(3) {
            let i0 = buffer.indices[tri_idx] as usize;
            let i1 = buffer.indices[tri_idx + 1] as usize;
            let i2 = buffer.indices[tri_idx + 2] as usize;

            // Get positions for this triangle
            let pos0 = buffer.positions.get(i0).copied().unwrap_or([0.0; 3]);
            let pos1 = buffer.positions.get(i1).copied().unwrap_or([0.0; 3]);
            let pos2 = buffer.positions.get(i2).copied().unwrap_or([0.0; 3]);

            // Fix NaN/infinite values
            let safe_pos = |pos: [f32; 3]| -> [f32; 3] {
                [
                    if pos[0].is_finite() { pos[0] } else { 0.0 },
                    if pos[1].is_finite() { pos[1] } else { 0.0 },
                    if pos[2].is_finite() { pos[2] } else { 0.0 },
                ]
            };

            let safe_pos0 = safe_pos(pos0);
            let safe_pos1 = safe_pos(pos1);
            let safe_pos2 = safe_pos(pos2);

            // Calculate local positions (offset for padding)
            let local0 = Vec3::new(safe_pos0[0] - 1.0, safe_pos0[1] - 1.0, safe_pos0[2] - 1.0);
            let local1 = Vec3::new(safe_pos1[0] - 1.0, safe_pos1[1] - 1.0, safe_pos1[2] - 1.0);
            let local2 = Vec3::new(safe_pos2[0] - 1.0, safe_pos2[1] - 1.0, safe_pos2[2] - 1.0);

            // Calculate triangle centroid for material sampling
            let centroid = (local0 + local1 + local2) / 3.0;

            // Get normals for this triangle
            let get_normal = |i: usize| -> [f32; 3] {
                let n = buffer.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                if n[0].is_finite() && n[1].is_finite() && n[2].is_finite() {
                    let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
                    if len > 0.001 {
                        [n[0]/len, n[1]/len, n[2]/len]
                    } else {
                        [0.0, 1.0, 0.0]
                    }
                } else {
                    [0.0, 1.0, 0.0]
                }
            };

            let normal0 = get_normal(i0);
            let normal1 = get_normal(i1);
            let normal2 = get_normal(i2);

            // Average normal for the triangle (used for material selection)
            let avg_normal = [
                (normal0[0] + normal1[0] + normal2[0]) / 3.0,
                (normal0[1] + normal1[1] + normal2[1]) / 3.0,
                (normal0[2] + normal1[2] + normal2[2]) / 3.0,
            ];
            let avg_len = (avg_normal[0]*avg_normal[0] + avg_normal[1]*avg_normal[1] + avg_normal[2]*avg_normal[2]).sqrt();
            let avg_normal = if avg_len > 0.001 {
                [avg_normal[0]/avg_len, avg_normal[1]/avg_len, avg_normal[2]/avg_len]
            } else {
                [0.0, 1.0, 0.0]
            };

            // Calculate material weights for each vertex
            let compute_vertex_weights = |local_pos: Vec3| -> [f32; 4] {
                let mut weights = [0.0f32; 4];
                let mut total_weight = 0.0;
                
                // Check 8 neighbors of the cell containing the vertex
                let base_x = local_pos.x.floor() as i32;
                let base_y = local_pos.y.floor() as i32;
                let base_z = local_pos.z.floor() as i32;
                
                let chunk_pos = chunk.position();
                let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);

                for dz in 0..2 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let lx = base_x + dx;
                            let ly = base_y + dy;
                            let lz = base_z + dz;
                            
                            let voxel = if lx >= 0 && lx < 16 && ly >= 0 && ly < 16 && lz >= 0 && lz < 16 {
                                chunk.get(UVec3::new(lx as u32, ly as u32, lz as u32))
                            } else {
                                let wx = chunk_origin.x + lx;
                                let wy = chunk_origin.y + ly;
                                let wz = chunk_origin.z + lz;
                                world.get_voxel(IVec3::new(wx, wy, wz)).unwrap_or(VoxelType::Air)
                            };

                            if voxel != VoxelType::Air && voxel != VoxelType::Water {
                                let mat_idx = match voxel {
                                    VoxelType::TopSoil => 0, // Grass
                                    
                                    VoxelType::Rock | VoxelType::Bedrock | 
                                    VoxelType::DungeonWall | VoxelType::DungeonFloor => 1, // Rock
                                    
                                    VoxelType::Sand => 2, // Sand
                                    
                                    // Everything else maps to Dirt
                                    VoxelType::SubSoil | VoxelType::Clay | 
                                    VoxelType::Wood | VoxelType::Leaves | _ => 3, 
                                };
                                
                                // Distance-based weighting (closer voxels have more influence)
                                // This assumes local_pos is within the cell [base, base+1]
                                // let dist_sq = (lx as f32 - local_pos.x).powi(2) + 
                                //               (ly as f32 - local_pos.y).powi(2) + 
                                //               (lz as f32 - local_pos.z).powi(2);
                                // let weight = 1.0 / (dist_sq + 0.001);
                                
                                // Simple binary presence also works well for Surface Nets
                                let weight = 1.0;
                                
                                weights[mat_idx] += weight;
                                total_weight += weight;
                            }
                        }
                    }
                }
                
                if total_weight > 0.0 {
                    [
                        weights[0] / total_weight,
                        weights[1] / total_weight,
                        weights[2] / total_weight,
                        weights[3] / total_weight,
                    ]
                } else {
                    // Default to dirt if isolated (shouldn't happen for valid mesh)
                    [0.0, 0.0, 0.0, 1.0] 
                }
            };

            let weights0 = compute_vertex_weights(local0);
            let weights1 = compute_vertex_weights(local1);
            let weights2 = compute_vertex_weights(local2);

            // Add all 3 vertices for this triangle (not shared)
            let base_idx = solid_mesh.positions.len() as u32;

            // Helper to scale vertex position outward from chunk center to close seams
            let scale_vertex = |local: Vec3| -> [f32; 3] {
                let pos = Vec3::new(local.x * VOXEL_SIZE, local.y * VOXEL_SIZE, local.z * VOXEL_SIZE);
                let scaled = chunk_center + (pos - chunk_center) * CHUNK_SCALE;
                [scaled.x, scaled.y, scaled.z]
            };

            // Vertex 0
            solid_mesh.positions.push(scale_vertex(local0));
            solid_mesh.normals.push(normal0);
            solid_mesh.uvs.push([0.0, 0.0]); // UVs not used for splatting logic
            solid_mesh.colors.push(weights0);

            // Vertex 1
            solid_mesh.positions.push(scale_vertex(local1));
            solid_mesh.normals.push(normal1);
            solid_mesh.uvs.push([0.0, 0.0]);
            solid_mesh.colors.push(weights1);

            // Vertex 2
            solid_mesh.positions.push(scale_vertex(local2));
            solid_mesh.normals.push(normal2);
            solid_mesh.uvs.push([0.0, 0.0]);
            solid_mesh.colors.push(weights2);

            // Add triangle indices (sequential since vertices are not shared)
            solid_mesh.indices.push(base_idx);
            solid_mesh.indices.push(base_idx + 1);
            solid_mesh.indices.push(base_idx + 2);
        }
    }

    // Water still uses blocky meshing for now
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let local = UVec3::new(x, y, z);
                let voxel = chunk.get(local);

                if voxel.is_liquid() {
                    check_water_face(chunk, world, local, Face::Top, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::Bottom, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::North, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::South, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::East, &mut water_mesh, voxel);
                    check_water_face(chunk, world, local, Face::West, &mut water_mesh, voxel);
                }
            }
        }
    }

    ChunkMeshResult {
        solid: solid_mesh,
        water: water_mesh,
    }
}

/// Mesh generation mode
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeshMode {
    /// Traditional blocky voxel meshing (Minecraft-style)
    #[default]
    Blocky,
    /// Smooth meshing using Surface Nets algorithm
    SurfaceNets,
}

/// Resource to control mesh generation mode globally
#[derive(Resource, Clone, Copy, Debug)]
pub struct MeshSettings {
    pub mode: MeshMode,
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            mode: MeshMode::Blocky,
        }
    }
}

/// Generate chunk mesh using the specified mode
pub fn generate_chunk_mesh_with_mode(
    chunk: &Chunk,
    world: &VoxelWorld,
    mode: MeshMode,
) -> ChunkMeshResult {
    match mode {
        MeshMode::Blocky => generate_chunk_mesh(chunk, world),
        MeshMode::SurfaceNets => generate_chunk_mesh_surface_nets(chunk, world),
    }
}

