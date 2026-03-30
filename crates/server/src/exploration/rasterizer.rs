use crate::exploration::{ExplorationSeed, EXPLORATION_SEED_SPACING};
use std::collections::HashSet;

/// Resolution multiplier: 32 texels per chunk.
pub const EXPLORATION_RESOLUTION: i32 = 32;

/// Spatial hash for fast nearest-seed lookups during rasterization.
/// Only built from seeds near the area of interest.
pub struct SeedSpatialHash {
    seeds: Vec<ExplorationSeed>,
    buckets: Vec<Vec<usize>>,
    bucket_size: f32,
    grid_w: usize,
    grid_h: usize,
    offset_x: f32,
    offset_y: f32,
}

impl SeedSpatialHash {
    /// Build a spatial hash for seeds within a local region.
    /// `offset` shifts coordinates so the hash grid starts near 0.
    pub fn build_local(seeds: Vec<ExplorationSeed>, region_w: f32, region_h: f32, offset_x: f32, offset_y: f32) -> Self {
        let bucket_size = 300.0f32;
        let grid_w = (region_w / bucket_size).ceil() as usize + 3;
        let grid_h = (region_h / bucket_size).ceil() as usize + 3;
        let mut buckets = vec![Vec::new(); grid_w * grid_h];

        for (i, seed) in seeds.iter().enumerate() {
            let bx = ((seed.x - offset_x) / bucket_size).floor() as i32;
            let by = ((seed.y - offset_y) / bucket_size).floor() as i32;
            if bx >= 0 && by >= 0 && (bx as usize) < grid_w && (by as usize) < grid_h {
                buckets[by as usize * grid_w + bx as usize].push(i);
            }
        }

        Self { seeds, buckets, bucket_size, grid_w, grid_h, offset_x, offset_y }
    }

    pub fn nearest_zone(&self, x: f32, y: f32) -> Option<i64> {
        let bx = ((x - self.offset_x) / self.bucket_size).floor() as i32;
        let by = ((y - self.offset_y) / self.bucket_size).floor() as i32;

        let mut best_dist_sq = f32::MAX;
        let mut best_id: Option<i64> = None;

        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                let nx = bx + dx;
                let ny = by + dy;
                if nx < 0 || ny < 0 || nx >= self.grid_w as i32 || ny >= self.grid_h as i32 {
                    continue;
                }
                for &seed_idx in &self.buckets[ny as usize * self.grid_w + nx as usize] {
                    let seed = &self.seeds[seed_idx];
                    let ddx = seed.x - x;
                    let ddy = seed.y - y;
                    let dist_sq = ddx * ddx + ddy * ddy;
                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        best_id = Some(seed.zone_id);
                    }
                }
            }
        }

        best_id
    }
}

