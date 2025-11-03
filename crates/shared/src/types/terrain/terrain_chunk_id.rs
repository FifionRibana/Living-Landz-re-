use bincode::{Encode, Decode};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TerrainChunkId {
    pub x: i32,
    pub y: i32
}