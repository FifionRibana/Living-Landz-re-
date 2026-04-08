//! Exploration Voronoi — fully deterministic, on-demand.
//!
//! Seeds are computed from (world_seed, grid position) — no DB storage needed.
//! Only the set of explored zone_ids is persisted.
//!
//! Architecture:
//!   zone_id = stable hash of (world_seed, grid_q, grid_r)
//!   seed position = grid center + deterministic jitter from same hash
//!   Any server/client instance can recompute any seed from coordinates alone.

use hexx::{Hex, HexLayout};

/// Spacing in hex cells between exploration Voronoi seeds.
/// At spacing=4, with chunk ~12.5×14 hex → ~10 seeds/chunk.
pub const EXPLORATION_SEED_SPACING: i32 = 4;

/// Jitter range applied to seed positions (in hex cells).
pub const EXPLORATION_SEED_JITTER: i32 = 1;

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
pub fn zone_id_from_grid(gq: i32, gr: i32, world_seed: u64) -> i64 {
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
pub fn jitter_from_grid(gq: i32, gr: i32, world_seed: u64) -> (i32, i32) {
    let mut h = world_seed.wrapping_add(0xBEEF);
    h = h.wrapping_add(gq as u64).wrapping_mul(6364136223846793005);
    h = h.wrapping_add(gr as u64).wrapping_mul(1442695040888963407);
    h ^= h >> 17;

    let jitter_range = EXPLORATION_SEED_JITTER * 2 + 1;
    let jq = ((h & 0xFFFF) as i32 % jitter_range) - EXPLORATION_SEED_JITTER;
    let jr = (((h >> 16) & 0xFFFF) as i32 % jitter_range) - EXPLORATION_SEED_JITTER;
    (jq, jr)
}

/// Compute all exploration seeds in a hex region [min_q..max_q) × [min_r..max_r).
///
/// This is **pure computation** — no DB, no IO. Can be called anywhere, anytime.
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

    let start_q = (min_q as f64 / spacing as f64).floor() as i32 * spacing;
    let start_r = (min_r as f64 / spacing as f64).floor() as i32 * spacing;

    let mut gq = start_q;
    while gq < max_q {
        let mut gr = start_r;
        while gr < max_r {
            let (jq, jr) = jitter_from_grid(gq, gr, world_seed);
            let q = gq + jq;
            let r = gr + jr;

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
