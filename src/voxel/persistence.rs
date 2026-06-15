use crate::voxel::chunk::ChunkData;
use crate::voxel::world::VoxelWorld;
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

const WORLD_SAVE_PATH: &str = "world_data.bin";

/// Serializable world data
#[derive(Serialize, Deserialize)]
pub struct WorldData {
    pub world_size_chunks: IVec3,
    pub chunks: Vec<ChunkData>,
}

/// Save the world to disk using bincode for fast serialization
pub fn save_world(world: &VoxelWorld) -> Result<(), String> {
    let data = world.to_data();

    let file = File::create(WORLD_SAVE_PATH)
        .map_err(|e| format!("Failed to create save file: {}", e))?;
    let writer = BufWriter::new(file);

    bincode::serialize_into(writer, &data)
        .map_err(|e| format!("Failed to serialize world: {}", e))?;

    info!("World saved to {} ({} chunks)", WORLD_SAVE_PATH, data.chunks.len());
    Ok(())
}

/// Load the world from disk
pub fn load_world() -> Result<VoxelWorld, String> {
    let path = Path::new(WORLD_SAVE_PATH);

    if !path.exists() {
        return Err("No saved world found".to_string());
    }

    let file = File::open(path)
        .map_err(|e| format!("Failed to open save file: {}", e))?;
    let reader = BufReader::new(file);

    let data: WorldData = bincode::deserialize_from(reader)
        .map_err(|e| format!("Failed to deserialize world: {}", e))?;

    info!("World loaded from {} ({} chunks)", WORLD_SAVE_PATH, data.chunks.len());

    Ok(VoxelWorld::from_data(data))
}

/// Check if a saved world exists
pub fn saved_world_exists() -> bool {
    Path::new(WORLD_SAVE_PATH).exists()
}

/// Delete the saved world file
pub fn delete_saved_world() -> Result<(), String> {
    let path = Path::new(WORLD_SAVE_PATH);
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to delete save file: {}", e))?;
        info!("Deleted saved world at {}", WORLD_SAVE_PATH);
    }
    Ok(())
}

/// Resource to control world persistence behavior
#[derive(Resource, Clone, Debug)]
pub struct WorldPersistence {
    /// Force regeneration even if saved world exists
    pub force_regenerate: bool,
    /// Auto-save world after generation
    pub auto_save: bool,
}

impl Default for WorldPersistence {
    fn default() -> Self {
        Self {
            force_regenerate: false,
            auto_save: true,
        }
    }
}
