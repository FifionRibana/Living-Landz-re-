use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{BiomeType, TerrainChunkId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct BiomeChunkId {
    pub x: i32,
    pub y: i32,
    pub biome: BiomeType,
}

impl Default for BiomeChunkId {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            biome: BiomeType::DeepOcean,
        }
    }
}

impl BiomeChunkId {
    pub fn from_terrain(terrain: &TerrainChunkId, biome: BiomeType) -> Self {
        Self {
            x: terrain.x,
            y: terrain.y,
            biome,
        }
    }
}
