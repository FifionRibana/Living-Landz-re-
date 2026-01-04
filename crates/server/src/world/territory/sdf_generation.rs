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

/// Generate SDF for territory borders in a chunk (one per organization)
/// Uses parallel processing for performance
/// Returns a Vec of SDFs, one for each organization in this chunk
pub fn generate_border_sdf_for_chunk(
    chunk_id: TerrainChunkId,
    chunk_size: Vec2,
    grid_config: &GridConfig,
    all_territories: &HashMap<GridCell, u64>,
    resolution: u32, // Typically 600 to match terrain resolution
) -> Vec<TerritoryBorderChunkSdfData> {
    let width = resolution;
    let height = (chunk_size.y / chunk_size.x * resolution as f32) as u32;

    // Calculate world offset for this chunk
    let chunk_offset = Vec2::new(
        chunk_id.x as f32 * chunk_size.x,
        chunk_id.y as f32 * chunk_size.y,
    );

    // Group territories by organization
    let mut territories_by_org: HashMap<u64, HashSet<GridCell>> = HashMap::new();
    for (cell, org_id) in all_territories {
        territories_by_org
            .entry(*org_id)
            .or_default()
            .insert(*cell);
    }

    info!(
        "Chunk ({},{}): Found {} organizations with territories",
        chunk_id.x,
        chunk_id.y,
        territories_by_org.len()
    );

    // If no territories, return empty list
    if territories_by_org.is_empty() {
        return Vec::new();
    }

    // Generate a separate SDF for each organization
    let chunk_min = chunk_offset;
    let chunk_max = chunk_offset + chunk_size;
    let texel_size_x = chunk_size.x / resolution as f32;
    let texel_size_y = chunk_size.y / resolution as f32;

    let mut result_sdfs = Vec::new();

    for (org_id, territory_cells) in territories_by_org.iter() {
        // Convert GridCell to Hex
        let territory_hex: HashSet<hexx::Hex> = territory_cells
            .iter()
            .map(|cell| cell.to_hex())
            .collect();

        // Use build_contour to get smooth, optimized contour points
        let contour_points = crate::world::territory::build_contour(
            &grid_config.layout,
            &territory_hex,
            2.0,    // jitter amplitude
            0,      // no jitter for SDF (we want exact borders)
        );

        // Convert contour points to line segments
        let mut contour_segments = Vec::new();
        for i in 0..contour_points.len() {
            let current = contour_points[i];
            let next = contour_points[(i + 1) % contour_points.len()];
            contour_segments.push((current, next));
        }

        // Clip segments to chunk boundaries
        let mut clipped_segments: Vec<(Vec2, Vec2)> = Vec::new();
        for (p1, p2) in contour_segments.iter() {
            let clipped = clip_segment_to_chunk(*p1, *p2, chunk_min, chunk_max);
            clipped_segments.extend(clipped);
        }

        // Skip if no segments in this chunk
        if clipped_segments.is_empty() {
            continue;
        }

        info!(
            "Org {}: {} clipped segments in chunk ({},{})",
            org_id,
            clipped_segments.len(),
            chunk_id.x,
            chunk_id.y
        );

        // Generate SDF for this organization in parallel
        let sdf_data: Vec<u8> = (0..(width * height))
            .into_par_iter()
            .map(|idx| {
                let x = idx % width;
                let y = idx / width;

                // World position of this pixel (center of texel)
                let pixel_world_pos = chunk_offset
                    + Vec2::new(
                        (x as f32 + 0.5) * texel_size_x,
                        (y as f32 + 0.5) * texel_size_y,
                    );

                // Find minimum distance to any contour segment
                let min_distance = clipped_segments
                    .iter()
                    .map(|(a, b)| distance_point_to_segment(pixel_world_pos, *a, *b))
                    .fold(f32::MAX, f32::min);

                // Normalize distance to 0-255
                // 0 = on border, 255 = far from border (50px max distance)
                let max_dist = 50.0;
                let normalized = (min_distance / max_dist).clamp(0.0, 1.0);
                (normalized * 255.0) as u8
            })
            .collect();

        // Generate colors for this organization
        let (border_color, fill_color) = crate::world::territory::generate_org_colors(*org_id);

        result_sdfs.push(TerritoryBorderChunkSdfData {
            chunk_x: chunk_id.x,
            chunk_y: chunk_id.y,
            width,
            height,
            sdf_data,
            organization_id: *org_id,
            border_color,
            fill_color,
        });
    }

    result_sdfs
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

/// Generate territory border SDFs for all affected chunks
/// This should be called after organizations are created or territories are updated
pub async fn generate_territory_border_sdfs(
    db_pool: &PgPool,
    grid_config: &GridConfig,
    chunk_size: Vec2,
    resolution: u32,
) -> Result<HashMap<(i32, i32), Vec<TerritoryBorderChunkSdfData>>, String> {
    // Load all territories
    let all_territories = load_all_territories_map(db_pool).await?;

    if all_territories.is_empty() {
        info!("No territories to generate borders for");
        return Ok(HashMap::new());
    }

    info!("Generating territory border SDFs for {} cells", all_territories.len());

    // Find all affected chunks
    let mut affected_chunks = HashSet::new();
    for cell in all_territories.keys() {
        // Convert cell to world position
        let world_pos = grid_config.layout.hex_to_world_pos(cell.to_hex());

        // Calculate chunk coordinates
        let chunk_x = (world_pos.x / chunk_size.x).floor() as i32;
        let chunk_y = (world_pos.y / chunk_size.y).floor() as i32;

        affected_chunks.insert((chunk_x, chunk_y));

        // Also check neighboring chunks (borders might span chunk boundaries)
        for dx in -1..=1 {
            for dy in -1..=1 {
                affected_chunks.insert((chunk_x + dx, chunk_y + dy));
            }
        }
    }

    info!("Generating SDFs for {} chunks", affected_chunks.len());

    // Generate SDF for each chunk in parallel, flattening the results
    let sdf_results: Vec<((i32, i32), TerritoryBorderChunkSdfData)> = affected_chunks
        .into_par_iter()
        .flat_map(|(chunk_x, chunk_y)| {
            let chunk_id = TerrainChunkId {
                x: chunk_x,
                y: chunk_y,
            };

            let sdf_data_list = generate_border_sdf_for_chunk(
                chunk_id,
                chunk_size,
                grid_config,
                &all_territories,
                resolution,
            );

            // Convert Vec<TerritoryBorderChunkSdfData> to Vec<((i32, i32), TerritoryBorderChunkSdfData)>
            sdf_data_list
                .into_iter()
                .map(move |sdf_data| ((chunk_x, chunk_y), sdf_data))
                .collect::<Vec<_>>()
        })
        .collect();

    // Group SDFs by chunk
    let mut result: HashMap<(i32, i32), Vec<TerritoryBorderChunkSdfData>> = HashMap::new();
    for (chunk_coords, sdf_data) in sdf_results {
        result.entry(chunk_coords).or_default().push(sdf_data);
    }

    info!("Generated {} territory border SDF chunks", result.len());
    Ok(result)
}

/// Generate territory border SDF for a single specific chunk
/// Returns an empty Vec if there are no borders in this chunk
pub async fn generate_territory_border_sdf_for_chunk(
    db_pool: &PgPool,
    grid_config: &GridConfig,
    chunk_id: TerrainChunkId,
    chunk_size: Vec2,
    resolution: u32,
) -> Result<Vec<TerritoryBorderChunkSdfData>, String> {
    // Load all territories
    let all_territories = load_all_territories_map(db_pool).await?;

    if all_territories.is_empty() {
        return Ok(Vec::new());
    }

    // Check if this chunk or its neighbors have any territories
    let chunk_x = chunk_id.x;
    let chunk_y = chunk_id.y;

    let mut has_nearby_territories = false;
    for dx in -1..=1 {
        for dy in -1..=1 {
            let check_chunk_x = chunk_x + dx;
            let check_chunk_y = chunk_y + dy;

            // Calculate chunk bounds
            let chunk_offset = Vec2::new(
                check_chunk_x as f32 * chunk_size.x,
                check_chunk_y as f32 * chunk_size.y,
            );

            // Check if any territory cell is in this neighboring chunk
            for cell in all_territories.keys() {
                let world_pos = grid_config.layout.hex_to_world_pos(cell.to_hex());

                if world_pos.x >= chunk_offset.x
                    && world_pos.x < chunk_offset.x + chunk_size.x
                    && world_pos.y >= chunk_offset.y
                    && world_pos.y < chunk_offset.y + chunk_size.y
                {
                    has_nearby_territories = true;
                    break;
                }
            }

            if has_nearby_territories {
                break;
            }
        }
        if has_nearby_territories {
            break;
        }
    }

    if !has_nearby_territories {
        return Ok(Vec::new());
    }

    // Generate SDFs for this chunk (one per organization)
    let sdf_data_list = generate_border_sdf_for_chunk(
        chunk_id,
        chunk_size,
        grid_config,
        &all_territories,
        resolution,
    );

    Ok(sdf_data_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed - border detection now uses edge-based approach instead of cell-based
    // TODO: Add tests for find_border_edges() and BorderEdge::distance_to_point()
}
