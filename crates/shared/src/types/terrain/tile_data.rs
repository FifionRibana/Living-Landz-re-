use bincode::prelude::{Decode, Encode};
use crate::BiomeTypeEnum;

#[derive(Debug, Clone, Encode, Decode)]
pub struct TileData {
    pub coord: [i32; 2],
    pub chunk_id: [i32; 2],
    pub biome: BiomeTypeEnum,
}
