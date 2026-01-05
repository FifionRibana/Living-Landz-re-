use shared::grid::{GridCell, GridConfig};
use shared::{TerritoryBorderChunkSdfData, TerrainChunkId};
use bevy::prelude::*;
use hexx::Hex;
use rayon::prelude::*;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};

/// Identify border cells of a territory (cells that have at least one neighbor outside the territory)
pub async fn get_territory_border_cells(
    db_pool: &PgPool,
    organization_id: u64,
) -> Result<Vec<GridCell>, String> {
    // Get all cells belonging to this organization
    let rows = sqlx::query(
        "SELECT cell_q, cell_r
         FROM organizations.territory_cells
         WHERE organization_id = $1"
    )
    .bind(organization_id as i64)
    .fetch_all(db_pool)
    .await
    .map_err(|e| format!("Failed to fetch organization cells: {}", e))?;

    let territory_cells: HashSet<GridCell> = rows
        .iter()
        .map(|row| GridCell {
            q: row.get::<i32, _>("cell_q"),
            r: row.get::<i32, _>("cell_r"),
        })
        .collect();

    if territory_cells.is_empty() {
        return Ok(Vec::new());
    }

    // Find border cells: cells that have at least one neighbor outside the territory
    let mut border_cells = Vec::new();

    for cell in &territory_cells {
        let neighbors = cell.neighbors();
        let has_external_neighbor = neighbors.iter().any(|neighbor| {
            !territory_cells.contains(neighbor)
        });

        if has_external_neighbor {
            border_cells.push(*cell);
        }
    }

    info!(
        "Organization {}: Found {} border cells out of {} total cells",
        organization_id,
        border_cells.len(),
        territory_cells.len()
    );

    Ok(border_cells)
}

/// Get the midpoint of an external edge for a cell
/// Returns the center point of the edge in world coordinates
fn get_external_edge_midpoint(
    cell: &GridCell,
    neighbor_index: usize,
    grid_config: &GridConfig,
) -> Option<Vec2> {
    let center = grid_config.layout.hex_to_world_pos(cell.to_hex());
    let radius = grid_config.hex_radius;

    // For flat-top hexagons:
    // - Vertex distance from center = radius
    // - Edge midpoint distance from center = radius * cos(30°) = radius * sqrt(3)/2
    let edge_distance = radius * 0.866025404; // sqrt(3)/2

    // For flat-top hexagons, edge midpoints are at these angles (in degrees from east):
    // Edge midpoint positions: 0° (East), 60° (NE), 120° (NW), 180° (West), 240° (SW), 300° (SE)
    let angle_deg = match neighbor_index {
        0 => 0.0_f32,   // East
        1 => 60.0,      // NorthEast
        2 => 120.0,     // NorthWest
        3 => 180.0,     // West
        4 => 240.0,     // SouthWest
        5 => 300.0,     // SouthEast
        _ => return None,
    };

    let rad = angle_deg.to_radians();
    let edge_midpoint = center + Vec2::new(edge_distance * rad.cos(), edge_distance * rad.sin());

    Some(edge_midpoint)
}

/// Group territory cells into contiguous regions
/// Returns a vector of cell sets, each representing a connected region
fn group_contiguous_regions(territory_cells: &HashSet<GridCell>) -> Vec<HashSet<GridCell>> {
    let mut regions = Vec::new();
    let mut visited = HashSet::new();

    for start_cell in territory_cells {
        if visited.contains(start_cell) {
            continue;
        }

        // Flood-fill to find all cells in this contiguous region
        let mut region = HashSet::new();
        let mut to_visit = vec![*start_cell];

        while let Some(cell) = to_visit.pop() {
            if visited.contains(&cell) || !territory_cells.contains(&cell) {
                continue;
            }

            visited.insert(cell);
            region.insert(cell);

            // Add neighbors that are also in the territory
            for neighbor in cell.neighbors() {
                if territory_cells.contains(&neighbor) && !visited.contains(&neighbor) {
                    to_visit.push(neighbor);
                }
            }
        }

        if !region.is_empty() {
            regions.push(region);
        }
    }

    regions
}

