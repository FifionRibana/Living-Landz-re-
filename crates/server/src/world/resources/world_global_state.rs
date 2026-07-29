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

    /// Enriched heightmap (u16 LE bytes, flipped to match Bevy Y-up).
    /// Used for ShoreType classification based on actual terrain height.
    pub enriched_heightmap: Option<Vec<u8>>,
    pub enriched_heightmap_width: u32,
    pub enriched_heightmap_height: u32,

    /// Effective binary map derived from enriched heightmap (0=water, 255=land).
    /// Same orientation as source_binary_flipped (Bevy Y-up).
    /// Sharp edges — used for gameplay (biome, ShoreType, chunk_has_land).
    pub effective_binary: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,

    /// Smoothed effective binary for rendering SDFs (chunk, ocean, lake).
    /// Gaussian blur on the global image rounds pixel staircases into curves.
    /// No chunk-junction issues since the blur is applied globally before cropping.
    pub effective_binary_smoothed: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,

    /// Normalized sea level in the enriched heightmap (0.0..1.0) — the land/sea
    /// threshold. `0.0` = legacy Azgaar convention (ocean is exactly `0u16`);
    /// `0.5` = Ymir full-range encoding (sea level at u16 ≈ 32768).
    pub water_threshold_norm: f32,

    /// Ymir sea-level coastline polylines in **cell space** (y=0 = south), used to
    /// build the coastal SDF from vectors instead of a smoothed pixel mask (LL-B).
    /// Empty on the Azgaar path (and on Ymir maps without a coastline layer).
    pub coastline_cells: Vec<Vec<[f32; 2]>>,

    /// Ymir resolved per-cell biome ids (`BiomeTypeEnum::to_id() as u8`), grid =
    /// heightmap dims, Y-flipped to match `source_biome_flipped_rgba` (LL-C).
    /// `None` on the Azgaar path or Ymir maps without a `biome.u8` layer, in which
    /// case per-chunk biome falls back to `find_closest_biome`.
    pub ymir_biome_ids: Option<Vec<u8>>,
}

