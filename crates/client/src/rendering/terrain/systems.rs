use std::collections::HashSet;

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use shared::{BiomeChunkData, TerrainChunkData, TerrainChunkId, constants, get_biome_color};

use super::components::{Biome, Terrain};
use crate::networking::client::NetworkClient;
use crate::state::resources::{ConnectionStatus, WorldCache};

pub fn initialize_terrain(
    connection: Res<ConnectionStatus>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    mut world_cache_opt: Option<ResMut<WorldCache>>,
    terrains: Query<&Terrain>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };
    let Some(mut world_cache) = world_cache_opt else {
        return;
    };

    if !connection.is_ready() {
        return;
    }
}

pub fn spawn_terrain(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    terrains: Query<&Terrain>,
    biomes: Query<&Biome>,
) {
    let Some(world_cache) = world_cache_opt else {
        return;
    };

    let spawned_terrains: HashSet<_> = terrains
        .iter()
        .map(|t| TerrainChunkData::storage_key(t.name.as_str(), t.id))
        .collect();
    let spawned_biomes: HashSet<_> = biomes
        .iter()
        .map(|b| BiomeChunkData::storage_key(b.name.as_str(), b.id))
        .collect();

    for terrain in world_cache.loaded_terrains() {
        let terrain_name = terrain.clone().name;
        if spawned_terrains.contains(&terrain.get_storage_key()) {
            continue;
        }

        info!(
            "Spawning {} triangles for chunk ({},{}).",
            terrain.mesh_data.triangles.len(),
            terrain.id.x,
            terrain.id.y
        );

        let mesh_data = terrain.mesh_data.clone();

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        commands.spawn((
            Name::new(format!("Terrain_{}", terrain_name)),
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.4, 0.6, 0.3)))),
            Transform::from_translation(world_position.extend(0.0)),
            Terrain {
                name: terrain_name,
                id: terrain.id,
            },
        ));
    }

    for biome in world_cache.loaded_biomes() {
        let biome_name = biome.clone().name;
        if spawned_biomes.contains(&biome.get_storage_key()) {
            continue;
        }

        info!(
            "Spawning {} triangles for biome {:?} chunk ({},{}).",
            biome.mesh_data.triangles.len(),
            biome.id.biome,
            biome.id.x,
            biome.id.y
        );

        let mesh_data = biome.mesh_data.clone();

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let world_position = Vec2::new(
            biome.id.x as f32 * constants::CHUNK_SIZE.x,
            biome.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        // Create an atlas instead of using a new one every time
        let color = *get_biome_color(&biome.id.biome).as_color();

        commands.spawn((
            Name::new(format!("Biome_{}", biome_name)),
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(world_position.extend(0.0)),
            Biome {
                name: biome_name,
                id: biome.id,
            },
        ));
    }
}
