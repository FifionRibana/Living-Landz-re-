//! Exploration Voronoi — fully deterministic, on-demand.
//!
//! Seeds are computed from (world_seed, grid position) — no DB storage needed.
//! Only the set of explored zone_ids is persisted.
//!
//! Architecture:
//!   zone_id = stable hash of (world_seed, grid_q, grid_r)
//!   seed position = grid center + deterministic jitter from same hash
//!   Any server instance can recompute any seed from coordinates alone.

use hexx::{Hex, HexLayout};
use shared::constants;
use std::collections::HashSet;

/// Spacing in hex cells between exploration Voronoi seeds.
/// At spacing=4, with chunk ~12.5×14 hex → ~10 seeds/chunk.
pub const EXPLORATION_SEED_SPACING: i32 = 4;

/// Jitter range applied to seed positions (in hex cells).
const EXPLORATION_SEED_JITTER: i32 = 1;

/// Global seed for exploration Voronoi. Deterministic — same seed = same world.
pub const EXPLORATION_WORLD_SEED: u64 = 54321;

/// A computed exploration seed — ephemeral, never stored in DB.
#[derive(Debug, Clone, Copy)]
pub struct ExplorationSeed {
    /// Deterministic ID derived from world_seed + grid position.
    pub zone_id: i64,
    /// World pixel position of the seed (after jitter).
    pub x: f32,
    pub y: f32,
    /// Hex grid position of the seed (after jitter).
    pub cell_q: i32,
    pub cell_r: i32,
}

