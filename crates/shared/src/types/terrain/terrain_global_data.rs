use bincode::{Decode, Encode};

/// Global terrain texture data, sent once per map (not per-chunk).
/// Contains the biome blend texture and heightmap for the entire continent.
/// Used by the terrain shader via world-space UVs.
#[derive(Debug, Clone, Encode, Decode)]
pub struct TerrainGlobalData {
    /// Map name
    pub name: String,

    /// Biome blend texture dimensions
    pub biome_width: u32,
    pub biome_height: u32,

    /// Biome blend texture in RGBA8 format.
    /// R = primary biome ID * 17, G = secondary biome ID * 17,
    /// B = blend factor, A = 255
    pub biome_values: Vec<u8>,

    /// Heightmap dimensions
    pub heightmap_width: u32,
    pub heightmap_height: u32,

    /// Heightmap in R8 format (0-255 elevation)
    pub heightmap_values: Vec<u8>,

    /// World dimensions in pixels (for UV computation)
    pub world_width: f32,
    pub world_height: f32,

    pub generated_at: u64,
}

impl Default for TerrainGlobalData {
    fn default() -> Self {
        Self {
            name: String::new(),
            biome_width: 1,
            biome_height: 1,
            biome_values: vec![5 * 17, 5 * 17, 0, 255],
            heightmap_width: 1,
            heightmap_height: 1,
            heightmap_values: vec![128],
            world_width: 1.0,
            world_height: 1.0,
            generated_at: 0,
        }
    }
}