/// Rasterize the full exploration texture.
/// Only rasterizes the bounding box around explored seeds — the rest stays 0.
pub fn rasterize_exploration(
    seeds: &[ExplorationSeed],
    explored: &HashSet<i64>,
    n_chunk_x: i32,
    n_chunk_y: i32,
    chunk_w: f32,
    chunk_h: f32,
) -> (i32, i32, Vec<u8>) {
    let tex_w = n_chunk_x * EXPLORATION_RESOLUTION;
    let tex_h = n_chunk_y * EXPLORATION_RESOLUTION;
    let world_w = n_chunk_x as f32 * chunk_w;
    let world_h = n_chunk_y as f32 * chunk_h;
    let texel_w = world_w / tex_w as f32;
    let texel_h = world_h / tex_h as f32;

    let mut data = vec![0u8; (tex_w * tex_h) as usize];

    if explored.is_empty() {
        tracing::info!("No explored zones — returning empty texture {}×{}", tex_w, tex_h);
        return (tex_w, tex_h, data);
    }

    // Find explored seeds and compute bounding box in world coords
    let explored_seeds: Vec<&ExplorationSeed> = seeds.iter()
        .filter(|s| explored.contains(&s.zone_id))
        .collect();

    if explored_seeds.is_empty() {
        tracing::warn!("Explored set has {} IDs but none match computed seeds", explored.len());
        return (tex_w, tex_h, data);
    }

    // Cell margin in world pixels: 2.5× cell diameter covers the full Voronoi cell
    let cell_margin = (EXPLORATION_SEED_SPACING as f32) * 48.0 * 2.5;

    let mut wx_min = f32::MAX;
    let mut wy_min = f32::MAX;
    let mut wx_max = f32::MIN;
    let mut wy_max = f32::MIN;
    for s in &explored_seeds {
        wx_min = wx_min.min(s.x);
        wy_min = wy_min.min(s.y);
        wx_max = wx_max.max(s.x);
        wy_max = wy_max.max(s.y);
    }
    wx_min -= cell_margin;
    wy_min -= cell_margin;
    wx_max += cell_margin;
    wy_max += cell_margin;

    // Texel bounds (clamped to texture)
    let tx_min = ((wx_min / texel_w).floor() as i32).max(0);
    let ty_min = ((wy_min / texel_h).floor() as i32).max(0);
    let tx_max = ((wx_max / texel_w).ceil() as i32).min(tex_w - 1);
    let ty_max = ((wy_max / texel_h).ceil() as i32).min(tex_h - 1);
    let patch_w = tx_max - tx_min + 1;
    let patch_h = ty_max - ty_min + 1;

    tracing::info!(
        "Rasterizing exploration: texture {}×{}, patch {}×{} at ({},{}), {} explored seeds",
        tex_w, tex_h, patch_w, patch_h, tx_min, ty_min, explored_seeds.len()
    );

    let start = std::time::Instant::now();

    // Build spatial hash from seeds near the explored region only
    let nearby_seeds: Vec<ExplorationSeed> = seeds.iter()
        .filter(|s| {
            s.x >= wx_min - cell_margin && s.x <= wx_max + cell_margin &&
            s.y >= wy_min - cell_margin && s.y <= wy_max + cell_margin
        })
        .cloned()
        .collect();

    let region_w = wx_max - wx_min + cell_margin * 2.0;
    let region_h = wy_max - wy_min + cell_margin * 2.0;
    let hash = SeedSpatialHash::build_local(nearby_seeds, region_w, region_h, wx_min - cell_margin, wy_min - cell_margin);

    // Rasterize only the patch
    for ty in ty_min..=ty_max {
        let world_y = (ty as f32 + 0.5) * texel_h;
        for tx in tx_min..=tx_max {
            let world_x = (tx as f32 + 0.5) * texel_w;
            if let Some(zone_id) = hash.nearest_zone(world_x, world_y) {
                if explored.contains(&zone_id) {
                    data[(ty * tex_w + tx) as usize] = 255;
                }
            }
        }
    }

    tracing::info!(
        "Rasterized in {:?} ({} explored texels, {:.0}× speedup from patch)",
        start.elapsed(),
        data.iter().filter(|&&v| v > 0).count(),
        (tex_w as f64 * tex_h as f64) / (patch_w as f64 * patch_h as f64)
    );

    // Apply blur to smooth Voronoi cell boundaries.
    // Radius 5 texels ≈ 95px world ≈ half a Voronoi cell → nice soft transition.
    let blur_margin = BLUR_RADIUS as i32 + 1;
    let blur_x = (tx_min - blur_margin).max(0);
    let blur_y = (ty_min - blur_margin).max(0);
    let blur_ex = (tx_max + blur_margin).min(tex_w - 1);
    let blur_ey = (ty_max + blur_margin).min(tex_h - 1);
    blur_region(&mut data, tex_w, tex_h, blur_x, blur_y, blur_ex - blur_x + 1, blur_ey - blur_y + 1);

    tracing::info!("Blur applied in region {}×{}", blur_ex - blur_x + 1, blur_ey - blur_y + 1);

    (tex_w, tex_h, data)
}

/// Rasterize a rectangular patch of the exploration texture.
pub fn rasterize_patch(
    seeds: &[ExplorationSeed],
    explored: &HashSet<i64>,
    n_chunk_x: i32,
    n_chunk_y: i32,
    chunk_w: f32,
    chunk_h: f32,
    patch_x: i32,
    patch_y: i32,
    patch_w: i32,
    patch_h: i32,
) -> Vec<u8> {
    let full_tex_w = n_chunk_x * EXPLORATION_RESOLUTION;
    let full_tex_h = n_chunk_y * EXPLORATION_RESOLUTION;
    let world_w = n_chunk_x as f32 * chunk_w;
    let world_h = n_chunk_y as f32 * chunk_h;
    let texel_w = world_w / full_tex_w as f32;
    let texel_h = world_h / full_tex_h as f32;

    // World bounds of the patch + margin
    let cell_margin = (EXPLORATION_SEED_SPACING as f32) * 48.0 * 2.5;
    let wx_min = patch_x as f32 * texel_w - cell_margin;
    let wy_min = patch_y as f32 * texel_h - cell_margin;
    let wx_max = (patch_x + patch_w) as f32 * texel_w + cell_margin;
    let wy_max = (patch_y + patch_h) as f32 * texel_h + cell_margin;

    let nearby_seeds: Vec<ExplorationSeed> = seeds.iter()
        .filter(|s| s.x >= wx_min && s.x <= wx_max && s.y >= wy_min && s.y <= wy_max)
        .cloned()
        .collect();

    let region_w = wx_max - wx_min;
    let region_h = wy_max - wy_min;
    let hash = SeedSpatialHash::build_local(nearby_seeds, region_w, region_h, wx_min, wy_min);

    let mut data = vec![0u8; (patch_w * patch_h) as usize];

    for py in 0..patch_h {
        let ty = patch_y + py;
        let world_y = (ty as f32 + 0.5) * texel_h;
        for px in 0..patch_w {
            let tx = patch_x + px;
            let world_x = (tx as f32 + 0.5) * texel_w;
            if let Some(zone_id) = hash.nearest_zone(world_x, world_y) {
                if explored.contains(&zone_id) {
                    data[(py * patch_w + px) as usize] = 255;
                }
            }
        }
    }

    // Apply blur to smooth Voronoi edges within the patch
    blur_patch(&mut data, patch_w, patch_h);

    data
}

