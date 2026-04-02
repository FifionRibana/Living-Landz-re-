use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::rendering::terrain::components::Terrain;
use crate::state::resources::WorldCache;

/// Handles world data messages that still go through tungstenite (roads, territory).
/// Terrain, ocean, lake, terrain global, exploration are handled by lightyear (#137).
pub fn handle_world_events(
    mut events: MessageReader<ServerEvent>,
    mut cache: Option<ResMut<WorldCache>>,
    mut commands: Commands,
    terrain_query: Query<(Entity, &Terrain)>,
) {
    for event in events.read() {
        match &event.0 {
            // Terrain, ocean, lake, terrain global data are now handled by lightyear (#137).
            ServerMessage::TerrainChunkData { .. } => {}
            ServerMessage::OceanData { .. } => {}
            ServerMessage::LakeData { .. } => {}
            ServerMessage::TerrainGlobalData { .. } => {}

            ServerMessage::RoadChunkSdfUpdate {
                terrain_name,
                chunk_id,
                road_sdf_data,
            } => {
                let Some(ref mut cache) = cache else { continue };
                info!(
                    "✓ Received road SDF update for chunk ({},{}) in terrain {}",
                    chunk_id.x, chunk_id.y, terrain_name
                );

                let storage_key = format!("{}_{}_{}", terrain_name, chunk_id.x, chunk_id.y);
                let terrain_chunk_opt = cache
                    .loaded_terrains()
                    .find(|t| t.get_storage_key() == storage_key)
                    .cloned();

                if let Some(mut updated_terrain) = terrain_chunk_opt {
                    updated_terrain.road_sdf_data = Some(road_sdf_data.clone());
                    cache.insert_terrain(&updated_terrain);

                    let terrain_id = updated_terrain.id;
                    let mut despawned = false;
                    for (entity, terrain) in terrain_query.iter() {
                        if terrain.name == *terrain_name && terrain.id == terrain_id {
                            info!(
                                "Despawning terrain for chunk ({},{}) to re-render with roads",
                                terrain_id.x, terrain_id.y
                            );
                            commands.entity(entity).despawn();
                            despawned = true;
                            break;
                        }
                    }
                    if !despawned {
                        warn!(
                            "Could not find terrain entity to despawn for chunk ({},{})",
                            chunk_id.x, chunk_id.y
                        );
                    }
                } else {
                    warn!(
                        "Received road SDF for non-loaded chunk ({},{}) in terrain {}",
                        chunk_id.x, chunk_id.y, terrain_name
                    );
                }
            }

            // Exploration data is now handled by lightyear (#137).
            ServerMessage::ExplorationMap { .. } => {}
            ServerMessage::ExplorationPatch { .. } => {
            }

            _ => {}
        }
    }
}