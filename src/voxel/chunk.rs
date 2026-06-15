use crate::constants::{CHUNK_SIZE, CHUNK_VOLUME};
use crate::voxel::types::VoxelType;
use bevy::prelude::*;

pub struct Chunk {
    voxels: [VoxelType; CHUNK_VOLUME],
    dirty: bool,
    mesh_entity: Option<Entity>,
    water_mesh_entity: Option<Entity>,
    position: IVec3, // Chunk coords (not world)
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            voxels: [VoxelType::Air; CHUNK_VOLUME],
            dirty: true,
            mesh_entity: None,
            water_mesh_entity: None,
            position,
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
}