/// Compute the bounding box (in texels) that covers the given seeds, with margin.
pub fn compute_patch_bounds(
    seeds: &[ExplorationSeed],
    n_chunk_x: i32,
    n_chunk_y: i32,
    chunk_w: f32,
    chunk_h: f32,
    margin_texels: i32,
) -> (i32, i32, i32, i32) {
    let tex_w = n_chunk_x * EXPLORATION_RESOLUTION;
    let tex_h = n_chunk_y * EXPLORATION_RESOLUTION;
    let world_w = n_chunk_x as f32 * chunk_w;
    let world_h = n_chunk_y as f32 * chunk_h;
    let texel_w = world_w / tex_w as f32;
    let texel_h = world_h / tex_h as f32;

    let mut min_tx = tex_w;
    let mut min_ty = tex_h;
    let mut max_tx = 0i32;
    let mut max_ty = 0i32;

    for seed in seeds {
        let tx = (seed.x / texel_w) as i32;
        let ty = (seed.y / texel_h) as i32;
        min_tx = min_tx.min(tx);
        min_ty = min_ty.min(ty);
        max_tx = max_tx.max(tx);
        max_ty = max_ty.max(ty);
    }

    let cell_radius_texels = 10;
    let margin = margin_texels + cell_radius_texels;

    let px = (min_tx - margin).max(0);
    let py = (min_ty - margin).max(0);
    let ex = (max_tx + margin).min(tex_w - 1);
    let ey = (max_ty + margin).min(tex_h - 1);

    (px, py, ex - px + 1, ey - py + 1)
}

// =============================================================================
// BLUR — separable two-pass box blur for smooth Voronoi edges
// =============================================================================

/// Blur radius in texels. At 32× resolution, 5 texels ≈ 95 px ≈ half a cell.
const BLUR_RADIUS: usize = 5;

/// Apply a separable box blur to a rectangular region of the texture.
/// Two passes (horizontal then vertical) for O(n) per pixel regardless of radius.
fn blur_region(
    data: &mut [u8],
    tex_w: i32,
    tex_h: i32,
    region_x: i32,
    region_y: i32,
    region_w: i32,
    region_h: i32,
) {
    let w = tex_w as usize;
    let r = BLUR_RADIUS as i32;
    let kernel = (2 * r + 1) as u32;

    // Pass 1: horizontal blur → temp buffer
    let mut temp = vec![0u8; data.len()];
    for ty in region_y..(region_y + region_h) {
        if ty < 0 || ty >= tex_h { continue; }
        // Running sum for this row
        let mut sum: u32 = 0;
        // Initialize sum for first pixel
        for dx in -r..=r {
            let tx = (region_x + dx).clamp(0, tex_w - 1);
            sum += data[ty as usize * w + tx as usize] as u32;
        }
        temp[ty as usize * w + region_x.max(0) as usize] = (sum / kernel) as u8;

        for tx in (region_x + 1)..(region_x + region_w) {
            if tx < 0 || tx >= tex_w { continue; }
            // Add right edge, remove left edge
            let add_x = (tx + r).min(tex_w - 1);
            let rem_x = (tx - r - 1).max(0);
            sum += data[ty as usize * w + add_x as usize] as u32;
            sum -= data[ty as usize * w + rem_x as usize] as u32;
            temp[ty as usize * w + tx as usize] = (sum / kernel) as u8;
        }
    }

    // Pass 2: vertical blur on temp → back into data
    for tx in region_x..(region_x + region_w) {
        if tx < 0 || tx >= tex_w { continue; }
        let mut sum: u32 = 0;
        for dy in -r..=r {
            let ty = (region_y + dy).clamp(0, tex_h - 1);
            sum += temp[ty as usize * w + tx as usize] as u32;
        }
        let first_y = region_y.max(0);
        data[first_y as usize * w + tx as usize] = (sum / kernel) as u8;

        for ty in (region_y + 1)..(region_y + region_h) {
            if ty < 0 || ty >= tex_h { continue; }
            let add_y = (ty + r).min(tex_h - 1);
            let rem_y = (ty - r - 1).max(0);
            sum += temp[add_y as usize * w + tx as usize] as u32;
            sum -= temp[rem_y as usize * w + tx as usize] as u32;
            data[ty as usize * w + tx as usize] = (sum / kernel) as u8;
        }
    }
}

/// Apply blur to a standalone patch buffer (for ExplorationPatch messages).
/// The patch is small and self-contained, so we blur the entire thing.
pub fn blur_patch(data: &mut Vec<u8>, patch_w: i32, patch_h: i32) {
    blur_region(data, patch_w, patch_h, 0, 0, patch_w, patch_h);
}