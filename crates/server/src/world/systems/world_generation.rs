use crate::database::client::DatabaseTables;
use crate::database::tables;
use crate::world;
use crate::world::components::BiomeMeshData;
use crate::world::components::NaturalBuildingGenerator;
use crate::world::components::TerrainMeshData;
use crate::world::components::generate_ocean_data;
use crate::world::resources::WorldMaps;
use bevy::prelude::*;
use hexx::HexOrientation;
use shared::BuildingData;
use shared::GameState;
use shared::constants;
use shared::grid::GridConfig;
use sqlx::Row;

pub fn setup_grid_config() -> GridConfig {
    let radius = constants::HEX_SIZE;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(constants::HEX_RATIO.x, constants::HEX_RATIO.y);
    let chunk_size = 3u8; //constants::CHUNK_SIZE.x as u8;
    let grid_config = GridConfig::new(radius, orientation, ratio, chunk_size);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
    grid_config
}

pub async fn generate_world(map_name: &str, db_tables: &DatabaseTables, game_state: &GameState) {
    tracing::info!("Starting world generation...");
    let start = std::time::Instant::now();
    tracing::info!("Using map: {}", map_name);

    // Load maps
    let maps = WorldMaps::load(map_name, 12345).expect("Failed to load world maps");

    tracing::info!("Map loaded");

    let cell_db = &db_tables.cells;
    let terrain_db = &db_tables.terrains;
    let ocean_db = &db_tables.ocean_data;
    let building_db = &db_tables.buildings;

    tracing::info!("=== WORLD GENERATION PARAMETERS ===");
    tracing::info!(
        "Original map dimensions: {}x{}",
        maps.binary_map.width(),
        maps.binary_map.height()
    );
    tracing::info!(
        "Original heightmap dimensions: {}x{}",
        maps.heightmap.width(),
        maps.heightmap.height()
    );
    tracing::info!(
        "Map config chunks: {}x{}",
        maps.config.chunks_x,
        maps.config.chunks_y
    );

    // Scale factor used for terrain upscaling
    let scale = Vec2::splat(5.);
    tracing::info!("Scale factor: {}", scale.x);

    // Calculate dimensions AFTER scaling (same as terrain)
    let scaled_width = maps.binary_map.width() as f32 * scale.x;
    let scaled_height = maps.binary_map.height() as f32 * scale.y;
    let scaled_chunks_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
    let scaled_chunks_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;

    tracing::info!(
        "Scaled dimensions: {}x{} -> {} chunks ({}x{})",
        scaled_width,
        scaled_height,
        scaled_chunks_x * scaled_chunks_y,
        scaled_chunks_x,
        scaled_chunks_y
    );
    tracing::info!(
        "Chunk size: {}x{}",
        constants::CHUNK_SIZE.x,
        constants::CHUNK_SIZE.y
    );

    tracing::info!("=== GENERATING TERRAIN ===");
    let (terrain_mesh_data, chunk_masks, scaled_binary_map) = TerrainMeshData::from_image(
        map_name,
        &maps.binary_map,
        Some(&maps.heightmap),
        &scale,
        &format!("assets/maps/{}_binarymap.bin", map_name),
    );

    tracing::info!("✓ Terrain generated");
    tracing::info!(
        "Scaled binary map output: {}x{}",
        scaled_binary_map.width(),
        scaled_binary_map.height()
    );

    // Now generate ocean data with SCALED images and correct chunk counts
    tracing::info!("=== PREPARING OCEAN DATA ===");
    tracing::info!("Resizing heightmap to match scaled binary map...");
    let scaled_heightmap = image::imageops::resize(
        &maps.heightmap.to_luma8(),
        scaled_binary_map.width(),
        scaled_binary_map.height(),
        image::imageops::FilterType::Lanczos3,
    );
    tracing::info!(
        "✓ Heightmap resized to: {}x{}",
        scaled_heightmap.width(),
        scaled_heightmap.height()
    );

    tracing::info!("Calling generate_ocean_data with:");
    tracing::info!(
        "  - binary_map: {}x{}",
        scaled_binary_map.width(),
        scaled_binary_map.height()
    );
    tracing::info!(
        "  - heightmap: {}x{}",
        scaled_heightmap.width(),
        scaled_heightmap.height()
    );
    tracing::info!("  - chunks: {}x{}", scaled_chunks_x, scaled_chunks_y);
    tracing::info!(
        "  - world_size: {}x{}",
        scaled_chunks_x as f32 * constants::CHUNK_SIZE.x,
        scaled_chunks_y as f32 * constants::CHUNK_SIZE.y
    );

    let ocean_data = generate_ocean_data(
        map_name.to_string(),
        &image::DynamicImage::ImageLuma8(scaled_binary_map.clone()),
        &image::DynamicImage::ImageLuma8(scaled_heightmap),
        scaled_chunks_x,
        scaled_chunks_y,
        scaled_chunks_x as f32 * constants::CHUNK_SIZE.x,
        scaled_chunks_y as f32 * constants::CHUNK_SIZE.y,
    );

    ocean_db
        .save_ocean_data(ocean_data)
        .await
        .expect("Failed to save ocean data");

    for (id, chunk) in terrain_mesh_data.chunks {
        terrain_db
            .save_terrain(chunk.to_shared_terrain_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain");
    }

    let biome_mesh_data = BiomeMeshData::from_image(
        map_name,
        &maps.biome_map,
        &scaled_binary_map, //maps.binary_map.to_luma8(),
        &chunk_masks,
        &Vec2::splat(5.),
        "assets/maps/",
    );

    for (id, chunk) in biome_mesh_data.chunks.into_iter() {
        terrain_db
            .save_terrain_biome(chunk.to_shared_biome_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain biome");
    }

    let grid_config = &setup_grid_config();
    let sampled_cells = BiomeMeshData::sample_biome(
        map_name,
        &maps.biome_map.to_rgba8(),
        &Vec2::splat(5.),
        &grid_config.layout,
        "assets/maps/",
    );

    cell_db
        .save_cells(&sampled_cells)
        .await
        .expect("Failed to save cell data");

    // === GENERATING VORONOI ZONES ===
    tracing::info!("=== GENERATING VORONOI ZONES ===");

    // Prepare cells with biome information for Voronoi generation
    let cells_with_biomes: Vec<(shared::grid::GridCell, shared::BiomeTypeEnum)> = sampled_cells
        .iter()
        .map(|cell_data| (cell_data.cell, cell_data.biome))
        .collect();

    // Calculate world bounds from sampled cells
    let min_q = cells_with_biomes
        .iter()
        .map(|(c, _)| c.q)
        .min()
        .unwrap_or(0);
    let max_q = cells_with_biomes
        .iter()
        .map(|(c, _)| c.q)
        .max()
        .unwrap_or(0)
        + 1;
    let min_r = cells_with_biomes
        .iter()
        .map(|(c, _)| c.r)
        .min()
        .unwrap_or(0);
    let max_r = cells_with_biomes
        .iter()
        .map(|(c, _)| c.r)
        .max()
        .unwrap_or(0)
        + 1;

    let voronoi_seed = 12345u64; // Could be from config or derived from map seed
    let bounds = (min_q, max_q, min_r, max_r);

    match crate::world::voronoi::generate_and_save_zones(
        &db_tables.voronoi_zones,
        &cells_with_biomes,
        bounds,
        voronoi_seed,
    )
    .await
    {
        Ok(zone_count) => {
            tracing::info!("✓ Generated {} Voronoi zones", zone_count);
        }
        Err(e) => {
            tracing::error!("Failed to generate Voronoi zones: {}", e);
        }
    }

    let trees = NaturalBuildingGenerator::generate(&sampled_cells, game_state);

    building_db
        .save_buildings(
            &trees
                .buildings
                .values()
                .cloned()
                .collect::<Vec<BuildingData>>(),
        )
        .await
        .expect("Failed to save tree data");

    tracing::info!("✓ Generated {} map in {:?}", map_name, start.elapsed());

    // let world_config = maps.config.clone();
}

pub async fn clear_world(map_name: &str, terrain_db: &tables::TerrainsTable) {
    tracing::info!("Starting world clearing...");
    let start = std::time::Instant::now();
    tracing::info!("Using map: {}", map_name);

    terrain_db
        .clear_terrain(map_name)
        .await
        .expect("Failed to clear terrain");

    terrain_db
        .clear_terrain_biome(map_name)
        .await
        .expect("Failed to clear biomes");

    tracing::info!("✓ Clearing {} map in {:?}", map_name, start.elapsed());
}

pub async fn save_world_to_png(map_name: &str) {
    tracing::info!("Starting saving...");
    let start = std::time::Instant::now();
    let _ = TerrainMeshData::save_png_image(
        map_name,
        &format!("assets/maps/{}_binarymap.bin", map_name),
    );
    let _ = BiomeMeshData::save_png_image(map_name, "assets/maps/");
    tracing::info!("✓ Saving {} map in {:?}", map_name, start.elapsed());
}

/// Regenerate territory contours for all existing organizations
pub async fn regenerate_territory_contours(db_tables: &DatabaseTables) {
    tracing::info!("=== Starting Territory Contours Regeneration ===");
    let start = std::time::Instant::now();

    // Setup grid config
    let grid_config = setup_grid_config();

    // Get all organizations
    let organizations = match sqlx::query("SELECT id, name FROM organizations.organizations")
        .fetch_all(&db_tables.pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch organizations: {}", e);
            return;
        }
    };

    tracing::info!("Found {} organizations to process", organizations.len());

    let mut total_contours = 0;
    let mut processed_orgs = 0;

    for org_row in organizations {
        let org_id: i64 = org_row.get("id");
        let org_name: String = org_row.get("name");

        tracing::info!("Processing organization {} ({})", org_id, org_name);

        // Get territory cells for this organization
        let territory_cells_result = db_tables
            .organizations
            .load_territory_cells(org_id as u64)
            .await;

        match territory_cells_result {
            Ok(cells) if !cells.is_empty() => {
                tracing::info!("  Found {} territory cells", cells.len());

                // Convert GridCells to Hex
                let territory_hex: std::collections::HashSet<hexx::Hex> =
                    cells.iter().map(|cell| cell.to_hex()).collect();

                // Generate and split contours
                let contour_chunks = world::territory::generate_and_split_contour(
                    &territory_hex,
                    &grid_config.layout,
                    4.0,   // jitter amplitude
                    12345, //org_id as u64, // jitter seed (ensures consistency)
                );

                tracing::info!("  Generated {} contour chunks", contour_chunks.len());

                // Store contours in database
                let mut stored_count = 0;
                for (chunk_id, contour_segments) in contour_chunks {
                    match db_tables
                        .territory_contours
                        .store_contour(org_id as u64, chunk_id.x, chunk_id.y, &contour_segments)
                        .await
                    {
                        Ok(_) => {
                            stored_count += 1;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "  Failed to store contour for chunk ({},{}): {}",
                                chunk_id.x,
                                chunk_id.y,
                                e
                            );
                        }
                    }
                }

                tracing::info!(
                    "  ✓ Stored {} territory contour chunks for organization {}",
                    stored_count,
                    org_name
                );
                total_contours += stored_count;
                processed_orgs += 1;
            }
            Ok(_) => {
                tracing::warn!("  Organization {} has no territory cells", org_name);
            }
            Err(e) => {
                tracing::error!(
                    "  Failed to load territory cells for organization {}: {}",
                    org_name,
                    e
                );
            }
        }
    }

    tracing::info!("=== Territory Contours Regeneration Complete ===");
    tracing::info!("Processed {} organizations", processed_orgs);
    tracing::info!("Generated {} total contour chunks", total_contours);
    tracing::info!("Completed in {:?}", start.elapsed());
}