/// Build contour segments for a territory by creating continuous ordered paths
/// Handles multiple disconnected regions within the same territory
/// Returns a list of line segments forming all closed perimeters
fn build_territory_contour_segments(
    territory_cells: &HashSet<GridCell>,
    grid_config: &GridConfig,
) -> Vec<(Vec2, Vec2)> {
    // Group cells into contiguous regions
    let regions = group_contiguous_regions(territory_cells);

    info!("Territory has {} contiguous region(s)", regions.len());

    let mut all_segments = Vec::new();

    // Build a contour for each contiguous region
    for (region_idx, region) in regions.iter().enumerate() {
        // Collect midpoints of external edges with their associated cells
        let mut edge_points: Vec<(Vec2, GridCell, usize)> = Vec::new();

        for cell in region {
            let neighbors = cell.neighbors();
            for (i, neighbor) in neighbors.iter().enumerate() {
                if !region.contains(&neighbor) {
                    if let Some(midpoint) = get_external_edge_midpoint(cell, i, grid_config) {
                        edge_points.push((midpoint, *cell, i));

                        // Debug: log first few edge points
                        if edge_points.len() <= 3 {
                            let cell_pos = grid_config.layout.hex_to_world_pos(cell.to_hex());
                            info!(
                                "  Edge point #{}: cell ({},{}) at {:?}, direction {}, midpoint at {:?}",
                                edge_points.len(),
                                cell.q,
                                cell.r,
                                cell_pos,
                                i,
                                midpoint
                            );
                        }
                    }
                }
            }
        }

        if edge_points.is_empty() {
            continue;
        }

        info!("Region #{}: {} external edge midpoints", region_idx + 1, edge_points.len());

        // Build ordered contour by walking around the perimeter
        let contour = build_ordered_contour_from_edges(&edge_points, region);

        info!("Region #{}: Built contour with {} points", region_idx + 1, contour.len());

        // Convert contour points to line segments
        if contour.len() >= 2 {
            for i in 0..contour.len() {
                let current = contour[i];
                let next = contour[(i + 1) % contour.len()];
                all_segments.push((current, next));
            }
        }
    }

    info!("Total segments generated: {}", all_segments.len());
    all_segments
}

/// Build an ordered contour by following the perimeter
fn build_ordered_contour_from_edges(
    edge_points: &[(Vec2, GridCell, usize)],
    region: &HashSet<GridCell>,
) -> Vec<Vec2> {
    if edge_points.is_empty() {
        return Vec::new();
    }

    // Build a map from (cell, direction) to point for quick lookup
    let mut edge_map: HashMap<(GridCell, usize), Vec2> = HashMap::new();
    for (point, cell, dir) in edge_points {
        edge_map.insert((*cell, *dir), *point);
    }

    // Find a starting cell on the perimeter (any cell with external edges)
    let start_cell = edge_points[0].1;
    let mut contour = Vec::new();
    let mut visited_edges: HashSet<(GridCell, usize)> = HashSet::new();

    // Start walking around the perimeter
    let mut current_cell = start_cell;
    let mut iterations = 0;
    let max_iterations = edge_points.len() * 2;

    // Find the first external edge of the starting cell (leftmost/smallest direction index)
    let mut current_dir = None;
    for dir in 0..6 {
        if edge_map.contains_key(&(current_cell, dir)) {
            current_dir = Some(dir);
            break;
        }
    }

    if current_dir.is_none() {
        warn!("No starting edge found for contour");
        return Vec::new();
    }

    let start_dir = current_dir.unwrap();
    let mut current_dir = start_dir;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            warn!("Max iterations reached in contour building");
            break;
        }

        // Add current edge point if not visited
        if let Some(point) = edge_map.get(&(current_cell, current_dir)) {
            if !visited_edges.contains(&(current_cell, current_dir)) {
                contour.push(*point);
                visited_edges.insert((current_cell, current_dir));
            }
        }

        // Try to move to next edge (clockwise around the perimeter)
        // First, try to continue on the same cell
        let mut found_next = false;
        for offset in 1..6 {
            let next_dir = (current_dir + offset) % 6;
            if edge_map.contains_key(&(current_cell, next_dir))
                && !visited_edges.contains(&(current_cell, next_dir)) {
                current_dir = next_dir;
                found_next = true;
                break;
            }
        }

        if !found_next {
            // Try to move to a neighboring cell
            let neighbors = current_cell.neighbors();
            for neighbor in neighbors.iter() {
                if region.contains(neighbor) {
                    // Check if this neighbor has external edges we haven't visited
                    for dir in 0..6 {
                        if edge_map.contains_key(&(*neighbor, dir))
                            && !visited_edges.contains(&(*neighbor, dir)) {
                            current_cell = *neighbor;
                            current_dir = dir;
                            found_next = true;
                            break;
                        }
                    }
                    if found_next {
                        break;
                    }
                }
            }
        }

        if !found_next {
            // Check if we've completed the loop
            if contour.len() >= 3 {
                break;
            } else {
                warn!("Could not find next edge in contour");
                break;
            }
        }

        // Check if we've returned to start
        if current_cell == start_cell && current_dir == start_dir && contour.len() > 1 {
            break;
        }
    }

    // Remove consecutive duplicates
    let epsilon = 1.0;
    contour.dedup_by(|a, b| a.distance(*b) < epsilon);

    info!("Contour built by perimeter following: {} points", contour.len());

    contour
}

