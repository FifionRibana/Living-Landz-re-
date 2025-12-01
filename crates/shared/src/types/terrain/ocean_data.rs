use bincode::{Decode, Encode};

/// Données globales de l'océan (SDF + heightmap)
/// Utilisées pour le rendu de l'océan autour du continent
#[derive(Debug, Clone, Encode, Decode)]
pub struct OceanData {
    /// Nom du monde
    pub name: String,

    /// Largeur de la texture SDF/heightmap
    pub width: usize,

    /// Hauteur de la texture SDF/heightmap
    pub height: usize,

    /// Distance maximale du SDF (en unités monde)
    pub max_distance: f32,

    /// Données du SDF signé (0 = loin dans l'eau, 128 = côte, 255 = loin sur terre)
    /// Format: Vec<u8> de taille width * height
    pub sdf_values: Vec<u8>,

    /// Données de heightmap (élévation du terrain)
    /// Format: Vec<u8> de taille width * height (normalisé 0-255)
    pub heightmap_values: Vec<u8>,

    /// Timestamp de génération
    pub generated_at: u64,
}

impl OceanData {
    pub fn new(
        name: String,
        width: usize,
        height: usize,
        max_distance: f32,
        sdf_values: Vec<u8>,
        heightmap_values: Vec<u8>,
    ) -> Self {
        assert_eq!(sdf_values.len(), width * height, "SDF size mismatch");
        assert_eq!(heightmap_values.len(), width * height, "Heightmap size mismatch");

        Self {
            name,
            width,
            height,
            max_distance,
            sdf_values,
            heightmap_values,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Découpe le SDF en chunks avec overlap
    pub fn split_sdf_into_chunks(
        &self,
        chunk_resolution: usize,
        n_chunk_x: i32,
        n_chunk_y: i32,
    ) -> std::collections::HashMap<crate::TerrainChunkId, Vec<u8>> {
        split_texture_into_chunks(
            &self.sdf_values,
            self.width,
            self.height,
            chunk_resolution,
            n_chunk_x,
            n_chunk_y,
        )
    }

    /// Découpe la heightmap en chunks avec overlap
    pub fn split_heightmap_into_chunks(
        &self,
        chunk_resolution: usize,
        n_chunk_x: i32,
        n_chunk_y: i32,
    ) -> std::collections::HashMap<crate::TerrainChunkId, Vec<u8>> {
        split_texture_into_chunks(
            &self.heightmap_values,
            self.width,
            self.height,
            chunk_resolution,
            n_chunk_x,
            n_chunk_y,
        )
    }
}

/// Découpe une texture globale en chunks avec overlap
/// Réutilisable pour SDF et heightmap
fn split_texture_into_chunks(
    global_texture: &[u8],
    global_width: usize,
    global_height: usize,
    chunk_resolution: usize,
    n_chunk_x: i32,
    n_chunk_y: i32,
) -> std::collections::HashMap<crate::TerrainChunkId, Vec<u8>> {
    use std::collections::HashMap;

    let mut result = HashMap::new();

    // Overlap en texels (0.5 de chaque côté = 1 texel total d'extension)
    let overlap = 0.5f32;

    for cy in 0..n_chunk_y {
        for cx in 0..n_chunk_x {
            let chunk_id = crate::TerrainChunkId { x: cx, y: cy };

            let mut chunk_values = Vec::with_capacity(chunk_resolution * chunk_resolution);

            let base_x = cx as f32 * chunk_resolution as f32;
            let base_y = cy as f32 * chunk_resolution as f32;

            for sy in 0..chunk_resolution {
                for sx in 0..chunk_resolution {
                    // Mapper [0, resolution-1] → [-0.5, resolution-0.5] dans la texture globale
                    let t_x = sx as f32 / (chunk_resolution - 1) as f32; // 0 à 1
                    let t_y = sy as f32 / (chunk_resolution - 1) as f32;

                    // Position avec overlap symétrique
                    let global_x =
                        base_x - overlap + t_x * (chunk_resolution as f32 - 1.0 + 2.0 * overlap);
                    let global_y =
                        base_y - overlap + t_y * (chunk_resolution as f32 - 1.0 + 2.0 * overlap);

                    let value = sample_texture_bilinear(
                        global_texture,
                        global_width,
                        global_height,
                        global_x,
                        global_y,
                    );

                    chunk_values.push(value);
                }
            }

            result.insert(chunk_id, chunk_values);
        }
    }

    result
}

/// Sample une texture avec interpolation bilinéaire
fn sample_texture_bilinear(texture: &[u8], width: usize, height: usize, x: f32, y: f32) -> u8 {
    let x0 = (x.floor() as i32).clamp(0, width as i32 - 1) as usize;
    let y0 = (y.floor() as i32).clamp(0, height as i32 - 1) as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let v00 = texture[y0 * width + x0] as f32;
    let v10 = texture[y0 * width + x1] as f32;
    let v01 = texture[y1 * width + x0] as f32;
    let v11 = texture[y1 * width + x1] as f32;

    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;
    let v = v0 * (1.0 - fy) + v1 * fy;

    v.round().clamp(0.0, 255.0) as u8
}
