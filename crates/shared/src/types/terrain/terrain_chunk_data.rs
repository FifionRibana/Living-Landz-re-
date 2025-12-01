use bincode::{Decode, Encode};

use crate::{TerrainChunkId, TerrainChunkSdfData, HeightmapChunkData, types::MeshData};

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct TerrainChunkData {
    pub name: String,
    pub id: TerrainChunkId,
    // biomes: Vec<BiomeTriangulation>,
    pub mesh_data: MeshData,
    pub sdf_data: Vec<TerrainChunkSdfData>,
    /// Heightmap chunk data (optional, for ocean rendering)
    pub heightmap_data: Option<HeightmapChunkData>,
    // pub mesh_data: HashMap<String, MeshData>,
    /// Contours du continent (outline noir)
    pub outline: Vec<Vec<[f64; 2]>>,
    pub generated_at: u64
}

impl TerrainChunkData {
    pub fn get_storage_key(&self) -> String {
        format!("{}_{}_{}", &self.name, &self.id.x, &self.id.y)
    }

    #[inline]
    pub fn storage_key(name: &str, id: TerrainChunkId) -> String {
        format!("{}_{}_{}", name, id.x, id.y)
    }
}
