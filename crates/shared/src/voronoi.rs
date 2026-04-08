use crate::grid::GridCell;
use std::collections::{HashSet, VecDeque};

/// Hex Manhattan distance between two cells.
pub fn hex_distance(a: GridCell, b: GridCell) -> i32 {
    let s1 = -a.q - a.r;
    let s2 = -b.q - b.r;
    ((a.q - b.q).abs() + (a.r - b.r).abs() + (s1 - s2).abs()) / 2
}

/// Find the closest seed to a cell. Returns (zone_id, distance).
/// Tie-breaking: smallest zone_id wins (deterministic).
pub fn find_closest_seed(cell: GridCell, seeds: &[(GridCell, i64)]) -> Option<(i64, i32)> {
    let mut best: Option<(i64, i32)> = None;
    for (seed_cell, zone_id) in seeds {
        let dist = hex_distance(cell, *seed_cell);
        match best {
            None => best = Some((*zone_id, dist)),
            Some((best_id, best_dist)) => {
                if dist < best_dist || (dist == best_dist && *zone_id < best_id) {
                    best = Some((*zone_id, dist));
                }
            }
        }
    }
    best
}

/// BFS flood-fill to find all cells belonging to a zone.
/// Starts from the seed and expands to neighbors as long as they are
/// closest to this seed (compared to all other seeds).
/// Capped at MAX_ZONE_CELLS to prevent runaway expansion.
pub fn compute_zone_cells(
    zone_id: i64,
    seed_cell: GridCell,
    all_seeds: &[(GridCell, i64)],
) -> Vec<GridCell> {
    const MAX_ZONE_CELLS: usize = 2000;

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(seed_cell);
    visited.insert(seed_cell);

    while let Some(cell) = queue.pop_front() {
        if result.len() >= MAX_ZONE_CELLS {
            break;
        }

        if let Some((closest_zone, _)) = find_closest_seed(cell, all_seeds) {
            if closest_zone != zone_id {
                continue;
            }
        } else {
            continue;
        }

        result.push(cell);

        for neighbor in cell.neighbors() {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    result
}
