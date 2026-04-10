use crate::database::client::DatabaseTables;
use crate::database::tables;
use crate::world;
use crate::world::components::BiomeMeshData;
use crate::world::components::NaturalBuildingGenerator;
use crate::world::components::TerrainMeshData;
use crate::world::components::generate_global_sdf;
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
    let scale = Vec2::splat(100.);

    let (mut global_state, terrain_global_data) = TerrainMeshData::generate_globals(
        map_name,
        &maps.binary_map,
        &maps.lake_map,
        Some(&maps.heightmap),
        Some(&maps.biome_map),
        &scale,
    );

    // Cache source biome RGBA for per-chunk cell sampling
    let source_biome_flipped = image::imageops::flip_vertical(&maps.biome_map.to_rgba8());
    tracing::info!(
        "✓ Source biome RGBA prepared ({}x{})",
        source_biome_flipped.width(),
        source_biome_flipped.height()
    );

    let source_lake_flipped = image::imageops::flip_vertical(&maps.lake_map);
    tracing::info!(
        "✓ Source lake map prepared ({}x{})",
        source_lake_flipped.width(),
        source_lake_flipped.height()
    );
    global_state.source_lake_flipped = source_lake_flipped;

    global_state.source_biome_flipped_rgba = Some(source_biome_flipped);
    global_state.maps = Some(maps);
    global_state.grid_config = Some(grid_config);

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

    // Generate ocean data from source at capped resolution
    tracing::info!("=== PREPARING OCEAN DATA ===");
    let global_maps = global_state.maps.as_ref().unwrap();

    let ocean_max_dim = 4096u32;
    let src_w = global_maps.binary_map.width();
    let src_h = global_maps.binary_map.height();
    let ocean_scale = if (src_w as f32 * global_state.scale.x) as u32 > ocean_max_dim {
        ocean_max_dim as f32 / src_w.max(src_h) as f32
    } else {
        global_state.scale.x
    };

    tracing::info!(
        "Ocean source: {}x{}, scale: {:.1} (capped from {:.1})",
        src_w,
        src_h,
        ocean_scale,
        global_state.scale.x
    );

    let ocean_binary_flipped = image::imageops::flip_vertical(&global_maps.binary_map);
    let mut ocean_binary = image::imageops::resize(
        &ocean_binary_flipped,
        (src_w as f32 * ocean_scale) as u32,
        (src_h as f32 * ocean_scale) as u32,
        image::imageops::FilterType::Lanczos3,
    );
    // Threshold like resize_image does
    ocean_binary.iter_mut().for_each(|p| {
        *p = if *p > 178 { 255 } else { 0 };
    });

    let ocean_heightmap = image::imageops::resize(
        &global_maps.heightmap,
        ocean_binary.width(),
        ocean_binary.height(),
        image::imageops::FilterType::Lanczos3,
    );

    let ocean_binary_w = ocean_binary.width();
    let ocean_binary_h = ocean_binary.height();

    // Pass ocean image dimensions as "world" for correct SDF search_radius.
    // The actual mesh size is computed from n_chunk_x * 64 in OceanData,
    // so the client mesh covers the real world regardless.
    let mut ocean_data = generate_ocean_data(
        map_name.to_string(),
        &ocean_binary,
        &ocean_heightmap,
        global_state.n_chunk_x,
        global_state.n_chunk_y,
        ocean_binary_w as f32,
        ocean_binary_h as f32,
    );

    // Override world dimensions with real world size (not capped ocean image size)
    ocean_data.world_width = global_state.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    ocean_data.world_height = global_state.n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    tracing::info!(
        "🌊 Saving ocean : (texture {}x{}, world_width={}, world_height={})",
        ocean_data.width,
        ocean_data.height,
        ocean_data.world_width,
        ocean_data.world_height
    );

    db_tables
        .ocean_data
        .save_ocean_data(ocean_data)
        .await
        .expect("Failed to save ocean data");

    // Generate lake data from lakemap (mask + SDF)
    tracing::info!("=== PREPARING LAKE DATA ===");
    let lake_src = &global_state.maps.as_ref().unwrap().lake_map;
    let lake_w = lake_src.width();
    let lake_h = lake_src.height();

    let lake_max_dim = 2048u32;
    let (lake_target_w, lake_target_h) = if lake_w > lake_max_dim || lake_h > lake_max_dim {
        let ratio = lake_h as f32 / lake_w as f32;
        if lake_w >= lake_h {
            (lake_max_dim, (lake_max_dim as f32 * ratio).round() as u32)
        } else {
            ((lake_max_dim as f32 / ratio).round() as u32, lake_max_dim)
        }
    } else {
        (lake_w, lake_h)
    };

    // Resize and flip mask
    // Upscale lake source for smoother SDF (same approach as ocean)
    let lake_upscale_dim = 4096u32;
    let lake_ratio = lake_h as f32 / lake_w as f32;
    let (lake_upscale_w, lake_upscale_h) = if lake_w >= lake_h {
        (
            lake_upscale_dim,
            (lake_upscale_dim as f32 * lake_ratio).round() as u32,
        )
    } else {
        (
            (lake_upscale_dim as f32 / lake_ratio).round() as u32,
            lake_upscale_dim,
        )
    };

    let lake_upscaled = image::imageops::resize(
        lake_src,
        lake_upscale_w,
        lake_upscale_h,
        image::imageops::FilterType::Lanczos3,
    );
    let lake_flipped = image::imageops::flip_vertical(&lake_upscaled);

    // Generate lake SDF: invert the mask (land=white, lake=black) to match
    // generate_global_sdf convention (white=land, black=water)
    let mut lake_inverted = lake_flipped.clone();
    lake_inverted.iter_mut().for_each(|p| {
        *p = if *p > 178 { 0 } else { 255 };
    });

    // Mask at original target resolution for client
    let lake_mask_resized = image::imageops::resize(
        lake_src,
        lake_target_w,
        lake_target_h,
        image::imageops::FilterType::Lanczos3,
    );
    let lake_mask_flipped = image::imageops::flip_vertical(&lake_mask_resized);

    // SDF at same resolution as mask
    let lake_sdf_w = lake_target_w as usize;
    let lake_sdf_h = lake_target_h as usize;
    let lake_world_width = global_state.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let lake_world_height = global_state.n_chunk_y as f32 * constants::CHUNK_SIZE.y;
    let lake_max_distance = 150.0f32 * 100.0 / 50.0; // banks are narrow

    tracing::info!(
        "Generating lake SDF {}x{} (max_distance: {})",
        lake_sdf_w,
        lake_sdf_h,
        lake_max_distance
    );

    let lake_sdf = generate_global_sdf(
        &lake_inverted,
        lake_sdf_w,
        lake_sdf_h,
        lake_world_width,
        lake_world_height,
        lake_max_distance,
    );

    tracing::info!("✓ Lake SDF generated: {} bytes", lake_sdf.len());

    let lake_data = shared::LakeData {
        name: map_name.to_string(),
        width: lake_target_w as usize,
        height: lake_target_h as usize,
        mask_values: lake_mask_flipped.into_raw(),
        sdf_width: lake_sdf_w,
        sdf_height: lake_sdf_h,
        sdf_values: lake_sdf,
        world_width: lake_world_width,
        world_height: lake_world_height,
        generated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    tracing::info!(
        "✓ Lake data: mask {}x{}, SDF {}x{} ({:.2} KB total)",
        lake_target_w,
        lake_target_h,
        lake_sdf_w,
        lake_sdf_h,
        (lake_data.mask_values.len() + lake_data.sdf_values.len()) as f64 / 1024.0
    );

    db_tables
        .lake_data
        .save_lake_data(lake_data)
        .await
        .expect("Failed to save lake data");

    tracing::info!("✓ World globals generated in {:?}", start.elapsed());
    global_state
}

