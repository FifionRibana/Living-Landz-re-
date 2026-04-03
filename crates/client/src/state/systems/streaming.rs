use std::collections::HashSet;

use bevy::prelude::*;
use shared::{
    BiomeChunkData, BiomeChunkId, BiomeTypeEnum, TerrainChunkData, TerrainChunkId, constants,
};

use crate::camera::MainCamera;
use crate::networking::client::lightyear_client::{PendingTerrainChunks, SendRequestExplorationMap, SendRequestTerrainChunks};
use crate::rendering::terrain::components::{Biome, Building, Terrain, TreeChunkMesh};
use crate::state::resources::{ConnectionStatus, StreamingConfig, WorldCache};
use crate::state::resources::streaming_config::MAX_IN_FLIGHT_CHUNKS;

pub fn request_chunks_around_camera(
    camera: Query<&Transform, With<MainCamera>>,
    connection: Res<ConnectionStatus>,
    world_cache_opt: Option<ResMut<WorldCache>>,
    mut streaming_config: ResMut<StreamingConfig>,
    time: Res<Time>,
    mut exploration_events: MessageWriter<SendRequestExplorationMap>,
    mut terrain_events: MessageWriter<SendRequestTerrainChunks>,
    pending_chunks: Res<PendingTerrainChunks>,
) {
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

    if !world_cache.is_exploration_loaded() {
        if !world_cache.is_exploration_requested() {
            exploration_events.write(SendRequestExplorationMap {
                terrain_name: "Gaulyia".to_string(),
            });
            world_cache.mark_exploration_requested();
        }
        return; // Don't request any chunks until we know what's explored
    }

    // Don't request chunks until terrain global data (biome + heightmap textures) is loaded.
    // Without it, chunks render with wrong/fallback colors.
    if !world_cache.is_terrain_global_loaded() {
        return;
    }

    let position = &transform.translation.truncate();

    let terrain_chunk_id = &TerrainChunkId {
        x: position.x.div_euclid(constants::CHUNK_SIZE.x).ceil() as i32,
        y: position.y.div_euclid(constants::CHUNK_SIZE.y).ceil() as i32,
    };

    // Prune stale entries from recently_unloaded
    streaming_config.prune_old_unloads();

    // Build set of chunk IDs already in the pending queue (received but not yet processed)
    let pending_ids: HashSet<TerrainChunkId> = pending_chunks.0
        .iter()
        .map(|msg| msg.chunk_id)
        .collect();

    let mut to_request = Vec::new();

    for dx in -streaming_config.view_radius..=streaming_config.view_radius {
        for dy in -streaming_config.view_radius..=streaming_config.view_radius {
            let id = TerrainChunkId {
                x: terrain_chunk_id.x + dx,
                y: terrain_chunk_id.y + dy,
            };

            if !world_cache.is_chunk_near_explored(&id) && !world_cache.is_chunk_coastal(&id) {
                continue;
            }

            // Skip if already loaded
            if world_cache.is_terrain_loaded("Gaulyia", &id) {
                continue;
            }

            // Skip if already in the pending queue (received, awaiting processing)
            if pending_ids.contains(&id) {
                continue;
            }

            // Skip if recently unloaded (anti-thrash)
            if streaming_config.was_recently_unloaded(&id) {
                continue;
            }

            // Skip if recently requested and not yet timed out
            let should_request = match world_cache.get_terrain_requested_time("Gaulyia", &id) {
                Some(requested_at) => {
                    time.elapsed_secs() - requested_at > streaming_config.request_timeout
                }
                None => true,
            };

            if should_request {
                to_request.push(id);
            }
        }
    }

    // Sort by distance from camera (closest first)
    to_request.sort_unstable_by_key(|id| {
        let dx = id.x - terrain_chunk_id.x;
        let dy = id.y - terrain_chunk_id.y;
        dx * dx + dy * dy
    });

    // Cap in-flight requests to prevent flooding
    to_request.truncate(MAX_IN_FLIGHT_CHUNKS);

    if !to_request.is_empty() {
        for id in &to_request {
            world_cache.mark_terrain_requested_at("Gaulyia", id, time.elapsed_secs());
        }
        terrain_events.write(SendRequestTerrainChunks {
            terrain_name: "Gaulyia".to_string(),
            chunk_ids: to_request,
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
    tree_mesh_entities: Query<(Entity, &TreeChunkMesh)>,
    world_cache_opt: Option<ResMut<WorldCache>>,
    mut streaming_config: ResMut<StreamingConfig>,
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

    let (removed_keys, removed_chunks) =
        world_cache.unload_distant_terrain(terrain_chunk_id, streaming_config.unload_distance);

    // Track recently unloaded chunks to prevent re-request thrashing
    for chunk in &removed_chunks {
        streaming_config.mark_unloaded(chunk.id);
    }

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

    // Despawn per-chunk tree meshes for unloaded chunks
    let removed_chunk_ids: HashSet<_> = removed_chunks.iter().map(|c| c.id).collect();
    for (entity, tree_mesh) in tree_mesh_entities.iter() {
        if removed_chunk_ids.contains(&tree_mesh.chunk_id) {
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