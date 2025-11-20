use bincode::{Decode, Encode};
use crate::{BiomeTypeEnum, TerrainChunkId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct BiomeChunkId {
    pub x: i32,
    pub y: i32,
    pub biome: BiomeTypeEnum,
}

impl Default for BiomeChunkId {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            biome: BiomeTypeEnum::DeepOcean,
        }
    }
}

impl BiomeChunkId {
    pub fn from_terrain(terrain: &TerrainChunkId, biome: BiomeTypeEnum) -> Self {
        Self {
            x: terrain.x,
            y: terrain.y,
            biome,
        }
    }
}