/// Load cached world globals from DB, or generate them if not found.
/// This is the normal server startup path.
pub async fn load_or_generate_world_globals(
    map_name: &str,
    db_tables: &DatabaseTables,
) -> WorldGlobalState {
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
        let scale = Vec2::splat(100.);

        let scaled_width = maps.binary_map.width() as f32 * scale.x;
        let scaled_height = maps.binary_map.height() as f32 * scale.y;
        let n_chunk_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;

        let source_binary_flipped = image::imageops::flip_vertical(&maps.binary_map);
        let source_lake_flipped = image::imageops::flip_vertical(&maps.lake_map);
        let source_biome_flipped = image::imageops::flip_vertical(&maps.biome_map.to_rgba8());

        // Load enriched heightmap from cached terrain global data
        let terrain_global = db_tables
            .terrain_global_data
            .load_terrain_global_data(map_name)
            .await
            .ok()
            .flatten();

        let (enriched_hm, ehm_w, ehm_h) = if let Some(ref tg) = terrain_global {
            (Some(tg.heightmap_values.clone()), tg.heightmap_width, tg.heightmap_height)
        } else {
            (None, 0, 0)
        };

        let global_state = WorldGlobalState {
            map_name: map_name.to_string(),
            maps: Some(maps),
            source_binary_flipped,
            source_lake_flipped,
            n_chunk_x,
            n_chunk_y,
            scale,
            sdf_resolution: 64,
            max_distance: 150.0,
            grid_config: Some(grid_config),
            source_biome_flipped_rgba: Some(source_biome_flipped),
            enriched_heightmap: enriched_hm,
            enriched_heightmap_width: ehm_w,
            enriched_heightmap_height: ehm_h,
        };

        tracing::info!(
            "✓ World globals loaded in {:?} ({}x{} chunks)",
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

/// Check if voronoi zone seeds exist in DB; generate them if missing.
/// Called at server startup. Only stores seeds — zone cell membership
/// is computed on demand via shared::voronoi (like the exploration/mist system).
pub async fn ensure_voronoi_zones(db_tables: &DatabaseTables, global_state: &WorldGlobalState) {
    let zone_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM terrain.voronoi_zones")
        .fetch_one(&db_tables.pool)
        .await
        .unwrap_or(0);

    if zone_count > 0 {
        tracing::info!("✓ Voronoi zones OK: {} seeds in database", zone_count);
        return;
    }

    tracing::info!("⚠️ Voronoi zones table is empty — generating seeds...");

    let Some(ref grid_config) = global_state.grid_config else {
        tracing::warn!("No grid config — cannot generate Voronoi zones");
        return;
    };

    // Compute hex bounds from all 4 world corners
    let n_chunk_x = global_state.n_chunk_x;
    let n_chunk_y = global_state.n_chunk_y;
    let world_w = n_chunk_x as f32 * shared::constants::CHUNK_SIZE.x;
    let world_h = n_chunk_y as f32 * shared::constants::CHUNK_SIZE.y;

    let c0 = grid_config.layout.world_pos_to_hex(Vec2::new(0.0, 0.0));
    let c1 = grid_config.layout.world_pos_to_hex(Vec2::new(world_w, 0.0));
    let c2 = grid_config.layout.world_pos_to_hex(Vec2::new(0.0, world_h));
    let c3 = grid_config
        .layout
        .world_pos_to_hex(Vec2::new(world_w, world_h));

    let min_q = c0.x.min(c1.x).min(c2.x).min(c3.x) - 1;
    let max_q = c0.x.max(c1.x).max(c2.x).max(c3.x) + 2;
    let min_r = c0.y.min(c1.y).min(c2.y).min(c3.y) - 1;
    let max_r = c0.y.max(c1.y).max(c2.y).max(c3.y) + 2;

    tracing::info!(
        "World hex bounds: q[{},{}] r[{},{}]",
        min_q,
        max_q,
        min_r,
        max_r
    );

    // Get terrain chunks for land filtering (seeds only on land)
    let chunk_rows = sqlx::query("SELECT DISTINCT chunk_x, chunk_y FROM terrain.terrains")
        .fetch_all(&db_tables.pool)
        .await
        .unwrap_or_default();

    let terrain_chunks: std::collections::HashSet<(i32, i32)> = chunk_rows
        .iter()
        .map(|r| (r.get::<i32, _>("chunk_x"), r.get::<i32, _>("chunk_y")))
        .collect();

    // Generate seeds, filter to land
    let base_spacing = 16;
    let jitter = 3;
    let voronoi_seed = 12345u64;

    let seeds = crate::world::voronoi::seed_generator::generate_seeds_simple(
        min_q,
        max_q,
        min_r,
        max_r,
        base_spacing,
        jitter,
        voronoi_seed,
    );

    let land_seeds: Vec<shared::grid::GridCell> = seeds
        .into_iter()
        .filter(|s| {
            let chunk = s.to_chunk_id(&grid_config.layout);
            terrain_chunks.contains(&(chunk.x, chunk.y))
        })
        .collect();

    tracing::info!("Generated {} land seeds", land_seeds.len());

    if land_seeds.is_empty() {
        tracing::warn!("No land seeds generated — cannot create Voronoi zones");
        return;
    }

    // Store seeds in DB (that's all — no cell partitioning needed)
    let mut count = 0;
    for seed_cell in &land_seeds {
        if db_tables
            .voronoi_zones
            .create_zone(*seed_cell, shared::BiomeTypeEnum::Grassland)
            .await
            .is_ok()
        {
            count += 1;
        }
    }

    tracing::info!("✓ Voronoi seed generation complete: {} seeds stored", count);
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
    let t_total = std::time::Instant::now();
    let map_name = &global.map_name;
    let grid_config = global.grid_config.as_ref().unwrap();

    // 1. Generate terrain mesh + SDF
    let t1 = std::time::Instant::now();
    let terrain_chunk = TerrainMeshData::generate_single_chunk(*chunk_id, global);

    let terrain_data = match terrain_chunk {
        Some(chunk) => chunk.to_shared_terrain_chunk_data(map_name, *chunk_id),
        None => shared::TerrainChunkData {
            name: map_name.to_string(),
            id: *chunk_id,
            ..Default::default()
        },
    };
    let t1_elapsed = t1.elapsed();

    // 2. Sample hex cells for this chunk
    let t2 = std::time::Instant::now();
    let maps = global.maps.as_ref().unwrap();
    let grid_config = global.grid_config.as_ref().unwrap();

    // 2. Sample hex cells for this chunk
    let t2 = std::time::Instant::now();
    let chunk_cells = if let Some(ref source_biome) = global.source_biome_flipped_rgba {
        BiomeMeshData::sample_biome_for_chunk(
            source_biome,
            &global.source_binary_flipped,
            &global.source_lake_flipped,
            &global.scale,
            &grid_config.layout,
            chunk_id,
            global.enriched_heightmap.as_ref().map(|data| {
                (data.as_slice(), global.enriched_heightmap_width, global.enriched_heightmap_height)
            }),
        )
    } else {
        vec![]
    };
    let t2_elapsed = t2.elapsed();

    // 3. Generate trees
    let t3 = std::time::Instant::now();
    let trees = NaturalBuildingGenerator::generate(&chunk_cells, game_state);
    let building_data: Vec<BuildingData> = trees.buildings.values().cloned().collect();
    let t3_elapsed = t3.elapsed();

    // 4. Save to DB
    let t4 = std::time::Instant::now();
    for attempt in 0..3 {
        match db_tables.terrains.save_terrain(terrain_data.clone()).await {
            Ok(_) => break,
            Err(e) if attempt < 2 => {
                tracing::warn!(
                    "Terrain save attempt {} failed, retrying: {}",
                    attempt + 1,
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    50 * (attempt as u64 + 1),
                ))
                .await;
            }
            Err(e) => {
                tracing::error!("Failed to save terrain after 3 attempts: {}", e);
            }
        }
    }

    if !chunk_cells.is_empty() {
        for attempt in 0..3 {
            match db_tables.cells.save_cells(&chunk_cells).await {
                Ok(_) => break,
                Err(e) if attempt < 2 => {
                    tracing::warn!(
                        "Cells save attempt {} failed (deadlock?), retrying: {}",
                        attempt + 1,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        50 * (attempt as u64 + 1),
                    ))
                    .await;
                }
                Err(e) => {
                    tracing::error!("Failed to save cells after 3 attempts: {}", e);
                }
            }
        }
    }

    if !building_data.is_empty() {
        for attempt in 0..3 {
            match db_tables.buildings.save_buildings(&building_data).await {
                Ok(_) => break,
                Err(e) if attempt < 2 => {
                    tracing::warn!(
                        "Buildings save attempt {} failed (deadlock?), retrying: {}",
                        attempt + 1,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        50 * (attempt as u64 + 1),
                    ))
                    .await;
                }
                Err(e) => {
                    tracing::error!("Failed to save buildings after 3 attempts: {}", e);
                }
            }
        }
    }
    let t4_elapsed = t4.elapsed();

    tracing::info!(
        "✓ Chunk ({},{}) generated in {:?} [terrain: {:?}, cells: {:?} ({}), trees: {:?} ({}), db: {:?}]",
        chunk_id.x,
        chunk_id.y,
        t_total.elapsed(),
        t1_elapsed,
        t2_elapsed,
        chunk_cells.len(),
        t3_elapsed,
        building_data.len(),
        t4_elapsed
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

    // NOTE: BiomeMeshData::from_image and global sample_biome are legacy
    // and still require upscaled images. Skip at large scales.
    tracing::info!("Skipping legacy biome mesh and Voronoi generation (use on-demand)");

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

    // NOTE: Exploration Voronoi seeds are deterministic and computed on-demand.
    // No pre-generation needed — see exploration_voronoi_gen.rs.

    tracing::info!(
        "✓ Full world generated in {:?} ({} land chunks out of {})",
        start.elapsed(),
        generated,
        total_chunks
    );
}

