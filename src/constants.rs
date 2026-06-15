// Chunk dimensions
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I32: i32 = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

// World defaults (overridden by config)
pub const DEFAULT_WORLD_CHUNKS_X: i32 = 32;
pub const DEFAULT_WORLD_CHUNKS_Y: i32 = 4;
pub const DEFAULT_WORLD_CHUNKS_Z: i32 = 32;

// Texture atlas
pub const ATLAS_TILE_SIZE: u32 = 256;
pub const ATLAS_COLUMNS: u32 = 4;

// Meshing
pub const VOXEL_SIZE: f32 = 1.0;
