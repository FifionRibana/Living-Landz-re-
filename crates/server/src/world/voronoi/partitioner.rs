use shared::grid::GridCell;
use rayon::prelude::*;
use std::collections::HashMap;

/// Calculate hexagonal Manhattan distance between two cells
/// Distance formula: (|Δq| + |Δr| + |Δs|) / 2
/// where s = -q - r (axial to cube coordinates)
#[inline]
pub fn hex_distance(a: GridCell, b: GridCell) -> i32 {
    let s1 = -a.q - a.r;
    let s2 = -b.q - b.r;
    ((a.q - b.q).abs() + (a.r - b.r).abs() + (s1 - s2).abs()) / 2
}

/// Partition all terrain cells into Voronoi zones
/// Uses parallel processing for performance
///
/// # Arguments
/// * `terrain_cells` - All cells to partition
/// * `seeds` - Voronoi seed cells with their zone IDs: (seed_cell, zone_id)
///
/// # Returns
/// HashMap mapping zone_id -> Vec<GridCell> containing all cells in that zone
pub fn partition_cells(
    terrain_cells: &[GridCell],
    seeds: &[(GridCell, i64)], // (seed_cell, zone_id)
) -> HashMap<i64, Vec<GridCell>> {
    if seeds.is_empty() {
        tracing::warn!("No Voronoi seeds provided for partitioning");
        return HashMap::new();
    }

    tracing::info!(
        "Partitioning {} cells into {} Voronoi zones...",
        terrain_cells.len(),
        seeds.len()
    );

    let start = std::time::Instant::now();

    // Parallel map: each cell → closest zone_id
    let assignments: Vec<(GridCell, i64)> = terrain_cells
        .par_iter()
        .map(|cell| {
            let mut min_dist = i32::MAX;
            let mut closest_zone = seeds[0].1;

            // Find closest seed
            for (seed, zone_id) in seeds {
                let dist = hex_distance(*cell, *seed);
                if dist < min_dist {
                    min_dist = dist;
                    closest_zone = *zone_id;
                }
            }

            (*cell, closest_zone)
        })
        .collect();

    // Group cells by zone
    let mut zones: HashMap<i64, Vec<GridCell>> = HashMap::new();
    for (cell, zone_id) in assignments {
        zones.entry(zone_id).or_default().push(cell);
    }

    tracing::info!(
        "Partitioned {} cells into {} zones in {:?}",
        terrain_cells.len(),
        zones.len(),
        start.elapsed()
    );

    // Log statistics
    if !zones.is_empty() {
        let sizes: Vec<usize> = zones.values().map(|cells| cells.len()).collect();
        let min_size = sizes.iter().min().unwrap_or(&0);
        let max_size = sizes.iter().max().unwrap_or(&0);
        let avg_size = sizes.iter().sum::<usize>() / sizes.len();

        tracing::info!(
            "Zone size stats: min={}, max={}, avg={}",
            min_size,
            max_size,
            avg_size
        );
    }

    zones
}

/// Partition cells in chunks for memory efficiency
/// Useful for very large worlds
pub fn partition_cells_chunked(
    terrain_cells: &[GridCell],
    seeds: &[(GridCell, i64)],
    chunk_size: usize,
) -> HashMap<i64, Vec<GridCell>> {
    let mut zones: HashMap<i64, Vec<GridCell>> = HashMap::new();

    for chunk in terrain_cells.chunks(chunk_size) {
        let chunk_zones = partition_cells(chunk, seeds);

        // Merge results
        for (zone_id, cells) in chunk_zones {
            zones.entry(zone_id).or_default().extend(cells);
        }
    }

    zones
}

/// Find cells on the border of a Voronoi zone
/// Border cells are those adjacent to cells in different zones
pub fn find_border_cells(
    zone_cells: &[GridCell],
    all_zones: &HashMap<i64, Vec<GridCell>>,
    zone_id: i64,
) -> Vec<GridCell> {
    // Build quick lookup set for this zone
    let cell_set: std::collections::HashSet<_> = zone_cells.iter().copied().collect();

    // Build lookup for all cells to zone
    let cell_to_zone: HashMap<GridCell, i64> = all_zones
        .iter()
        .flat_map(|(zid, cells)| cells.iter().map(move |c| (*c, *zid)))
        .collect();

    // Find border cells
    zone_cells
        .iter()
        .filter(|cell| {
            // Check if any neighbor belongs to a different zone
            cell.neighbors().iter().any(|neighbor| {
                cell_to_zone.get(neighbor).map_or(true, |nzid| *nzid != zone_id)
            })
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance() {
        let a = GridCell { q: 0, r: 0 };
        let b = GridCell { q: 3, r: 0 };
        assert_eq!(hex_distance(a, b), 3);

        let c = GridCell { q: 1, r: 1 };
        let d = GridCell { q: 4, r: 4 };
        assert_eq!(hex_distance(c, d), 6);
    }

    #[test]
    fn test_partition_cells() {
        // Create a simple grid of cells
        let mut cells = Vec::new();
        for q in 0..10 {
            for r in 0..10 {
                cells.push(GridCell { q, r });
            }
        }

        // Two seeds
        let seeds = vec![
            (GridCell { q: 2, r: 2 }, 1i64),
            (GridCell { q: 7, r: 7 }, 2i64),
        ];

        let zones = partition_cells(&cells, &seeds);

        // Should have 2 zones
        assert_eq!(zones.len(), 2);

        // All cells should be assigned
        let total: usize = zones.values().map(|v| v.len()).sum();
        assert_eq!(total, cells.len());

        // Zone 1 should have cells closer to (2,2)
        let zone1 = &zones[&1];
        assert!(zone1.contains(&GridCell { q: 2, r: 2 }));
        assert!(zone1.contains(&GridCell { q: 0, r: 0 }));

        // Zone 2 should have cells closer to (7,7)
        let zone2 = &zones[&2];
        assert!(zone2.contains(&GridCell { q: 7, r: 7 }));
        assert!(zone2.contains(&GridCell { q: 9, r: 9 }));
    }

    #[test]
    fn test_find_border_cells() {
        let zone1_cells = vec![
            GridCell { q: 0, r: 0 },
            GridCell { q: 1, r: 0 },
            GridCell { q: 0, r: 1 },
        ];

        let zone2_cells = vec![
            GridCell { q: 2, r: 0 },
            GridCell { q: 3, r: 0 },
        ];

        let mut all_zones = HashMap::new();
        all_zones.insert(1i64, zone1_cells.clone());
        all_zones.insert(2i64, zone2_cells);

        let borders = find_border_cells(&zone1_cells, &all_zones, 1);

        // Cell (1,0) should be a border (adjacent to zone 2 at (2,0))
        assert!(borders.contains(&GridCell { q: 1, r: 0 }));
    }
}
