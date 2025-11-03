use crate::database::TerrainDatabase;
use crate::world::components::BiomeMeshData;
use crate::world::components::TerrainMeshData;
use crate::world::resources::WorldMaps;
use bevy::prelude::*;
use image::{ImageBuffer, Luma, Rgba};
use shared::{BiomeType, get_biome_color, get_biome_from_color};

pub async fn generate_world(map_name: &str, terrain_db: &TerrainDatabase) {
    tracing::info!("Starting world generation...");
    let start = std::time::Instant::now();
    tracing::info!("Using map: {}", map_name);

    // Load maps
    let maps = WorldMaps::load(
        map_name,
        12345,
    )
    .expect("Failed to load world maps");

    tracing::info!("Map loaded");

    let terrain_mesh_data = TerrainMeshData::from_image(
        &map_name.to_string(),
        &maps.binary_map,
        &Vec2::splat(20.),
        &format!("assets/maps/{}_binarymap.bin", map_name),
    );

    for (id, chunk) in terrain_mesh_data.chunks {
        terrain_db
            .save_terrain(chunk.to_shared_terrain_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain");
    }

    let biome_mesh_data = BiomeMeshData::from_image(
        &map_name.to_string(),
        &maps.biome_map,
        &maps.binary_map.to_luma8(),
        &Vec2::splat(20.),
        "assets/maps/",
    );

    for (id, chunk) in biome_mesh_data.chunks.into_iter() {
        terrain_db
            .save_terrain_biome(chunk.to_shared_biome_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain biome");
    }

    tracing::info!("✓ Generated {} map in {:?}", map_name, start.elapsed());

    // let world_config = maps.config.clone();
}

pub async fn clear_world(map_name: &str, terrain_db: &TerrainDatabase) {
    tracing::info!("Starting world clearing...");
    let start = std::time::Instant::now();
    tracing::info!("Using map: {}", map_name);

    terrain_db
        .clear_terrain(map_name)
        .await
        .expect("Failed to clear terrain");

    // TODO: Add clear_terrain for biomes
    tracing::info!("✓ Clearing {} map in {:?}", map_name, start.elapsed());
}

pub async fn save_world_to_png(map_name: &str) {
    tracing::info!("Starting saving...");
    let start = std::time::Instant::now();
    TerrainMeshData::save_png_image(
        map_name,
        &format!("assets/maps/{}_binarymap.bin", map_name),
    );
    BiomeMeshData::save_png_image(
        map_name,
        "assets/maps/",
    );
    tracing::info!("✓ Saving {} map in {:?}", map_name, start.elapsed());
}