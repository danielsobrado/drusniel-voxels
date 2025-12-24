use crate::constants::{CHUNK_SIZE, CHUNK_VOLUME};
use crate::voxel::types::VoxelType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Serializable chunk data (voxels only)
#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub voxels: Vec<VoxelType>,
    pub position: IVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodLevel {
    High,
    Low,
    Culled,
}

impl LodLevel {
    pub fn detail_value(&self) -> u8 {
        match self {
            LodLevel::High => 2,
            LodLevel::Low => 1,
            LodLevel::Culled => 0,
        }
    }

    pub fn is_lower_detail_than(self, other: LodLevel) -> bool {
        self.detail_value() < other.detail_value()
    }

    pub fn is_higher_detail_than(self, other: LodLevel) -> bool {
        self.detail_value() > other.detail_value()
    }
}

pub struct Chunk {
    voxels: [VoxelType; CHUNK_VOLUME],
    dirty: bool,
    mesh_entity: Option<Entity>,
    water_mesh_entity: Option<Entity>,
    position: IVec3, // Chunk coords (not world)
    lod_level: LodLevel,
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            voxels: [VoxelType::Air; CHUNK_VOLUME],
            dirty: true,
            mesh_entity: None,
            water_mesh_entity: None,
            position,
            lod_level: LodLevel::High,
        }
    }

    pub fn get(&self, local: UVec3) -> VoxelType {
        let index = Self::index(local.x as usize, local.y as usize, local.z as usize);
        self.voxels[index]
    }

    pub fn set(&mut self, local: UVec3, voxel: VoxelType) {
        let index = Self::index(local.x as usize, local.y as usize, local.z as usize);
        if self.voxels[index] != voxel {
            self.voxels[index] = voxel;
            self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn set_mesh_entity(&mut self, entity: Entity) {
        self.mesh_entity = Some(entity);
    }

    pub fn mesh_entity(&self) -> Option<Entity> {
        self.mesh_entity
    }

    pub fn set_water_mesh_entity(&mut self, entity: Entity) {
        self.water_mesh_entity = Some(entity);
    }

    pub fn water_mesh_entity(&self) -> Option<Entity> {
        self.water_mesh_entity
    }

    pub fn clear_mesh_entity(&mut self) {
        self.mesh_entity = None;
    }

    pub fn clear_water_mesh_entity(&mut self) {
        self.water_mesh_entity = None;
    }

    pub fn position(&self) -> IVec3 {
        self.position
    }

    pub fn lod_level(&self) -> LodLevel {
        self.lod_level
    }

    pub fn set_lod_level(&mut self, lod_level: LodLevel) -> bool {
        if self.lod_level != lod_level {
            self.lod_level = lod_level;
            self.dirty = true;
            return true;
        }
        false
    }

    // For meshing - index conversion
    fn index(x: usize, y: usize, z: usize) -> usize {
        x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE)
    }

    #[allow(dead_code)]
    fn coords(index: usize) -> (usize, usize, usize) {
        let x = index % CHUNK_SIZE;
        let y = (index / CHUNK_SIZE) % CHUNK_SIZE;
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        (x, y, z)
    }

    /// Convert chunk to serializable data
    pub fn to_data(&self) -> ChunkData {
        ChunkData {
            voxels: self.voxels.to_vec(),
            position: self.position,
        }
    }

    /// Create chunk from serializable data
    pub fn from_data(data: ChunkData) -> Self {
        let mut voxels = [VoxelType::Air; CHUNK_VOLUME];
        for (i, v) in data.voxels.into_iter().enumerate() {
            if i < CHUNK_VOLUME {
                voxels[i] = v;
            }
        }
        Self {
            voxels,
            dirty: true, // Mark dirty so mesh gets generated
            mesh_entity: None,
            water_mesh_entity: None,
            position: data.position,
            lod_level: LodLevel::High,
        }
    }
}
