use bincode::{Decode, Encode};

/// Lake data — mask + SDF, sent to client for lake rendering
#[derive(Debug, Clone, Encode, Decode)]
pub struct LakeData {
    pub name: String,
    /// Mask texture dimensions
    pub width: usize,
    pub height: usize,
    /// Lake mask values (0 = no lake, 255 = lake)
    pub mask_values: Vec<u8>,
    /// Lake SDF dimensions (may differ from mask)
    pub sdf_width: usize,
    pub sdf_height: usize,
    /// Lake SDF values (0 = deep in lake, 128 = shore, 255 = deep on land)
    /// Same convention as ocean SDF
    pub sdf_values: Vec<u8>,
    pub world_width: f32,
    pub world_height: f32,
    pub generated_at: u64,
}