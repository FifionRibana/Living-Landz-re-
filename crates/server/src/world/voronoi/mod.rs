pub mod seed_generator;
pub mod partitioner;

use seed_generator::*;
use partitioner::*;
use crate::database::tables::VoronoiZonesTable;
use shared::grid::GridCell;
use shared::BiomeTypeEnum;

/// Generate Voronoi zones and save them to the database
///
/// This is the main entry point for Voronoi zone generation during world gen.
/// It generates seed points, partitions all terrain cells, and stores everything in DB.
///
/// # Arguments
/// * `voronoi_db` - Database table handler for Voronoi zones
/// * `terrain_cells` - All terrain cells to partition (with biome info)
/// * `bounds` - (min_q, max_q, min_r, max_r) - world boundaries
/// * `seed` - Random seed for reproducibility
///
/// # Returns
/// Number of zones created
pub async fn generate_and_save_zones(
    voronoi_db: &VoronoiZonesTable,
    terrain_cells: &[(GridCell, BiomeTypeEnum)], // cells with their biomes
    bounds: (i32, i32, i32, i32), // (min_q, max_q, min_r, max_r)
    seed: u64,
) -> Result<usize, String> {
    let (min_q, max_q, min_r, max_r) = bounds;

    tracing::info!("=== VORONOI ZONE GENERATION ===");
    tracing::info!("World bounds: q[{},{}] r[{},{}]", min_q, max_q, min_r, max_r);
    tracing::info!("Total terrain cells: {}", terrain_cells.len());

    // 1. Generate seeds using biome-aware spacing
    tracing::info!("Generating Voronoi seeds...");
    let config = SeedDensityConfig::default();

    // Build biome lookup for quick queries
    let biome_map: std::collections::HashMap<GridCell, BiomeTypeEnum> =
        terrain_cells.iter().copied().collect();

    let biome_query = |cell: GridCell| {
        *biome_map.get(&cell).unwrap_or(&BiomeTypeEnum::Grassland)
    };

    let seeds_with_biome = generate_seeds_with_biome(
        min_q,
        max_q,
        min_r,
        max_r,
        &config,
        biome_query,
        seed,
    );

    tracing::info!("Generated {} Voronoi seeds", seeds_with_biome.len());

    if seeds_with_biome.is_empty() {
        return Err("No Voronoi seeds generated".to_string());
    }

    // 2. Create zones in DB and collect (seed_cell, zone_id) pairs
    tracing::info!("Creating zones in database...");
    let mut seeds_with_ids = Vec::new();

    for (seed_cell, biome) in &seeds_with_biome {
        let zone_id = voronoi_db.create_zone(*seed_cell, *biome).await?;
        seeds_with_ids.push((*seed_cell, zone_id));
    }

    tracing::info!("Created {} zones in database", seeds_with_ids.len());

    // 3. Partition all terrain cells into zones
    tracing::info!("Partitioning {} cells into zones...", terrain_cells.len());

    let cells_only: Vec<GridCell> = terrain_cells.iter().map(|(cell, _)| *cell).collect();
    let zones = partition_cells(&cells_only, &seeds_with_ids);

    tracing::info!("Partitioned cells into {} zones", zones.len());

    // 4. Save zone assignments to database (in batches for performance)
    tracing::info!("Saving zone assignments to database...");

    const BATCH_SIZE: usize = 1000;
    let mut saved_count = 0;

    for (zone_id, cells) in zones.iter() {
        // Save in batches
        for batch in cells.chunks(BATCH_SIZE) {
            voronoi_db.add_cells_to_zone(*zone_id, batch).await?;
            saved_count += batch.len();
        }

        if saved_count % 10000 == 0 {
            tracing::info!("Saved {}/{} cell assignments...", saved_count, terrain_cells.len());
        }
    }

    tracing::info!("✓ Saved {} cell assignments", saved_count);
    tracing::info!("✓ Voronoi zone generation complete!");

    Ok(seeds_with_ids.len())
}

/// Simplified version that generates zones without biome information
/// Uses uniform spacing across the entire world
pub async fn generate_and_save_zones_simple(
    voronoi_db: &VoronoiZonesTable,
    terrain_cells: &[GridCell],
    bounds: (i32, i32, i32, i32),
    base_spacing: i32,
    jitter: i32,
    seed: u64,
) -> Result<usize, String> {
    let (min_q, max_q, min_r, max_r) = bounds;

    tracing::info!("=== VORONOI ZONE GENERATION (SIMPLE) ===");
    tracing::info!("World bounds: q[{},{}] r[{},{}]", min_q, max_q, min_r, max_r);
    tracing::info!("Spacing: {}, Jitter: ±{}", base_spacing, jitter);

    // 1. Generate seeds
    let seeds = generate_seeds_simple(min_q, max_q, min_r, max_r, base_spacing, jitter, seed);

    if seeds.is_empty() {
        return Err("No Voronoi seeds generated".to_string());
    }

    // 2. Create zones in DB (use Grassland as default biome)
    let mut seeds_with_ids = Vec::new();
    for seed_cell in &seeds {
        let zone_id = voronoi_db
            .create_zone(*seed_cell, BiomeTypeEnum::Grassland)
            .await?;
        seeds_with_ids.push((*seed_cell, zone_id));
    }

    tracing::info!("Created {} zones in database", seeds_with_ids.len());

    // 3. Partition
    let zones = partition_cells(terrain_cells, &seeds_with_ids);

    // 4. Save to DB
    const BATCH_SIZE: usize = 1000;
    for (zone_id, cells) in zones.iter() {
        for batch in cells.chunks(BATCH_SIZE) {
            voronoi_db.add_cells_to_zone(*zone_id, batch).await?;
        }
    }

    tracing::info!("✓ Voronoi zone generation complete!");

    Ok(seeds_with_ids.len())
}
