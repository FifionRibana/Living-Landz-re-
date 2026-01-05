use bincode::{Decode, Encode};

use crate::ContourSegmentData;

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct TerritoryChunkData {
    pub organization_id: u64,
    pub segments: Vec<ContourSegmentData>,
}