/// Selective world clearing: removes procedurally generated data while preserving player data.
///
/// DELETES:
///   - terrain.terrains (chunk meshes)
///   - terrain.terrain_biomes (biome chunk data)
///   - terrain.cells (hex cells)
///   - terrain.voronoi_zones (seeds only — cell membership computed on demand)
///   - terrain.ocean_data
///   - terrain.lake_data
///   - terrain.road_chunk_visibility (regenerated cache, not player roads)
///   - terrain.terrain_global_data
///   - buildings.trees → buildings.buildings_base WHERE category_id = 1 (Natural only)
///
/// PRESERVES:
///   - buildings.buildings_base WHERE category_id != 1 (player buildings)
///   - terrain.road_segments (player-built roads)
///   - organizations.* (orgs, territory, members)
///   - game.players, game.characters
///   - units, inventories, actions
pub async fn clear_world(map_name: &str, db_tables: &DatabaseTables) {
    tracing::info!(
        "=== Starting Selective World Clearing for '{}' ===",
        map_name
    );
    let start = std::time::Instant::now();

    let pool = &db_tables.pool;
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // 1. Natural buildings sub-tables (FK: trees → buildings_base)
    let trees_deleted = sqlx::query(
        "DELETE FROM buildings.trees WHERE building_id IN (SELECT id FROM buildings.buildings_base WHERE category_id = 1)"
    ).execute(&mut *tx).await.expect("Failed to clear trees");
    tracing::info!(
        "  🗑️  buildings.trees: {} rows",
        trees_deleted.rows_affected()
    );

    // 2. Natural buildings base
    let natural_deleted = sqlx::query("DELETE FROM buildings.buildings_base WHERE category_id = 1")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear natural buildings");
    tracing::info!(
        "  🗑️  buildings.buildings_base (Natural): {} rows",
        natural_deleted.rows_affected()
    );

    // 3. Voronoi seeds
    let vz_deleted = sqlx::query("DELETE FROM terrain.voronoi_zones")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear voronoi zones");
    tracing::info!(
        "  🗑️  terrain.voronoi_zones: {} rows",
        vz_deleted.rows_affected()
    );

    // 3b. Exploration Voronoi (only explored state — seeds are deterministic)
    let ev_deleted = sqlx::query("DELETE FROM terrain.explored_voronoi")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear explored voronoi");
    tracing::info!(
        "  🗑️  terrain.explored_voronoi: {} rows",
        ev_deleted.rows_affected()
    );

    // 4. Cells
    let cells_deleted = sqlx::query("DELETE FROM terrain.cells")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear cells");
    tracing::info!(
        "  🗑️  terrain.cells: {} rows",
        cells_deleted.rows_affected()
    );

    // 5. Terrain chunks
    let terrains_deleted = sqlx::query("DELETE FROM terrain.terrains WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrains");
    tracing::info!(
        "  🗑️  terrain.terrains: {} rows",
        terrains_deleted.rows_affected()
    );

    // 6. Biome chunks
    let biomes_deleted = sqlx::query("DELETE FROM terrain.terrain_biomes WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrain biomes");
    tracing::info!(
        "  🗑️  terrain.terrain_biomes: {} rows",
        biomes_deleted.rows_affected()
    );

    // 7. Ocean data
    let ocean_deleted = sqlx::query("DELETE FROM terrain.ocean_data WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear ocean data");
    tracing::info!(
        "  🗑️  terrain.ocean_data: {} rows",
        ocean_deleted.rows_affected()
    );

    // 8. Lake data (only if table exists may not exist on all DB versions)
    let lake_table_exists = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'terrain' AND table_name = 'lake_data')"
)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(false);

    if lake_table_exists {
        let lake_deleted = sqlx::query("DELETE FROM terrain.lake_data WHERE name = $1")
            .bind(map_name)
            .execute(&mut *tx)
            .await
            .expect("Failed to clear lake data");
        tracing::info!(
            "  🗑️  terrain.lake_data: {} rows",
            lake_deleted.rows_affected()
        );
    } else {
        tracing::info!("  ⏩  terrain.lake_data: table not found, skipping");
    }

    // 9. Road chunk visibility cache (not the road_segments themselves)
    let road_cache_deleted = sqlx::query("DELETE FROM terrain.road_chunk_visibility")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear road chunk visibility");
    tracing::info!(
        "  🗑️  terrain.road_chunk_visibility: {} rows",
        road_cache_deleted.rows_affected()
    );

    // 10. Terrain global data
    let global_deleted = sqlx::query("DELETE FROM terrain.terrain_global_data WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrain global data");
    tracing::info!(
        "  🗑️  terrain.terrain_global_data: {} rows",
        global_deleted.rows_affected()
    );

    // 11. Territory contours (only if table exists and regenerated from territory data)
    let table_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'terrain' AND table_name = 'territory_chunk_contours')"
    )
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(false);

    if table_exists {
        let contours_deleted = sqlx::query("DELETE FROM terrain.territory_chunk_contours")
            .execute(&mut *tx)
            .await
            .expect("Failed to clear territory chunk contours");
        tracing::info!(
            "  🗑️  terrain.territory_chunk_contours: {} rows",
            contours_deleted.rows_affected()
        );
    } else {
        tracing::info!("  ⏩  terrain.territory_chunk_contours: table not found, skipping");
    }

    tx.commit()
        .await
        .expect("Failed to commit clear transaction");

    // 12. Clean up .bin cache files
    let cache_patterns = [
        format!("assets/maps/{}_binarymap.bin", map_name),
        format!("assets/maps/{}_biomemap.bin", map_name),
    ];
    for pattern in &cache_patterns {
        if std::fs::remove_file(pattern).is_ok() {
            tracing::info!("  🗑️  Removed cache file: {}", pattern);
        }
    }

    tracing::info!(
        "=== ✓ World '{}' cleared in {:?} ===",
        map_name,
        start.elapsed()
    );
    tracing::info!("  Preserved: player buildings, road segments, organizations, units, players");
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