/// Deterministic hash for a grid position → zone_id.
/// Same (world_seed, gq, gr) always produces the same i64.
/// Uses a simple but well-distributed mixing function.
fn zone_id_from_grid(gq: i32, gr: i32, world_seed: u64) -> i64 {
    let mut h = world_seed;
    h = h.wrapping_add(gq as u64).wrapping_mul(6364136223846793005);
    h = h.wrapping_add(gr as u64).wrapping_mul(1442695040888963407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    // Keep positive for DB compatibility
    (h & 0x7FFFFFFFFFFFFFFF) as i64
}

/// Deterministic jitter for a grid position.
/// Returns (jitter_q, jitter_r) in [-JITTER, +JITTER].
fn jitter_from_grid(gq: i32, gr: i32, world_seed: u64) -> (i32, i32) {
    // Use a different mixing than zone_id to avoid correlation
    let mut h = world_seed.wrapping_add(0xBEEF);
    h = h.wrapping_add(gq as u64).wrapping_mul(6364136223846793005);
    h = h.wrapping_add(gr as u64).wrapping_mul(1442695040888963407);
    h ^= h >> 17;

    let jitter_range = EXPLORATION_SEED_JITTER * 2 + 1; // e.g. 3 for jitter=1
    let jq = ((h & 0xFFFF) as i32 % jitter_range) - EXPLORATION_SEED_JITTER;
    let jr = (((h >> 16) & 0xFFFF) as i32 % jitter_range) - EXPLORATION_SEED_JITTER;
    (jq, jr)
}

/// Compute all exploration seeds in a hex region [min_q..max_q) × [min_r..max_r).
///
/// This is **pure computation** — no DB, no IO. Can be called anywhere, anytime.
/// For rasterization, call with the full world bounds.
/// For local operations, call with a sub-region + margin.
pub fn compute_seeds_in_region(
    min_q: i32,
    max_q: i32,
    min_r: i32,
    max_r: i32,
    layout: &HexLayout,
    world_seed: u64,
) -> Vec<ExplorationSeed> {
    let spacing = EXPLORATION_SEED_SPACING;
    let mut seeds = Vec::new();

    // Align to grid: find first grid point >= min_q/min_r
    let start_q = (min_q as f64 / spacing as f64).floor() as i32 * spacing;
    let start_r = (min_r as f64 / spacing as f64).floor() as i32 * spacing;

    let mut gq = start_q;
    while gq < max_q {
        let mut gr = start_r;
        while gr < max_r {
            let (jq, jr) = jitter_from_grid(gq, gr, world_seed);
            let q = gq + jq;
            let r = gr + jr;

            // Seed must stay within the requested region
            if q >= min_q && q < max_q && r >= min_r && r < max_r {
                let zone_id = zone_id_from_grid(gq, gr, world_seed);
                let world_pos = layout.hex_to_world_pos(Hex::new(q, r));
                seeds.push(ExplorationSeed {
                    zone_id,
                    x: world_pos.x,
                    y: world_pos.y,
                    cell_q: q,
                    cell_r: r,
                });
            }

            gr += spacing;
        }
        gq += spacing;
    }

    seeds
}

/// Compute seeds in the entire world.
/// Derives hex bounds from chunk grid dimensions + hex layout.
pub fn compute_all_seeds(
    n_chunk_x: i32,
    n_chunk_y: i32,
    layout: &HexLayout,
    world_seed: u64,
) -> Vec<ExplorationSeed> {
    let world_w = n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let world_h = n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    // In axial hex coordinates (flat-top), q and r don't map linearly to x,y.
    // We must check ALL 4 corners and take the global min/max.
    let corners = [
        bevy::math::Vec2::new(0.0, 0.0),
        bevy::math::Vec2::new(world_w, 0.0),
        bevy::math::Vec2::new(0.0, world_h),
        bevy::math::Vec2::new(world_w, world_h),
    ];

    let hex_corners: Vec<_> = corners.iter().map(|c| layout.world_pos_to_hex(*c)).collect();

    let margin = EXPLORATION_SEED_SPACING + EXPLORATION_SEED_JITTER + 1;
    let min_q = hex_corners.iter().map(|h| h.x).min().unwrap() - margin;
    let max_q = hex_corners.iter().map(|h| h.x).max().unwrap() + margin;
    let min_r = hex_corners.iter().map(|h| h.y).min().unwrap() - margin;
    let max_r = hex_corners.iter().map(|h| h.y).max().unwrap() + margin;

    tracing::info!(
        "🔍 compute_all_seeds: world={}×{}, hex corners={:?}, bounds q[{}..{}] r[{}..{}]",
        world_w, world_h,
        hex_corners.iter().map(|h| (h.x, h.y)).collect::<Vec<_>>(),
        min_q, max_q, min_r, max_r
    );

    compute_seeds_in_region(min_q, max_q, min_r, max_r, layout, world_seed)
}

/// Find zone_ids of all seeds within `radius_hex` hex distance of `center`.
pub fn zones_in_radius(
    center: &shared::grid::GridCell,
    radius_hex: i32,
    layout: &HexLayout,
    world_seed: u64,
) -> Vec<i64> {
    // Compute seeds in a bounding box around the center
    let margin = radius_hex + EXPLORATION_SEED_SPACING + EXPLORATION_SEED_JITTER;
    let seeds = compute_seeds_in_region(
        center.q - margin,
        center.q + margin + 1,
        center.r - margin,
        center.r + margin + 1,
        layout,
        world_seed,
    );

    // Filter by hex distance
    seeds
        .iter()
        .filter(|seed| {
            let dq = (seed.cell_q - center.q).abs();
            let dr = (seed.cell_r - center.r).abs();
            let ds = ((seed.cell_q + seed.cell_r) - (center.q + center.r)).abs();
            (dq + dr + ds) / 2 <= radius_hex
        })
        .map(|s| s.zone_id)
        .collect()
}

/// Check if a chunk has any explored Voronoi zone overlapping it.
/// Used for streaming: "should this chunk be loaded?"
pub fn is_chunk_explored_voronoi(
    chunk_x: i32,
    chunk_y: i32,
    explored_set: &HashSet<i64>,
    layout: &HexLayout,
    world_seed: u64,
) -> bool {
    let chunk_w = constants::CHUNK_SIZE.x;
    let chunk_h = constants::CHUNK_SIZE.y;

    // Chunk bounding box in world pixels
    let wx_min = chunk_x as f32 * chunk_w;
    let wy_min = chunk_y as f32 * chunk_h;
    let wx_max = wx_min + chunk_w;
    let wy_max = wy_min + chunk_h;

    // Convert chunk corners to hex bounds (all 4 for axial correctness)
    let margin = EXPLORATION_SEED_SPACING + EXPLORATION_SEED_JITTER + 1;
    let corners = [
        bevy::math::Vec2::new(wx_min, wy_min),
        bevy::math::Vec2::new(wx_max, wy_min),
        bevy::math::Vec2::new(wx_min, wy_max),
        bevy::math::Vec2::new(wx_max, wy_max),
    ];
    let hex_corners: Vec<_> = corners.iter().map(|c| layout.world_pos_to_hex(*c)).collect();

    let seeds = compute_seeds_in_region(
        hex_corners.iter().map(|h| h.x).min().unwrap() - margin,
        hex_corners.iter().map(|h| h.x).max().unwrap() + margin + 1,
        hex_corners.iter().map(|h| h.y).min().unwrap() - margin,
        hex_corners.iter().map(|h| h.y).max().unwrap() + margin + 1,
        layout,
        world_seed,
    );

    // Check if any seed whose Voronoi cell overlaps the chunk is explored
    // Approximate: a seed "overlaps" if it's within ~cell_radius of the chunk bbox
    let cell_radius = (EXPLORATION_SEED_SPACING as f32) * 48.0; // spacing * hex_width

    seeds.iter().any(|seed| {
        if !explored_set.contains(&seed.zone_id) {
            return false;
        }
        // Check if seed is close enough to the chunk to potentially overlap
        let closest_x = seed.x.clamp(wx_min, wx_max);
        let closest_y = seed.y.clamp(wy_min, wy_max);
        let dx = seed.x - closest_x;
        let dy = seed.y - closest_y;
        dx * dx + dy * dy < cell_radius * cell_radius
    })
}