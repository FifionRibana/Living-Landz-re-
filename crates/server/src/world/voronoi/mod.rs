pub mod seed_generator;

use crate::database::tables::VoronoiZonesTable;
use seed_generator::*;
use shared::grid::GridCell;
use shared::BiomeTypeEnum;

/// Generate Voronoi zone seeds and save them to the database.
/// Cell membership is computed on demand via shared::voronoi — no cell storage.
///
/// Uses biome-aware seed spacing for natural zone density variation.
pub async fn generate_and_save_zones(
    voronoi_db: &VoronoiZonesTable,
    terrain_cells: &[(GridCell, BiomeTypeEnum)],
    bounds: (i32, i32, i32, i32),
    seed: u64,
) -> Result<usize, String> {
    let (min_q, max_q, min_r, max_r) = bounds;

    tracing::info!("=== VORONOI ZONE GENERATION ===");
    tracing::info!(
        "World bounds: q[{},{}] r[{},{}]",
        min_q, max_q, min_r, max_r
    );
    tracing::info!("Total terrain cells: {}", terrain_cells.len());

    let config = SeedDensityConfig::default();

    let biome_map: std::collections::HashMap<GridCell, BiomeTypeEnum> =
        terrain_cells.iter().copied().collect();

    let biome_query =
        |cell: GridCell| *biome_map.get(&cell).unwrap_or(&BiomeTypeEnum::Grassland);

    let seeds_with_biome =
        generate_seeds_with_biome(min_q, max_q, min_r, max_r, &config, biome_query, seed);

    tracing::info!("Generated {} Voronoi seeds", seeds_with_biome.len());

    if seeds_with_biome.is_empty() {
        return Err("No Voronoi seeds generated".to_string());
    }

    let mut count = 0;
    for (seed_cell, biome) in &seeds_with_biome {
        voronoi_db.create_zone(*seed_cell, *biome).await?;
        count += 1;
    }

    tracing::info!(
        "✓ Voronoi zone generation complete: {} seeds stored",
        count
    );
    Ok(count)
}
