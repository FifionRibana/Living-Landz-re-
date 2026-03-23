use bevy::prelude::*;
use image::{ImageBuffer, Luma, Rgba};
use shared::grid::GridConfig;
use shared::{TerrainChunkId, TerrainChunkSdfData, constants};

use super::WorldMaps;
use crate::world::components::TerrainMeshData;

/// Pre-computed global data held in server memory.
/// Source images + parameters for on-demand per-chunk generation.
#[derive(Resource)]
pub struct WorldGlobalState {
    /// Map name
    pub map_name: String,

    /// Source images (biome, heightmap, binary)
    pub maps: Option<WorldMaps>,

    /// Source binary map, flipped vertically (NOT upscaled, ~2MB)
    /// Used for per-chunk local upscale + SDF computation
    pub source_binary_flipped: ImageBuffer<Luma<u8>, Vec<u8>>,

    /// Source lake mask, flipped vertically (NOT upscaled, ~2MB)
    pub source_lake_flipped: ImageBuffer<Luma<u8>, Vec<u8>>,

    /// Chunk grid dimensions
    pub n_chunk_x: i32,
    pub n_chunk_y: i32,

    /// Scale factor
    pub scale: Vec2,

    /// SDF parameters
    pub sdf_resolution: usize,
    pub max_distance: f32,

    /// Grid config for hex cell operations
    pub grid_config: Option<GridConfig>,