/// Calculate distance from a point to a line segment
fn distance_point_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = point - a;

    let ab_len_sq = ab.length_squared();
    if ab_len_sq < 0.0001 {
        return point.distance(a);
    }

    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;

    point.distance(projection)
}

/// Check if a point is inside chunk bounds
fn point_in_chunk(point: Vec2, chunk_min: Vec2, chunk_max: Vec2) -> bool {
    point.x >= chunk_min.x && point.x <= chunk_max.x
        && point.y >= chunk_min.y && point.y <= chunk_max.y
}

/// Calculate intersection of a line segment with chunk boundary
/// Returns the intersection point if it exists
fn line_chunk_intersection(
    p1: Vec2,
    p2: Vec2,
    chunk_min: Vec2,
    chunk_max: Vec2,
) -> Vec<Vec2> {
    let mut intersections = Vec::new();
    let dir = p2 - p1;

    // Check intersection with each edge of the chunk
    // Left edge (x = chunk_min.x)
    if dir.x.abs() > 0.0001 {
        let t = (chunk_min.x - p1.x) / dir.x;
        if t >= 0.0 && t <= 1.0 {
            let y = p1.y + t * dir.y;
            if y >= chunk_min.y && y <= chunk_max.y {
                intersections.push(Vec2::new(chunk_min.x, y));
            }
        }
    }

    // Right edge (x = chunk_max.x)
    if dir.x.abs() > 0.0001 {
        let t = (chunk_max.x - p1.x) / dir.x;
        if t >= 0.0 && t <= 1.0 {
            let y = p1.y + t * dir.y;
            if y >= chunk_min.y && y <= chunk_max.y {
                intersections.push(Vec2::new(chunk_max.x, y));
            }
        }
    }

    // Bottom edge (y = chunk_min.y)
    if dir.y.abs() > 0.0001 {
        let t = (chunk_min.y - p1.y) / dir.y;
        if t >= 0.0 && t <= 1.0 {
            let x = p1.x + t * dir.x;
            if x >= chunk_min.x && x <= chunk_max.x {
                intersections.push(Vec2::new(x, chunk_min.y));
            }
        }
    }

    // Top edge (y = chunk_max.y)
    if dir.y.abs() > 0.0001 {
        let t = (chunk_max.y - p1.y) / dir.y;
        if t >= 0.0 && t <= 1.0 {
            let x = p1.x + t * dir.x;
            if x >= chunk_min.x && x <= chunk_max.x {
                intersections.push(Vec2::new(x, chunk_max.y));
            }
        }
    }

    // Remove duplicates (points very close to each other)
    intersections.dedup_by(|a, b| a.distance(*b) < 0.1);

    intersections
}

/// Clip a line segment to chunk boundaries
/// Returns the clipped segment(s) that are inside the chunk
fn clip_segment_to_chunk(
    p1: Vec2,
    p2: Vec2,
    chunk_min: Vec2,
    chunk_max: Vec2,
) -> Vec<(Vec2, Vec2)> {
    let p1_inside = point_in_chunk(p1, chunk_min, chunk_max);
    let p2_inside = point_in_chunk(p2, chunk_min, chunk_max);

    if p1_inside && p2_inside {
        // Both points inside: keep the whole segment
        return vec![(p1, p2)];
    }

    if !p1_inside && !p2_inside {
        // Both points outside: check if segment crosses the chunk
        let intersections = line_chunk_intersection(p1, p2, chunk_min, chunk_max);
        if intersections.len() >= 2 {
            // Segment crosses chunk - return the part inside
            return vec![(intersections[0], intersections[1])];
        } else {
            // Segment doesn't touch chunk
            return vec![];
        }
    }

    // One point inside, one outside: clip to boundary
    let intersections = line_chunk_intersection(p1, p2, chunk_min, chunk_max);
    if intersections.is_empty() {
        return vec![];
    }

    if p1_inside {
        vec![(p1, intersections[0])]
    } else {
        vec![(intersections[0], p2)]
    }
}

/// Load all territories from database and create a lookup map
pub async fn load_all_territories_map(
    db_pool: &sqlx::PgPool,
) -> Result<HashMap<GridCell, u64>, String> {
    let rows = sqlx::query(
        r#"
        SELECT organization_id, cell_q, cell_r
        FROM organizations.territory_cells
        "#,
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| format!("Failed to load territories: {}", e))?;

    let mut map = HashMap::new();
    for row in rows {
        let org_id: i64 = row.get("organization_id");
        let cell = GridCell {
            q: row.get("cell_q"),
            r: row.get("cell_r"),
        };
        map.insert(cell, org_id as u64);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed - border detection now uses edge-based approach instead of cell-based
    // TODO: Add tests for find_border_edges() and BorderEdge::distance_to_point()
}
