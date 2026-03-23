use bincode::{Decode, Encode};

use super::GridCell;
use crate::{BiomeTypeEnum, ShoreType, types::TerrainChunkId};

#[derive(Debug, Default, Copy, Clone, Encode, Decode)]
pub struct CellData {
    pub cell: GridCell,
    pub chunk: TerrainChunkId,
    pub biome: BiomeTypeEnum,
    pub shore_type: ShoreType,
}
