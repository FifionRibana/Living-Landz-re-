use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::rendering::terrain::components::Terrain;
use crate::state::resources::{UnitsCache, UnitsDataCache, WorldCache};

use super::db_to_slot_position;

/// Handles world data messages (terrain, biome, ocean, roads).
/// Only meaningful when InGame (world resources exist).
pub fn handle_world_events(
    mut events: MessageReader<ServerEvent>,
    mut cache: Option<ResMut<WorldCache>>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut commands: Commands,
    terrain_query: Query<(Entity, &Terrain)>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::TerrainChunkData {
                terrain_chunk_data,
                biome_chunk_data,
                cell_data,
                building_data,
                unit_data,
            } => {
                let Some(ref mut cache) = cache else { continue };
                let Some(ref mut units_cache) = units_cache else { continue };
                let Some(ref mut units_data_cache) = units_data_cache else { continue };

                info!(
                    "✓ Received terrain: {} with {} units",
                    terrain_chunk_data.name,
                    unit_data.len()
                );

                let is_update = cache.insert_terrain(&terrain_chunk_data);

                if is_update {
                    let terrain_name = &terrain_chunk_data.name;
                    let terrain_id = terrain_chunk_data.id;

                    for (entity, terrain) in terrain_query.iter() {
                        if &terrain.name == terrain_name && terrain.id == terrain_id {
                            info!(
                                "Despawning terrain entity for chunk ({},{}) to trigger re-render",
                                terrain_id.x, terrain_id.y
                            );
                            commands.entity(entity).despawn();
                            break;
                        }
                    }
                }

                for chunk_data in biome_chunk_data.iter() {
                    cache.insert_biome(chunk_data);
                }

                cache.insert_cells(&cell_data);
                cache.insert_buildings(&building_data);

                for unit in unit_data {
                    let cell = unit.current_cell;
                    let unit_id = unit.id;

                    units_cache.add_unit(cell, unit_id);

                    if let Some(slot_pos) =
                        db_to_slot_position(unit.slot_type.clone(), unit.slot_index)
                    {
                        info!(
                            "Loading unit {} at cell ({},{}) slot {:?}:{}",
                            unit_id, cell.q, cell.r, slot_pos.slot_type, slot_pos.index
                        );
                        units_cache.set_unit_slot(cell, slot_pos, unit_id);
                    }

                    units_data_cache.insert_unit(unit.clone());
                }
            }

            ServerMessage::OceanData { ocean_data } => {
                let Some(ref mut cache) = cache else { continue };
                info!("✓ Received ocean data for world: {}", ocean_data.name);
                cache.insert_ocean(ocean_data.clone());
            }

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

            _ => {}
        }
    }
}
