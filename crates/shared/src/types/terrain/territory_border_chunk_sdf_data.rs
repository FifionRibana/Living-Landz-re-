use bincode::{Decode, Encode};

/// SDF data for territory borders in a chunk
/// Similar to TerrainChunkSdfData but for organization borders
#[derive(Debug, Clone, Encode, Decode)]
pub struct TerritoryBorderChunkSdfData {
    /// Chunk coordinates
    pub chunk_x: i32,
    pub chunk_y: i32,

    /// SDF texture dimensions
    pub width: u32,
    pub height: u32,

    /// SDF data: R8 format (distance to nearest border)
    /// Value 0 = on border, 255 = far from border
    pub sdf_data: Vec<u8>,

    /// Organization ID for this territory border
    pub organization_id: u64,

    /// Border color (RGBA, 0.0-1.0 range)
    pub border_color: (f32, f32, f32, f32),

    /// Fill color (RGBA, 0.0-1.0 range)
    pub fill_color: (f32, f32, f32, f32),
}

impl TerritoryBorderChunkSdfData {
    pub fn new(
        chunk_x: i32,
        chunk_y: i32,
        width: u32,
        height: u32,
        organization_id: u64,
        border_color: (f32, f32, f32, f32),
        fill_color: (f32, f32, f32, f32),
    ) -> Self {
        Self {
            chunk_x,
            chunk_y,
            width,
            height,
            sdf_data: vec![255; (width * height) as usize], // Default: no borders
            organization_id,
            border_color,
            fill_color,
        }
    }

    /// Get the SDF value at a specific pixel
    pub fn get_distance(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 255;
        }
        self.sdf_data[(y * self.width + x) as usize]
    }

    /// Set the SDF value at a specific pixel
    pub fn set_distance(&mut self, x: u32, y: u32, distance: u8) {
        if x < self.width && y < self.height {
            self.sdf_data[(y * self.width + x) as usize] = distance;
        }
    }
}
