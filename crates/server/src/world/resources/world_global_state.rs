use bevy::prelude::*;
use image::{ImageBuffer, Luma};
use shared::grid::GridConfig;
use shared::{TerrainChunkId, TerrainChunkSdfData, constants};
use std::collections::HashMap;

use super::WorldMaps;

/// Pre-computed global data held in server memory.
/// Generated once at startup, used for on-demand per-chunk generation.
#[derive(Resource)]
pub struct WorldGlobalState {
    /// Map name
    pub map_name: String,

    /// Source images (biome, heightmap, binary)
    pub maps: Option<WorldMaps>,

    /// Upscaled binary map (for contour detection per chunk)
    pub scaled_binary_map: ImageBuffer<Luma<u8>, Vec<u8>>,

    /// Global SDF values (flat array, row-major)
    pub global_sdf: Vec<u8>,
    pub sdf_resolution: usize,
    pub global_sdf_width: usize,
    pub global_sdf_height: usize,

    /// Chunk grid dimensions
    pub n_chunk_x: i32,
    pub n_chunk_y: i32,

    /// Scale factor
    pub scale: Vec2,

    /// Grid config for hex cell operations
    pub grid_config: Option<GridConfig>,
}

impl WorldGlobalState {
    /// Extract SDF data for a single chunk from the global SDF buffer
    pub fn extract_chunk_sdf(&self, chunk_id: &TerrainChunkId) -> Vec<TerrainChunkSdfData> {
        let res = self.sdf_resolution;
        let overlap = 0.5f32;

        let base_x = chunk_id.x as f32 * res as f32;
        let base_y = chunk_id.y as f32 * res as f32;

        let mut sdf_values = Vec::with_capacity(res * res);

        for sy in 0..res {
            for sx in 0..res {
                let t_x = sx as f32 / (res - 1) as f32;
                let t_y = sy as f32 / (res - 1) as f32;

                let global_x =
                    base_x - overlap + t_x * (res as f32 - 1.0 + 2.0 * overlap);
                let global_y =
                    base_y - overlap + t_y * (res as f32 - 1.0 + 2.0 * overlap);

                let value = sample_bilinear(
                    &self.global_sdf,
                    self.global_sdf_width,
                    self.global_sdf_height,
                    global_x,
                    global_y,
                );

                sdf_values.push(value);
            }
        }

        let mut data = TerrainChunkSdfData::new(res as u8);
        data.values = sdf_values;
        vec![data]
    }

    /// Crop the scaled binary map for a single chunk
    pub fn crop_chunk_binary_mask(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let x_offset = (chunk_id.x * constants::CHUNK_SIZE.x as i32) as u32;
        let y_offset = (chunk_id.y * constants::CHUNK_SIZE.y as i32) as u32;

        ImageBuffer::from_fn(
            constants::CHUNK_SIZE.x as u32,
            constants::CHUNK_SIZE.y as u32,
            |px, py| {
                let gx = x_offset + px;
                let gy = y_offset + py;
                if gx < self.scaled_binary_map.width() && gy < self.scaled_binary_map.height() {
                    *self.scaled_binary_map.get_pixel(gx, gy)
                } else {
                    Luma([0u8])
                }
            },
        )
    }

    /// Check if a chunk has any land based on SDF values
    pub fn chunk_has_land(&self, chunk_id: &TerrainChunkId) -> bool {
        let sdf = self.extract_chunk_sdf(chunk_id);
        sdf.first()
            .map(|s| s.values.iter().any(|&v| v > 128))
            .unwrap_or(false)
    }
}

fn sample_bilinear(data: &[u8], width: usize, height: usize, x: f32, y: f32) -> u8 {
    let x0 = (x.floor() as i32).clamp(0, width as i32 - 1) as usize;
    let y0 = (y.floor() as i32).clamp(0, height as i32 - 1) as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let v00 = data[y0 * width + x0] as f32;
    let v10 = data[y0 * width + x1] as f32;
    let v01 = data[y1 * width + x0] as f32;
    let v11 = data[y1 * width + x1] as f32;

    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;
    let v = v0 * (1.0 - fy) + v1 * fy;

    v.round().clamp(0.0, 255.0) as u8
}