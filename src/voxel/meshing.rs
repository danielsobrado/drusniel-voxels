use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::constants::VOXEL_SIZE;
use crate::voxel::chunk::Chunk;
use crate::voxel::types::{VoxelType, Voxel};
use crate::voxel::world::VoxelWorld;

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
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
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
                    // Water blocks - only render faces adjacent to air
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
        // If outside world bounds, assume visible
        true
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
    let cols = 4.0;
    let rows = 4.0;
    let col = (atlas_idx % 4) as f32;
    let row = (atlas_idx / 4) as f32;
    
    // UV padding to prevent texture bleeding from adjacent tiles
    let padding = 0.01;
    
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
        mesh_data.indices.push(start_idx + 1);
        mesh_data.indices.push(start_idx + 3);
        mesh_data.indices.push(start_idx);
        
        mesh_data.indices.push(start_idx + 1);
        mesh_data.indices.push(start_idx + 2);
        mesh_data.indices.push(start_idx + 3);
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
    let padding = 0.01;
    
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

