use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BiomeTriangulation {
    /// Identifiant du biome (couleur RGB)
    biome_color: (u8, u8, u8),
    /// Points 2D des vertices
    points: Vec<[f64; 2]>,
    /// Indices pour former les triangles
    indices: Vec<u32>,
}