    /// Source biome map, flipped vertically (NOT upscaled, ~7MB RGBA)
    /// Used by sample_biome_for_chunk to sample at source resolution
    pub source_biome_flipped_rgba: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl WorldGlobalState {
    /// Generate SDF data and binary mask for a single chunk.
    /// Crops source image with overlap, upscales locally, computes SDF.
    /// Returns (SDF data, binary mask for contour detection).
    pub fn generate_chunk_sdf_and_mask(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> (Vec<TerrainChunkSdfData>, ImageBuffer<Luma<u8>, Vec<u8>>) {
        let res = self.sdf_resolution;
        let chunk_w = constants::CHUNK_SIZE.x;
        let chunk_h = constants::CHUNK_SIZE.y;
        let world_w = self.n_chunk_x as f32 * chunk_w;
        let world_h = self.n_chunk_y as f32 * chunk_h;

        // Chunk world bounds
        let chunk_x = chunk_id.x as f32 * chunk_w;
        let chunk_y = chunk_id.y as f32 * chunk_h;

        // Extended bounds with overlap = max_distance
        let ext_x_min = (chunk_x - self.max_distance).max(0.0);
        let ext_y_min = (chunk_y - self.max_distance).max(0.0);
        let ext_x_max = (chunk_x + chunk_w + self.max_distance).min(world_w);
        let ext_y_max = (chunk_y + chunk_h + self.max_distance).min(world_h);

        // Map to source image coordinates (divide by scale)
        let src_w = self.source_binary_flipped.width();
        let src_h = self.source_binary_flipped.height();

        let src_x_min = ((ext_x_min / self.scale.x).floor() as u32).min(src_w);
        let src_y_min = ((ext_y_min / self.scale.y).floor() as u32).min(src_h);
        let src_x_max = ((ext_x_max / self.scale.x).ceil() as u32).min(src_w);
        let src_y_max = ((ext_y_max / self.scale.y).ceil() as u32).min(src_h);

        let crop_w = src_x_max - src_x_min;
        let crop_h = src_y_max - src_y_min;

        if crop_w == 0 || crop_h == 0 {
            // Chunk is entirely outside the map
            let mut data = TerrainChunkSdfData::new(res as u8);
            data.values = vec![0u8; res * res]; // all water
            let mask = ImageBuffer::new(chunk_w as u32, chunk_h as u32);
            return (vec![data], mask);
        }

        // Crop source image
        let crop: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(crop_w, crop_h, |x, y| {
            *self
                .source_binary_flipped
                .get_pixel(src_x_min + x, src_y_min + y)
        });

        // Upscale + threshold (same as global pipeline)
        let upscaled = TerrainMeshData::resize_image(&crop, &self.scale, 178);

        // The upscaled crop covers ext_x_min..ext_x_max in world space
        // (approximately — rounding may shift by ±1 pixel)
        let up_origin_x = src_x_min as f32 * self.scale.x;
        let up_origin_y = src_y_min as f32 * self.scale.y;

        // Compute SDF on the full upscaled crop
        let crop_world_w = upscaled.width() as f32;
        let crop_world_h = upscaled.height() as f32;

        // SDF resolution proportional to crop size
        let sdf_per_world_x = res as f32 / chunk_w;
        let sdf_per_world_y = res as f32 / chunk_h;
        let local_sdf_w = (crop_world_w * sdf_per_world_x).ceil() as usize;
        let local_sdf_h = (crop_world_h * sdf_per_world_y).ceil() as usize;

        let local_sdf = generate_local_sdf(
            &upscaled,
            local_sdf_w,
            local_sdf_h,
            crop_world_w,
            crop_world_h,
            self.max_distance,
        );

        // Extract chunk's 64×64 SDF from the center of the local SDF
        let chunk_offset_x = chunk_x - up_origin_x;
        let chunk_offset_y = chunk_y - up_origin_y;
        let sdf_start_x = (chunk_offset_x * sdf_per_world_x).round() as usize;
        let sdf_start_y = (chunk_offset_y * sdf_per_world_y).round() as usize;

        let mut chunk_sdf_values = Vec::with_capacity(res * res);
        for sy in 0..res {
            for sx in 0..res {
                let gx = sdf_start_x + sx;
                let gy = sdf_start_y + sy;
                if gx < local_sdf_w && gy < local_sdf_h {
                    chunk_sdf_values.push(local_sdf[gy * local_sdf_w + gx]);
                } else {
                    chunk_sdf_values.push(0); // water outside bounds
                }
            }
        }

        let mut sdf_data = TerrainChunkSdfData::new(res as u8);
        sdf_data.values = chunk_sdf_values;

        // Extract chunk binary mask from upscaled crop (for contour detection)
        let mask_offset_x = chunk_offset_x.round() as u32;
        let mask_offset_y = chunk_offset_y.round() as u32;

        let mask = ImageBuffer::from_fn(chunk_w as u32, chunk_h as u32, |x, y| {
            let gx = mask_offset_x + x;
            let gy = mask_offset_y + y;
            if gx < upscaled.width() && gy < upscaled.height() {
                *upscaled.get_pixel(gx, gy)
            } else {
                Luma([0u8])
            }
        });

        (vec![sdf_data], mask)
    }

    /// Check if a chunk has any land (quick check from source image, no SDF needed)
    pub fn chunk_has_land(&self, chunk_id: &TerrainChunkId) -> bool {
        let chunk_x = chunk_id.x as f32 * constants::CHUNK_SIZE.x;
        let chunk_y = chunk_id.y as f32 * constants::CHUNK_SIZE.y;

        // Sample a few points in the source image
        let src_cx = ((chunk_x + constants::CHUNK_SIZE.x * 0.5) / self.scale.x) as u32;
        let src_cy = ((chunk_y + constants::CHUNK_SIZE.y * 0.5) / self.scale.y) as u32;

        let src_w = self.source_binary_flipped.width();
        let src_h = self.source_binary_flipped.height();

        // Check center and corners — if any pixel has land, the chunk might have land
        let offsets = [
            (0.5, 0.5),
            (0.1, 0.1),
            (0.9, 0.1),
            (0.1, 0.9),
            (0.9, 0.9),
            (0.3, 0.3),
            (0.7, 0.7),
            (0.3, 0.7),
            (0.7, 0.3),
        ];

        for (fx, fy) in offsets {
            let px = ((chunk_x + constants::CHUNK_SIZE.x * fx) / self.scale.x) as u32;
            let py = ((chunk_y + constants::CHUNK_SIZE.y * fy) / self.scale.y) as u32;
            if px < src_w && py < src_h && self.source_binary_flipped.get_pixel(px, py)[0] > 30 {
                return true;
            }
        }

        false
    }
}

/// Compute SDF on a local image crop. Same algorithm as generate_global_sdf
/// but operates on a local patch.
fn generate_local_sdf(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    sdf_width: usize,
    sdf_height: usize,
    world_width: f32,
    world_height: f32,
    max_distance: f32,
) -> Vec<u8> {
    use rayon::prelude::*;

    let img_width = image.width() as f32;
    let img_height = image.height() as f32;

    let sdf_to_img_x = img_width / sdf_width as f32;
    let sdf_to_img_y = img_height / sdf_height as f32;

    let world_to_img_x = img_width / world_width;
    let world_to_img_y = img_height / world_height;

    let search_radius = ((max_distance * world_to_img_x.max(world_to_img_y)) as i32).max(1);

    let total_pixels = sdf_width * sdf_height;

    (0..total_pixels)
        .into_par_iter()
        .map(|idx| {
            let sx = idx % sdf_width;
            let sy = idx / sdf_width;

            let img_x = ((sx as f32 + 0.5) * sdf_to_img_x) as i32;
            let img_y = ((sy as f32 + 0.5) * sdf_to_img_y) as i32;

            let current_is_land = if img_x >= 0
                && img_x < image.width() as i32
                && img_y >= 0
                && img_y < image.height() as i32
            {
                image.get_pixel(img_x as u32, img_y as u32)[0] > 30
            } else {
                false
            };

            let mut min_dist_sq = i32::MAX;

            for dy in -search_radius..=search_radius {
                for dx in -search_radius..=search_radius {
                    let nx = img_x + dx;
                    let ny = img_y + dy;

                    if nx < 0
                        || nx >= image.width() as i32
                        || ny < 0
                        || ny >= image.height() as i32
                    {
                        continue;
                    }

                    let neighbor_is_land = image.get_pixel(nx as u32, ny as u32)[0] > 30;

                    if neighbor_is_land != current_is_land {
                        let dist_sq = dx * dx + dy * dy;
                        min_dist_sq = min_dist_sq.min(dist_sq);
                    }
                }
            }

            let min_dist_pixels = (min_dist_sq as f32).sqrt();
            let min_dist_world = min_dist_pixels / world_to_img_x.max(world_to_img_y);

            let min_dist = if min_dist_sq == i32::MAX {
                max_distance
            } else {
                min_dist_world.min(max_distance)
            };

            let signed_dist = if current_is_land {
                min_dist
            } else {
                -min_dist
            };

            let normalized = (signed_dist / max_distance).clamp(-1.0, 1.0);
            ((normalized + 1.0) * 0.5 * 255.0) as u8
        })
        .collect()
}