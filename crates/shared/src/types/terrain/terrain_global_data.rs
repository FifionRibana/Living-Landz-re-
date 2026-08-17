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

    /// Heightmap in R16 format (u16 LE bytes, 0-65535 elevation)
    pub heightmap_values: Vec<u8>,

    /// World dimensions in pixels (for UV computation)
    pub world_width: f32,
    pub world_height: f32,

    /// World units per source grid cell (the biome/heightmap grid `scale`). The
    /// cell grid covers `[0, heightmap_dim * world_units_per_cell]`, which is
    /// slightly less than `world_*` (chunk rounding). Lets consumers place things
    /// on the *cell* grid (e.g. the river render) so they line up with the hex
    /// cells rather than the texture's `world_*` mapping. `0.0` = unknown/legacy.
    pub world_units_per_cell: f32,

    /// Ground-plane metres per source grid cell (0.0 = unknown / legacy Azgaar).
    pub metres_per_cell: f32,
    /// Metres represented by one R16 height step (0.0 = unknown / legacy Azgaar).
    pub metres_per_height_unit: f32,
    /// Absolute metres at R16 value 0 (offset for `metres = height_min_m + u16 * metres_per_height_unit`).
    pub height_min_m: f32,
    /// Normalized sea level in the R16 range, i.e. the land/sea threshold.
    /// 0.0 = legacy convention (ocean is exactly 0); 0.5 = Ymir full-range (sea level mid-range).
    pub sea_level_norm: f32,

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
            heightmap_values: vec![0, 128], // single u16 LE pixel
            world_width: 1.0,
            world_height: 1.0,
            world_units_per_cell: 0.0,
            metres_per_cell: 0.0,
            metres_per_height_unit: 0.0,
            height_min_m: 0.0,
            sea_level_norm: 0.0,
            generated_at: 0,
        }
    }
}
