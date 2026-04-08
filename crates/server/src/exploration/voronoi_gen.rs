//! Server-side exploration Voronoi functions.
//!
//! Pure computation (constants, seeds, hashing) lives in shared::exploration::voronoi.
//! This module re-exports those and adds server-only functions that depend on
//! world geometry (chunks, constants::CHUNK_SIZE) or tracing.

use hexx::HexLayout;
use shared::constants;
use std::collections::HashSet;

// Re-export shared computation
pub use shared::exploration::voronoi::*;

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
    let margin = radius_hex + EXPLORATION_SEED_SPACING + EXPLORATION_SEED_JITTER;
    let seeds = compute_seeds_in_region(
        center.q - margin,
        center.q + margin + 1,
        center.r - margin,
        center.r + margin + 1,
        layout,
        world_seed,
    );

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
pub fn is_chunk_explored_voronoi(
    chunk_x: i32,
    chunk_y: i32,
    explored_set: &HashSet<i64>,
    layout: &HexLayout,
    world_seed: u64,
) -> bool {
    let chunk_w = constants::CHUNK_SIZE.x;
    let chunk_h = constants::CHUNK_SIZE.y;

    let wx_min = chunk_x as f32 * chunk_w;
    let wy_min = chunk_y as f32 * chunk_h;
    let wx_max = wx_min + chunk_w;
    let wy_max = wy_min + chunk_h;

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

    let cell_radius = (EXPLORATION_SEED_SPACING as f32) * 48.0;

    seeds.iter().any(|seed| {
        if !explored_set.contains(&seed.zone_id) {
            return false;
        }
        let closest_x = seed.x.clamp(wx_min, wx_max);
        let closest_y = seed.y.clamp(wy_min, wy_max);
        let dx = seed.x - closest_x;
        let dy = seed.y - closest_y;
        dx * dx + dy * dy < cell_radius * cell_radius
    })
}
