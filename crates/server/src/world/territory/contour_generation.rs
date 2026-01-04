use bevy::prelude::*;
use hexx::*;
use std::collections::HashSet;
use shared::{constants, TerrainChunkId};

use crate::world::territory::territory_contours::build_contour;

/// Generate territory contour and split into chunks using geometric clipping
///
/// Returns a Vec of (TerrainChunkId, Vec<Vec2>) where each chunk contains only
/// the contour points that intersect with that chunk's boundaries
pub fn generate_and_split_contour(
    territory_cells: &HashSet<Hex>,
    layout: &HexLayout,
    jitter_amplitude: f32,
    jitter_seed: u64,
) -> Vec<(TerrainChunkId, Vec<Vec2>)> {
    if territory_cells.is_empty() {
        return Vec::new();
    }

    // Generate full contour using proven algorithm from territory_contours.rs
    let full_contour = build_contour(layout, territory_cells, jitter_amplitude, jitter_seed);

    if full_contour.is_empty() {
        tracing::warn!("build_contour returned empty contour for non-empty territory");
        return Vec::new();
    }

    tracing::info!(
        "Generated full contour with {} points for territory of {} cells",
        full_contour.len(),
        territory_cells.len()
    );

    // Find all chunks that this contour might intersect
    let affected_chunks = find_affected_chunks(&full_contour);
    tracing::info!(
        "Contour affects {} chunks (with buffer)",
        affected_chunks.len()
    );

    // For each chunk, clip the contour to only segments that intersect
    let mut result = Vec::new();
    for chunk_id in affected_chunks {
        let chunk_segments = clip_contour_to_chunk(&full_contour, chunk_id);
        if !chunk_segments.is_empty() {
            tracing::debug!(
                "Chunk ({},{}) has {} contour points",
                chunk_id.x,
                chunk_id.y,
                chunk_segments.len()
            );
            result.push((chunk_id, chunk_segments));
        }
    }

    tracing::info!(
        "Split contour into {} chunk segments (some chunks had no intersecting points)",
        result.len()
    );
    result
}

/// Find all chunks that the contour bounding box intersects
fn find_affected_chunks(contour: &[Vec2]) -> HashSet<TerrainChunkId> {
    let mut chunks = HashSet::new();

    // Get bounding box
    let (min, max) = compute_bounds(contour);

    // Calculate chunk range
    let min_chunk_x = (min.x / constants::CHUNK_SIZE.x).floor() as i32;
    let max_chunk_x = (max.x / constants::CHUNK_SIZE.x).ceil() as i32;
    let min_chunk_y = (min.y / constants::CHUNK_SIZE.y).floor() as i32;
    let max_chunk_y = (max.y / constants::CHUNK_SIZE.y).ceil() as i32;

    // Add all chunks in range (with 1-chunk buffer for segments near boundaries)
    for x in (min_chunk_x - 1)..=(max_chunk_x + 1) {
        for y in (min_chunk_y - 1)..=(max_chunk_y + 1) {
            chunks.insert(TerrainChunkId { x, y });
        }
    }

    chunks
}

/// Clip contour to only points within or near a chunk
///
/// This function includes all points that are within the chunk bounds + buffer,
/// ensuring smooth rendering across chunk boundaries
fn clip_contour_to_chunk(contour: &[Vec2], chunk_id: TerrainChunkId) -> Vec<Vec2> {
    let chunk_offset = Vec2::new(
        chunk_id.x as f32 * constants::CHUNK_SIZE.x,
        chunk_id.y as f32 * constants::CHUNK_SIZE.y,
    );

    // Expanded bounds to catch segments crossing chunk boundaries
    // Buffer matches TerritoryMaterial padding (50px)
    let buffer = 50.0;
    let min = chunk_offset - Vec2::splat(buffer);
    let max = chunk_offset + constants::CHUNK_SIZE + Vec2::splat(buffer);

    let mut clipped = Vec::new();
    let n = contour.len();

    // Track which points we've already added to avoid duplicates
    let mut added_points = HashSet::new();

    for i in 0..n {
        let current = contour[i];
        let next = contour[(i + 1) % n];

        // Check if segment intersects chunk bounds
        if segment_intersects_box(current, next, min, max) {
            // Add current point if in bounds and not already added
            if point_in_box(current, min, max) {
                // Use a simple hash for deduplication
                let key = (
                    (current.x * 10.0) as i32,
                    (current.y * 10.0) as i32,
                );
                if added_points.insert(key) {
                    clipped.push(current);
                }
            }

            // Add next point if in bounds and not already added
            if point_in_box(next, min, max) {
                let key = ((next.x * 10.0) as i32, (next.y * 10.0) as i32);
                if added_points.insert(key) {
                    clipped.push(next);
                }
            }
        }
    }

    clipped
}

/// Check if a line segment intersects with a bounding box
fn segment_intersects_box(a: Vec2, b: Vec2, box_min: Vec2, box_max: Vec2) -> bool {
    // Check if either endpoint is inside
    if point_in_box(a, box_min, box_max) || point_in_box(b, box_min, box_max) {
        return true;
    }

    // Check if segment bounding box intersects chunk box
    let seg_min = a.min(b);
    let seg_max = a.max(b);

    !(seg_max.x < box_min.x
        || seg_min.x > box_max.x
        || seg_max.y < box_min.y
        || seg_min.y > box_max.y)
}

/// Check if a point is inside a bounding box (inclusive)
fn point_in_box(p: Vec2, box_min: Vec2, box_max: Vec2) -> bool {
    p.x >= box_min.x && p.x <= box_max.x && p.y >= box_min.y && p.y <= box_max.y
}

/// Compute bounding box from a set of points
fn compute_bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_box() {
        let box_min = Vec2::new(0.0, 0.0);
        let box_max = Vec2::new(10.0, 10.0);

        assert!(point_in_box(Vec2::new(5.0, 5.0), box_min, box_max));
        assert!(point_in_box(Vec2::new(0.0, 0.0), box_min, box_max)); // Edge
        assert!(point_in_box(Vec2::new(10.0, 10.0), box_min, box_max)); // Edge
        assert!(!point_in_box(Vec2::new(-1.0, 5.0), box_min, box_max));
        assert!(!point_in_box(Vec2::new(11.0, 5.0), box_min, box_max));
    }

    #[test]
    fn test_segment_intersects_box() {
        let box_min = Vec2::new(0.0, 0.0);
        let box_max = Vec2::new(10.0, 10.0);

        // Segment fully inside
        assert!(segment_intersects_box(
            Vec2::new(2.0, 2.0),
            Vec2::new(8.0, 8.0),
            box_min,
            box_max
        ));

        // Segment crossing through
        assert!(segment_intersects_box(
            Vec2::new(-5.0, 5.0),
            Vec2::new(15.0, 5.0),
            box_min,
            box_max
        ));

        // Segment completely outside
        assert!(!segment_intersects_box(
            Vec2::new(-5.0, -5.0),
            Vec2::new(-2.0, -2.0),
            box_min,
            box_max
        ));

        // Segment with one endpoint inside
        assert!(segment_intersects_box(
            Vec2::new(5.0, 5.0),
            Vec2::new(15.0, 15.0),
            box_min,
            box_max
        ));
    }

    #[test]
    fn test_compute_bounds() {
        let points = vec![
            Vec2::new(1.0, 2.0),
            Vec2::new(-3.0, 5.0),
            Vec2::new(4.0, -1.0),
        ];

        let (min, max) = compute_bounds(&points);
        assert_eq!(min, Vec2::new(-3.0, -1.0));
        assert_eq!(max, Vec2::new(4.0, 5.0));
    }
}
