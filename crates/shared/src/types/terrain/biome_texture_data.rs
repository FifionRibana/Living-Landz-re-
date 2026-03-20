use bincode::{Decode, Encode};

/// Biome blend texture data for a chunk.
/// RGBA8 format, pre-computed on server:
///   R = primary biome ID (stored as id * 17, so 0-15 maps to 0-255)
///   G = secondary biome ID (nearest different biome)
///   B = blend factor (0 = 100% primary, 255 = 100% secondary)
///   A = reserved (255)
///
/// The blend factor is computed from distance to the nearest biome boundary,
/// giving perfectly smooth, continuous transitions.
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct BiomeTextureData {
    /// Grid resolution (e.g. 64 means 64x64)
    pub resolution: u16,

    /// RGBA8 data in row-major order: index = (y * resolution + x) * 4
    /// 4 bytes per pixel: [primary_id_scaled, secondary_id_scaled, blend, 255]
    pub values: Vec<u8>,
}

impl BiomeTextureData {
    pub fn new(resolution: u16) -> Self {
        let size = (resolution as usize) * (resolution as usize) * 4;
        Self {
            resolution,
            values: vec![0; size],
        }
    }

    pub fn from_values(resolution: u16, values: Vec<u8>) -> Self {
        assert_eq!(
            values.len(),
            (resolution as usize) * (resolution as usize) * 4,
            "Biome texture values size mismatch (expected RGBA8)"
        );
        Self { resolution, values }
    }
}