/// Check if territory cells exist but contours are missing, and regenerate if needed.
/// Called at startup after loading world globals to recover from --clear.
pub async fn ensure_territory_contours(db_tables: &DatabaseTables) {
    let contour_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM organizations.territory_contours")
            .fetch_one(&db_tables.pool)
            .await
            .unwrap_or(0);

    let territory_count: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT organization_id) FROM organizations.territory_cells")
            .fetch_one(&db_tables.pool)
            .await
            .unwrap_or(0);

    if territory_count == 0 {
        tracing::info!("✓ No territories — skipping contour check");
        return;
    }

    if contour_count > 0 {
        tracing::info!(
            "✓ Territory contours OK: {} chunks for {} organizations",
            contour_count,
            territory_count
        );
        return;
    }

    tracing::warn!(
        "⚠️ {} organizations have territory but 0 contour chunks — regenerating...",
        territory_count
    );
    regenerate_territory_contours(db_tables).await;
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

                // Log cell world positions and expected chunks for debug
                for cell in cells.iter().take(5) {
                    let hex = cell.to_hex();
                    let world_pos = grid_config.layout.hex_to_world_pos(hex);
                    let expected_chunk = shared::TerrainChunkId::from_world_pos(world_pos);
                    tracing::info!(
                        "  [DIAG] cell q={} r={} → world=({:.0},{:.0}) → chunk ({},{})",
                        cell.q, cell.r, world_pos.x, world_pos.y,
                        expected_chunk.x, expected_chunk.y
                    );
                }

                tracing::info!("  Generated {} contour chunks:", contour_chunks.len());

                // Store contours in database
                let mut stored_count = 0;
                for (chunk_id, contour_segments) in contour_chunks {
                    tracing::info!(
                        "    chunk ({},{}) → {} segments",
                        chunk_id.x, chunk_id.y, contour_segments.len()
                    );
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
