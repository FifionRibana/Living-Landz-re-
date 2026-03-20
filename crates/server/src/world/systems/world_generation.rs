use crate::database::client::DatabaseTables;
use crate::database::tables;
use crate::world;
use crate::world::components::BiomeMeshData;
use crate::world::components::NaturalBuildingGenerator;
use crate::world::components::TerrainMeshData;
use crate::world::components::generate_ocean_data;
use crate::world::resources::{WorldGlobalState, WorldMaps};
use bevy::prelude::*;
use hexx::HexOrientation;
use shared::BuildingData;
use shared::GameState;
use shared::TerrainChunkId;
use shared::constants;
use shared::grid::GridConfig;
use sqlx::Row;

pub fn setup_grid_config() -> GridConfig {
    let radius = constants::HEX_SIZE;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(constants::HEX_RATIO.x, constants::HEX_RATIO.y);
    let chunk_size = 3u8;
    let grid_config = GridConfig::new(radius, orientation, ratio, chunk_size);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
    grid_config
}

/// Generate global data only (SDF, biome, heightmap, ocean).
/// Returns WorldGlobalState to keep in memory for on-demand chunk generation.
pub async fn generate_world_globals(
    map_name: &str,
    db_tables: &DatabaseTables,
) -> WorldGlobalState {
    tracing::info!("=== GENERATING WORLD GLOBALS : {} ===", map_name);
    let start = std::time::Instant::now();

    let maps = WorldMaps::load(map_name, 12345).expect("Failed to load world maps");
    let grid_config = setup_grid_config();
    let scale = Vec2::splat(5.);
    let cache_path = format!("assets/maps/{}_binarymap.bin", map_name);

    let (mut global_state, terrain_global_data, scaled_binary_map) =
        TerrainMeshData::generate_globals(
            map_name,
            &maps.binary_map,
            Some(&maps.heightmap),
            Some(&maps.biome_map),
            &scale,
            &cache_path,
        );

    global_state.maps = Some(maps);
    global_state.grid_config = Some(grid_config);

    let global_maps = global_state.maps.as_ref().unwrap();

    // Save terrain global data (biome + heightmap textures for client)
    if let Some(ref global_data) = terrain_global_data {
        db_tables
            .terrain_global_data
            .save_terrain_global_data(global_data.clone())
            .await
            .expect("Failed to save terrain global data");
        tracing::info!(
            "✓ Terrain global data saved (biome {}x{}, heightmap {}x{})",
            global_data.biome_width,
            global_data.biome_height,
            global_data.heightmap_width,
            global_data.heightmap_height
        );
    }

    // Generate ocean data
    tracing::info!("=== PREPARING OCEAN DATA ===");
    let scaled_heightmap = image::imageops::resize(
        &global_maps.heightmap.to_luma8(),
        scaled_binary_map.width(),
        scaled_binary_map.height(),
        image::imageops::FilterType::Lanczos3,
    );

    let ocean_data = generate_ocean_data(
        map_name.to_string(),
        &image::DynamicImage::ImageLuma8(scaled_binary_map),
        &image::DynamicImage::ImageLuma8(scaled_heightmap),
        global_state.n_chunk_x,
        global_state.n_chunk_y,
        global_state.n_chunk_x as f32 * constants::CHUNK_SIZE.x,
        global_state.n_chunk_y as f32 * constants::CHUNK_SIZE.y,
    );

    db_tables
        .ocean_data
        .save_ocean_data(ocean_data)
        .await
        .expect("Failed to save ocean data");

    tracing::info!("✓ World globals generated in {:?}", start.elapsed());
    global_state
}

/// Load cached world globals from DB, or generate them if not found.
/// This is the normal server startup path.
pub async fn load_or_generate_world_globals(
    map_name: &str,
    db_tables: &DatabaseTables,
) -> WorldGlobalState {
    // Check if terrain global data exists in DB (biome/heightmap/ocean)
    let has_globals = db_tables
        .terrain_global_data
        .load_terrain_global_data(map_name)
        .await
        .ok()
        .flatten()
        .is_some();

    if has_globals {
        tracing::info!("Found cached terrain globals, loading maps only...");
        let t = std::time::Instant::now();

        let maps = WorldMaps::load(map_name, 12345).expect("Failed to load world maps");
        let grid_config = setup_grid_config();
        let scale = Vec2::splat(5.);

        // Compute chunk dimensions from source image + scale
        let scaled_width = maps.binary_map.width() as f32 * scale.x;
        let scaled_height = maps.binary_map.height() as f32 * scale.y;
        let n_chunk_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;

        let source_binary_flipped = image::imageops::flip_vertical(&maps.binary_map.to_luma8());

        let global_state = WorldGlobalState {
            map_name: map_name.to_string(),
            maps: Some(maps),
            source_binary_flipped,
            n_chunk_x,
            n_chunk_y,
            scale,
            sdf_resolution: 64,
            max_distance: 150.0,
            grid_config: Some(grid_config),
        };

        tracing::info!(
            "✓ World globals loaded in {:?} ({}x{} chunks, SDF computed per-chunk)",
            t.elapsed(),
            n_chunk_x,
            n_chunk_y
        );
        global_state
    } else {
        tracing::info!("No cached globals found, generating...");
        generate_world_globals(map_name, db_tables).await
    }
}

