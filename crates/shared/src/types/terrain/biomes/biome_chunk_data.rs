use bincode::{Decode, Encode};

use super::BiomeChunkId;
use crate::types::MeshData;

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct BiomeChunkData {
    pub name: String,
    pub id: BiomeChunkId,
    // biomes: Vec<BiomeTriangulation>,
    pub mesh_data: MeshData,
    // pub mesh_data: HashMap<String, MeshData>,
    /// Contours du continent (outline noir)
    pub outline: Vec<Vec<[f64; 2]>>,
    pub generated_at: u64,
}

impl BiomeChunkData {
    pub fn get_storage_key(&self) -> String {
        format!(
            "{}_{}_{}_{:?}",
            &self.name, &self.id.x, &self.id.y, &self.id.biome
        )
    }

    #[inline]
    pub fn storage_key(name: &str, id: BiomeChunkId) -> String {
        format!("{}_{}_{}_{:?}", name, id.x, id.y, &id.biome)
    }
}
