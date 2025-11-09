use bincode::{Decode, Encode};

use crate::{BuildingType, TerrainChunkId, grid::GridCell};

#[derive(Debug, Clone, Encode, Decode)]
pub struct BuildingData {
    pub id: u64,
    pub building_type: BuildingType,
    pub chunk: TerrainChunkId,
    pub cell: GridCell,

    pub created_at: u64,
}