impl WorldGlobalState {
    /// Generate the effective binary map from the enriched heightmap.
    /// Must be called after enriched_heightmap is populated.
    ///
    /// `smoothed` controls whether the Gaussian-blurred `effective_binary_smoothed`
    /// is produced for render SDFs. The Azgaar path passes `true`; the Ymir path
    /// passes `false` (its coast comes from the vector coastline SDF, so the σ=5
    /// mask blur + re-threshold are skipped — LL-B).
    pub fn build_effective_binary(&mut self, smoothed: bool) {
        let Some(ref hm_data) = self.enriched_heightmap else {
            return;
        };
        let w = self.enriched_heightmap_width as usize;
        let h = self.enriched_heightmap_height as usize;
        let expected_u16_len = w * h * 2;
        let is_u16 = hm_data.len() >= expected_u16_len;

        // Land = normalized height strictly above sea level. For the legacy
        // Azgaar convention `water_threshold_norm == 0.0`, so this reduces to the
        // original "> 0" test; for Ymir full-range it is sea_level_norm (~0.5).
        let threshold = self.water_threshold_norm;
        let mut effective = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32);
        for y in 0..h {
            for x in 0..w {
                let norm = if is_u16 {
                    let idx = (y * w + x) * 2;
                    u16::from_le_bytes([hm_data[idx], hm_data[idx + 1]]) as f32 / 65535.0
                } else {
                    hm_data[y * w + x] as f32 / 255.0
                };
                let is_land = norm > threshold;
                effective.put_pixel(x as u32, y as u32, Luma([if is_land { 255u8 } else { 0u8 }]));
            }
        }
        let land_count = effective.pixels().filter(|p| p[0] > 128).count();
        let total = w * h;
        tracing::info!(
            "✓ Effective binary map generated from enriched heightmap ({}x{}): {} land pixels ({:.1}%)",
            w, h, land_count, land_count as f64 / total as f64 * 100.0
        );
        // Build smoothed version for rendering SDFs (organic coastlines).
        // Applied globally — no chunk-junction issues since both chunks crop
        // from the same blurred image. Skipped on the Ymir path (LL-B): its
        // coastline comes from the vector SDF, not a blurred pixel mask.
        if smoothed {
            let sigma = 5.0f32;
            let radius = (sigma * 2.5).ceil() as i32;
            let ksize = (2 * radius + 1) as usize;
            let mut kernel = vec![0.0f32; ksize];
            let mut ksum = 0.0f32;
            for i in 0..ksize {
                let xf = (i as i32 - radius) as f32;
                kernel[i] = (-xf * xf / (2.0 * sigma * sigma)).exp();
                ksum += kernel[i];
            }
            for v in kernel.iter_mut() { *v /= ksum; }

            let mut tmp = vec![0.0f32; w * h];
            for y in 0..h {
                for x in 0..w {
                    let mut s = 0.0f32;
                    for k in -radius..=radius {
                        let sx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                        s += (effective.get_pixel(sx as u32, y as u32)[0] as f32 / 255.0)
                            * kernel[(k + radius) as usize];
                    }
                    tmp[y * w + x] = s;
                }
            }
            let mut smoothed_img = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32);
            for y in 0..h {
                for x in 0..w {
                    let mut s = 0.0f32;
                    for k in -radius..=radius {
                        let sy = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                        s += tmp[sy * w + x] * kernel[(k + radius) as usize];
                    }
                    smoothed_img.put_pixel(x as u32, y as u32, Luma([if s > 0.5 { 255u8 } else { 0u8 }]));
                }
            }
            tracing::info!("✓ Smoothed effective binary for render (sigma={}, {}x{})", sigma, w, h);
            self.effective_binary_smoothed = Some(smoothed_img);
        } else {
            self.effective_binary_smoothed = None;
        }

        self.effective_binary = Some(effective);
    }

    /// Generate SDF data and binary mask for a single chunk.
    /// Uses effective_binary (from enriched heightmap) when available,
    /// falls back to source_binary_flipped with upscale.
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

        // Extended bounds with overlap = max_distance.
        // Snap to chunk grid so that two adjacent chunks share identical SDF
        // values at their boundary (prevents seam/flicker artifacts).
        let overlap_chunks_x = (self.max_distance / chunk_w).ceil();
        let overlap_chunks_y = (self.max_distance / chunk_h).ceil();
        let ext_chunk_x_min = (chunk_id.x as f32 - overlap_chunks_x).max(0.0) as i32;
        let ext_chunk_y_min = (chunk_id.y as f32 - overlap_chunks_y).max(0.0) as i32;
        let ext_chunk_x_max = ((chunk_id.x + 1) as f32 + overlap_chunks_x).min(self.n_chunk_x as f32) as i32;
        let ext_chunk_y_max = ((chunk_id.y + 1) as f32 + overlap_chunks_y).min(self.n_chunk_y as f32) as i32;

        let ext_x_min = ext_chunk_x_min as f32 * chunk_w;
        let ext_y_min = ext_chunk_y_min as f32 * chunk_h;
        let ext_x_max = ext_chunk_x_max as f32 * chunk_w;
        let ext_y_max = ext_chunk_y_max as f32 * chunk_h;

        // Choose binary source: smoothed for rendering, sharp fallback, legacy last
        let (binary_img, pix_scale_x, pix_scale_y) = if let Some(ref eff) = self.effective_binary_smoothed {
            let sx = world_w / eff.width() as f32;
            let sy = world_h / eff.height() as f32;
            (eff as &ImageBuffer<Luma<u8>, Vec<u8>>, sx, sy)
        } else if let Some(ref eff) = self.effective_binary {
            let sx = world_w / eff.width() as f32;
            let sy = world_h / eff.height() as f32;
            (eff as &ImageBuffer<Luma<u8>, Vec<u8>>, sx, sy)
        } else {
            (&self.source_binary_flipped, self.scale.x, self.scale.y)
        };

        let img_w = binary_img.width();
        let img_h = binary_img.height();

        let src_x_min = ((ext_x_min / pix_scale_x).floor() as u32).min(img_w);
        let src_y_min = ((ext_y_min / pix_scale_y).floor() as u32).min(img_h);
        let src_x_max = ((ext_x_max / pix_scale_x).ceil() as u32).min(img_w);
        let src_y_max = ((ext_y_max / pix_scale_y).ceil() as u32).min(img_h);

        let crop_w = src_x_max - src_x_min;
        let crop_h = src_y_max - src_y_min;

        if crop_w == 0 || crop_h == 0 {
            let mut data = TerrainChunkSdfData::new(res as u8);
            data.values = vec![0u8; res * res];
            let mask = ImageBuffer::new(chunk_w as u32, chunk_h as u32);
            return (vec![data], mask);
        }

        // Crop binary image
        let crop: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(crop_w, crop_h, |x, y| {
            *binary_img.get_pixel(src_x_min + x, src_y_min + y)
        });

        // For effective_binary (already high-res and clean 0/255), use directly as
        // "upscaled" at 1 pixel = 1 world unit. For legacy source, upscale + threshold.
        let (upscaled, up_origin_x, up_origin_y) = if self.effective_binary.is_some() {
            // Effective binary: 1 pixel ≈ pix_scale world units.
            // The crop is already at the right resolution.
            let origin_x = src_x_min as f32 * pix_scale_x;
            let origin_y = src_y_min as f32 * pix_scale_y;
            (crop, origin_x, origin_y)
        } else {
            let upscaled = TerrainMeshData::resize_image(&crop, &self.scale, 178);
            let origin_x = src_x_min as f32 * self.scale.x;
            let origin_y = src_y_min as f32 * self.scale.y;
            (upscaled, origin_x, origin_y)
        };

        // Compute SDF on the binary crop.
        // crop_world_w/h must be in WORLD units, not pixels.
        // For effective_binary: pixels * pix_scale = world units.
        // For legacy upscaled: 1 pixel = 1 world unit (resize_image applies scale).
        let crop_world_w = if self.effective_binary.is_some() {
            upscaled.width() as f32 * pix_scale_x
        } else {
            upscaled.width() as f32
        };
        let crop_world_h = if self.effective_binary.is_some() {
            upscaled.height() as f32 * pix_scale_y
        } else {
            upscaled.height() as f32
        };

        // Local SDF is exactly (ext_chunks_w * res) × (ext_chunks_h * res)
        let ext_chunks_w = (ext_chunk_x_max - ext_chunk_x_min) as usize;
        let ext_chunks_h = (ext_chunk_y_max - ext_chunk_y_min) as usize;
        let local_sdf_w = ext_chunks_w * res;
        let local_sdf_h = ext_chunks_h * res;

        let local_sdf = generate_local_sdf(
            &upscaled,
            local_sdf_w,
            local_sdf_h,
            crop_world_w,
            crop_world_h,
            self.max_distance,
        );

        // Extract chunk's 64×64 SDF from the local SDF.
        // Because we snapped to chunk grid, the offset is exactly N chunks * res pixels.
        let sdf_start_x = (chunk_id.x - ext_chunk_x_min) as usize * res;
        let sdf_start_y = (chunk_id.y - ext_chunk_y_min) as usize * res;

        let mut chunk_sdf_values = Vec::with_capacity(res * res);
        for sy in 0..res {
            for sx in 0..res {
                let gx = sdf_start_x + sx;
                let gy = sdf_start_y + sy;
                if gx < local_sdf_w && gy < local_sdf_h {
                    chunk_sdf_values.push(local_sdf[gy * local_sdf_w + gx]);
                } else {
                    chunk_sdf_values.push(0);
                }
            }
        }

        let mut sdf_data = TerrainChunkSdfData::new(res as u8);
        sdf_data.values = chunk_sdf_values;

        // Extract chunk binary mask from crop (for contour detection).
        let chunk_offset_x = chunk_x - (ext_chunk_x_min as f32 * chunk_w);
        let chunk_offset_y = chunk_y - (ext_chunk_y_min as f32 * chunk_h);

        let mask = if self.effective_binary_smoothed.is_some() || self.effective_binary.is_some() {
            let pix_off_x = (chunk_offset_x / pix_scale_x).round() as u32;
            let pix_off_y = (chunk_offset_y / pix_scale_y).round() as u32;
            let pix_per_chunk_w = (chunk_w / pix_scale_x).ceil() as u32;
            let pix_per_chunk_h = (chunk_h / pix_scale_y).ceil() as u32;
            ImageBuffer::from_fn(chunk_w as u32, chunk_h as u32, |x, y| {
                let sx = pix_off_x + (x * pix_per_chunk_w / chunk_w as u32);
                let sy = pix_off_y + (y * pix_per_chunk_h / chunk_h as u32);
                if sx < upscaled.width() && sy < upscaled.height() {
                    *upscaled.get_pixel(sx, sy)
                } else {
                    Luma([0u8])
                }
            })
        } else {
            let mask_offset_x = chunk_offset_x.round() as u32;
            let mask_offset_y = chunk_offset_y.round() as u32;
            ImageBuffer::from_fn(chunk_w as u32, chunk_h as u32, |x, y| {
                let gx = mask_offset_x + x;
                let gy = mask_offset_y + y;
                if gx < upscaled.width() && gy < upscaled.height() {
                    *upscaled.get_pixel(gx, gy)
                } else {
                    Luma([0u8])
                }
            })
        };

        (vec![sdf_data], mask)
    }

    /// Check if a chunk has any land (quick sample from effective binary or source image)
    pub fn chunk_has_land(&self, chunk_id: &TerrainChunkId) -> bool {
        let chunk_x = chunk_id.x as f32 * constants::CHUNK_SIZE.x;
        let chunk_y = chunk_id.y as f32 * constants::CHUNK_SIZE.y;
        let world_w = self.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
        let world_h = self.n_chunk_y as f32 * constants::CHUNK_SIZE.y;

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

        if let Some(ref eff) = self.effective_binary {
            let eff_w = eff.width();
            let eff_h = eff.height();
            for (fx, fy) in offsets {
                let wx = chunk_x + constants::CHUNK_SIZE.x * fx;
                let wy = chunk_y + constants::CHUNK_SIZE.y * fy;
                let px = (wx / world_w * eff_w as f32) as u32;
                let py = (wy / world_h * eff_h as f32) as u32;
                if px < eff_w && py < eff_h && eff.get_pixel(px, py)[0] > 128 {
                    return true;
                }
            }
            false
        } else {
            // Fallback: source binary map
            let src_w = self.source_binary_flipped.width();
            let src_h = self.source_binary_flipped.height();
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