/// Generate a single chunk's data on demand: terrain mesh, cells, buildings.
/// Saves everything to DB and returns the data for immediate client response.
pub async fn generate_chunk_data(
    chunk_id: &TerrainChunkId,
    global: &WorldGlobalState,
    db_tables: &DatabaseTables,
    game_state: &GameState,
) -> (
    shared::TerrainChunkData,
    Vec<shared::grid::CellData>,
    Vec<BuildingData>,
) {
    let t = std::time::Instant::now();
    let map_name = &global.map_name;

    let global_maps = global.maps.as_ref().unwrap();
    let global_grid_config = global.grid_config.as_ref().unwrap();

    // 1. Generate terrain mesh
    let terrain_chunk = TerrainMeshData::generate_single_chunk(*chunk_id, global);

    let terrain_data = match terrain_chunk {
        Some(chunk) => chunk.to_shared_terrain_chunk_data(map_name, *chunk_id),
        None => shared::TerrainChunkData {
            name: map_name.to_string(),
            id: *chunk_id,
            ..Default::default()
        },
    };

    // Save terrain
    db_tables
        .terrains
        .save_terrain(terrain_data.clone())
        .await
        .expect("Failed to save terrain chunk");

    // 2. Sample hex cells for this chunk
    let chunk_world_min = Vec2::new(
        chunk_id.x as f32 * constants::CHUNK_SIZE.x,
        chunk_id.y as f32 * constants::CHUNK_SIZE.y,
    );
    let chunk_world_max = chunk_world_min + constants::CHUNK_SIZE;

    let maps = global.maps.as_ref().unwrap();
    let grid_config = global.grid_config.as_ref().unwrap();

    let chunk_cells = BiomeMeshData::sample_biome_for_chunk(
        map_name,
        &maps.biome_map.to_rgba8(),
        &global.scale,
        &grid_config.layout,
        "assets/maps/",
        chunk_id,
    );

    // Save cells
    if !chunk_cells.is_empty() {
        db_tables
            .cells
            .save_cells(&chunk_cells)
            .await
            .expect("Failed to save cells");
    }

    // 3. Generate trees for this chunk's cells
    let trees = NaturalBuildingGenerator::generate(&chunk_cells, game_state);
    let building_data: Vec<BuildingData> = trees.buildings.values().cloned().collect();

    if !building_data.is_empty() {
        db_tables
            .buildings
            .save_buildings(&building_data)
            .await
            .expect("Failed to save buildings");
    }

    tracing::info!(
        "✓ Chunk ({},{}) generated on demand in {:?} ({} cells, {} buildings)",
        chunk_id.x,
        chunk_id.y,
        t.elapsed(),
        chunk_cells.len(),
        building_data.len()
    );

    (terrain_data, chunk_cells, building_data)
}

/// Generate everything in batch (convenience for dev/testing).
/// Uses generate_world_globals + generate_chunk_data for each chunk.
pub async fn generate_world(map_name: &str, db_tables: &DatabaseTables, game_state: &GameState) {
    tracing::info!("Starting full world generation...");
    let start = std::time::Instant::now();

    let global_state = generate_world_globals(map_name, db_tables).await;
    let global_maps = global_state.maps.as_ref().unwrap();
    let global_grid_config = global_state.grid_config.as_ref().unwrap();

    // Generate all chunks
    let total_chunks = global_state.n_chunk_x * global_state.n_chunk_y;
    tracing::info!("Generating {} chunks...", total_chunks);

    let mut generated = 0;
    for cy in 0..global_state.n_chunk_y {
        for cx in 0..global_state.n_chunk_x {
            let chunk_id = TerrainChunkId { x: cx, y: cy };

            if global_state.chunk_has_land(&chunk_id) {
                generate_chunk_data(&chunk_id, &global_state, db_tables, game_state).await;
                generated += 1;
            }
        }
    }

    // Load cached scaled binary map for biome mesh generation
    let cache_path = format!("assets/maps/{}_binarymap.bin", map_name);
    let scaled_binary_map: image::ImageBuffer<image::Luma<u8>, Vec<u8>> =
        crate::utils::file_system::load_from_disk(&cache_path)
            .expect("Scaled binary map should be cached after generate_world_globals");

    let biome_mesh_data = BiomeMeshData::from_image(
        map_name,
        &global_maps.biome_map,
        &scaled_binary_map,
        &std::collections::HashMap::new(),
        &global_state.scale,
        "assets/maps/",
    );

    for (id, chunk) in biome_mesh_data.chunks.into_iter() {
        db_tables
            .terrains
            .save_terrain_biome(chunk.to_shared_biome_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain biome");
    }

    // Voronoi zones (still global)
    tracing::info!("=== GENERATING VORONOI ZONES ===");
    let all_cells = BiomeMeshData::sample_biome(
        map_name,
        &global_maps.biome_map.to_rgba8(),
        &global_state.scale,
        &global_grid_config.layout,
        "assets/maps/",
    );

    let cells_with_biomes: Vec<(shared::grid::GridCell, shared::BiomeTypeEnum)> = all_cells
        .iter()
        .map(|cell_data| (cell_data.cell, cell_data.biome))
        .collect();

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

    let voronoi_seed = 12345u64;
    let bounds = (min_q, max_q, min_r, max_r);

    match crate::world::voronoi::generate_and_save_zones(
        &db_tables.voronoi_zones,
        &cells_with_biomes,
        bounds,
        voronoi_seed,
    )
    .await
    {
        Ok(zone_count) => tracing::info!("✓ Generated {} Voronoi zones", zone_count),
        Err(e) => tracing::error!("Failed to generate Voronoi zones: {}", e),
    }

    tracing::info!(
        "✓ Full world generated in {:?} ({} land chunks out of {})",
        start.elapsed(),
        generated,
        total_chunks
    );
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
