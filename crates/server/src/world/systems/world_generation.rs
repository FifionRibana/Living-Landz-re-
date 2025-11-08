use crate::database::client::DatabaseTables;
use crate::database::tables;
use crate::world::components::BiomeMeshData;
use crate::world::components::TerrainMeshData;
use crate::world::resources::WorldMaps;
use bevy::prelude::*;
use hexx::HexOrientation;
use image::{ImageBuffer, Luma, Rgba};
use shared::constants;
use shared::grid::GridConfig;
use shared::{BiomeType, get_biome_color, get_biome_from_color};

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

pub async fn generate_world(map_name: &str, db_tables: &DatabaseTables) {
    tracing::info!("Starting world generation...");
    let start = std::time::Instant::now();
    tracing::info!("Using map: {}", map_name);

    // Load maps
    let maps = WorldMaps::load(map_name, 12345).expect("Failed to load world maps");

    tracing::info!("Map loaded");

    let cell_db = &db_tables.cells;
    let terrain_db = &db_tables.terrains;

    // let (terrain_mesh_data, chunk_masks, mask) = TerrainMeshData::from_image(
    //     &map_name.to_string(),
    //     &maps.binary_map,
    //     &Vec2::splat(5.),
    //     &format!("assets/maps/{}_binarymap.bin", map_name),
    // );

    // for (id, chunk) in terrain_mesh_data.chunks {
    //     terrain_db
    //         .save_terrain(chunk.to_shared_terrain_chunk_data(map_name, id))
    //         .await
    //         .expect("Failed to save terrain");
    // }

    // let biome_mesh_data = BiomeMeshData::from_image(
    //     &map_name.to_string(),
    //     &maps.biome_map,
    //     &mask, //maps.binary_map.to_luma8(),
    //     &chunk_masks,
    //     &Vec2::splat(5.),
    //     "assets/maps/",
    // );

    // for (id, chunk) in biome_mesh_data.chunks.into_iter() {
    //     terrain_db
    //         .save_terrain_biome(chunk.to_shared_biome_chunk_data(map_name, id))
    //         .await
    //         .expect("Failed to save terrain biome");
    // }

    let grid_config = &setup_grid_config();
    let sampled_cells = BiomeMeshData::sample_biome(
        &map_name.to_string(),
        &maps.biome_map.to_rgba8(),
        &Vec2::splat(5.),
        &grid_config.layout,
        "assets/maps/",
    );

    cell_db
        .save_cells(&sampled_cells)
        .await
        .expect("Failed to save cell data");

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
    TerrainMeshData::save_png_image(map_name, &format!("assets/maps/{}_binarymap.bin", map_name));
    BiomeMeshData::save_png_image(map_name, "assets/maps/");
    tracing::info!("✓ Saving {} map in {:?}", map_name, start.elapsed());
}
