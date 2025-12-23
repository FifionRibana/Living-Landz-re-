// use bevy::math::Vec2;
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct TerrainChunkSdfData {
    /// Grid resolution
    pub resolution: u8,

    /// SDF values encoded as u8 (0-255)
    /// Stored in row-major order: index = y * resolution + x
    /// 0 = water border, 255 = far inland
    pub values: Vec<u8>,
}

impl Default for TerrainChunkSdfData {
    fn default() -> Self {
        Self::new(64)
    }
}

impl TerrainChunkSdfData {
    pub fn new(resolution: u8) -> Self {
        let size = (resolution as usize) * (resolution as usize);
        Self {
            resolution,
            values: vec![255; size], // Default to all inland
        }
    }

    /// Size in bytes for db storage
    pub fn byte_size(&self) -> usize {
        1 + self.values.len() // 1 byte resolution + data
    }
}
