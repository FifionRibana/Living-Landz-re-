use std::collections::HashSet;

use bevy::prelude::*;
use shared::{
    BiomeChunkData, BiomeChunkId, BiomeTypeEnum, TerrainChunkData, TerrainChunkId, constants,
};

use crate::camera::MainCamera;
use crate::networking::client::NetworkClient;
use crate::rendering::terrain::components::{Biome, Building, Terrain};
// use crate::rendering::terrain::components::Terrain;
use crate::state::resources::{ConnectionStatus, StreamingConfig, WorldCache};

pub fn request_chunks_around_camera(
    camera: Query<&Transform, With<MainCamera>>,
    // terrains: Query<&Terrain>,
    connection: Res<ConnectionStatus>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    world_cache_opt: Option<ResMut<WorldCache>>,
    mut streaming_config: ResMut<StreamingConfig>,
    time: Res<Time>,
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

    let Ok(transform) = camera.single() else {
        return;
    };

    if time.elapsed_secs() - streaming_config.last_request < streaming_config.request_cooldown {
        return;
    }

    let position = &transform.translation.truncate();

    let terrain_chunk_id = &TerrainChunkId {
        x: position.x.div_euclid(constants::CHUNK_SIZE.x).ceil() as i32,
        y: position.y.div_euclid(constants::CHUNK_SIZE.y).ceil() as i32,
    };

    let mut to_request = Vec::new();

    for dx in -streaming_config.view_radius..=streaming_config.view_radius {
        for dy in -streaming_config.view_radius..=streaming_config.view_radius {
            let id = TerrainChunkId {
                x: terrain_chunk_id.x + dx,
                y: terrain_chunk_id.y + dy,
            };
   
            if !world_cache.is_chunk_explored(&id) {
                continue;
            }

            if !world_cache.is_terrain_loaded("Gaulyia", &id) {
                let should_request = match world_cache.get_terrain_requested_time("Gaulyia", &id) {
                    Some(requested_at) => {
                        time.elapsed_secs() - requested_at > streaming_config.request_timeout
                    }
                    None => true,
                };

                if should_request {
                    world_cache.mark_terrain_requested_at("Gaulyia", &id, time.elapsed_secs());
                    to_request.push(id);
                }
            }

            for biome_type in BiomeTypeEnum::iter() {
                let biome_id = BiomeChunkId::from_terrain(&id, biome_type);

                if !world_cache.is_biome_loaded("Gaulyia", &biome_id)
                    && !world_cache.is_biome_requested("Gaulyia", &biome_id)
                {
                    world_cache.mark_biome_requested("Gaulyia", &biome_id);
                    // No additional requests as biomes are retrieved with RequstTerrainChunk too.
                }
            }
        }
    }

    to_request.sort_unstable_by_key(|id| {
        let dx = id.x - terrain_chunk_id.x;
        let dy = id.y - terrain_chunk_id.y;
        dx * dx + dy * dy
    });

    if !to_request.is_empty() {
        info!("Requesting {} chunks", to_request.len());
        network_client.send_message(shared::protocol::ClientMessage::RequestTerrainChunks {
            terrain_name: "Gaulyia".to_string(),
            terrain_chunk_ids: to_request,
        });
        streaming_config.last_request = time.elapsed_secs();
    }
}

pub fn unload_distant_chunks(
    mut commands: Commands,
    camera: Query<&Transform, With<MainCamera>>,
    terrain_entities: Query<(Entity, &Terrain)>,
    biome_entities: Query<(Entity, &Biome)>,
    building_entities: Query<(Entity, &Building)>,
    world_cache_opt: Option<ResMut<WorldCache>>,
    streaming_config: ResMut<StreamingConfig>,
) {
    let Some(mut world_cache) = world_cache_opt else {
        return;
    };

    let Ok(transform) = camera.single() else {
        return;
    };

    let position = &transform.translation.truncate();

    let terrain_chunk_id = &TerrainChunkId {
        x: position.x.div_euclid(constants::CHUNK_SIZE.x).ceil() as i32,
        y: position.y.div_euclid(constants::CHUNK_SIZE.y).ceil() as i32,
    };

    let (removed_keys, _) =
        world_cache.unload_distant_terrain(terrain_chunk_id, streaming_config.unload_distance);

    let mut entities: HashSet<_> = terrain_entities
        .iter()
        .map(|(e, t)| (e, TerrainChunkData::storage_key(&t.name, t.id)))
        .collect();

    entities.retain(|(_, key)| removed_keys.contains(key));

    let to_despawn: Vec<Entity> = entities.into_iter().map(|(e, _)| e).collect();
    if !to_despawn.is_empty() {
        info!("Despawning {} terrain entities", to_despawn.len());
        for entity in to_despawn {
            commands.entity(entity).despawn();
        }
    }

    for biome_type in BiomeTypeEnum::iter() {
        let biome_chunk_id = &BiomeChunkId::from_terrain(terrain_chunk_id, biome_type);
        let (removed_biome_keys, _) =
            world_cache.unload_distant_biome(biome_chunk_id, streaming_config.unload_distance);

        let mut entities: HashSet<_> = biome_entities
            .iter()
            .map(|(e, b)| (e, BiomeChunkData::storage_key(&b.name, b.id)))
            .collect();

        entities.retain(|(_, key)| removed_biome_keys.contains(key));

        for (entity, _) in entities {
            commands.entity(entity).despawn();
        }
    }

    let (removed_building_keys, _) =
        world_cache.unload_distant_building(terrain_chunk_id, streaming_config.unload_distance);

    if !removed_building_keys.is_empty() {
        let mut count = 0;
        for (entity, building) in building_entities.iter() {
            if removed_building_keys.contains(&building.id) {
                commands.entity(entity).despawn();
                count += 1;
            }
        }
        if count > 0 {
            info!("Despawned {} building entities", count);
        }
    }
}
