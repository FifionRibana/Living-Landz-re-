use bincode::{Decode, Encode};

use super::GridCell;
use crate::{BiomeType, types::TerrainChunkId};

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct CellData {
    pub cell: GridCell,
    pub chunk: TerrainChunkId,
    pub biome: BiomeType,
}
