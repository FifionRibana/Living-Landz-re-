use bevy::prelude::*;

use super::NetworkClient;
use crate::rendering::terrain::components::Terrain;
use crate::rendering::territory::{
    TerritoryBorderCellsDebug, TerritoryBorderSdfCache, TerritoryContourCache,
};
use crate::state::resources::{
    ActionTracker, ConnectionStatus, CurrentOrganization, PlayerInfo, TrackedAction, UnitsCache,
    UnitsDataCache, WorldCache,
};
use crate::states::AppState;
use crate::ui::components::{SlotIndicator, SlotUnitPortrait};
use crate::ui::systems::panels::InSlot;
use shared::SlotPosition;
use shared::SlotType;

/// Helper function to convert database slot data to SlotPosition
fn db_to_slot_position(slot_type: Option<String>, slot_index: Option<i32>) -> Option<SlotPosition> {
    match (slot_type, slot_index) {
        (Some(type_str), Some(index)) if index >= 0 => {
            let slot_type_enum = match type_str.as_str() {
                "interior" => SlotType::Interior,
                "exterior" => SlotType::Exterior,
                _ => return None,
            };
            Some(SlotPosition {
                slot_type: slot_type_enum,
                index: index as usize,
            })
        }
        _ => None,
    }
}

pub fn handle_server_message(
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
    mut cache: Option<ResMut<WorldCache>>,
    mut action_tracker: Option<ResMut<ActionTracker>>,
    mut current_organization: Option<ResMut<CurrentOrganization>>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut territory_border_cache: ResMut<TerritoryBorderSdfCache>,
    mut territory_contour_cache: ResMut<TerritoryContourCache>,
    mut next_app_state: ResMut<NextState<AppState>>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    // time: Res<Time>,
    mut commands: Commands,
    terrain_query: Query<(Entity, &Terrain)>,
    unit_query: Query<(Entity, &InSlot, &SlotUnitPortrait)>,
    slot_query: Query<(Entity, &SlotIndicator)>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };

    let messages = network_client.poll_messages();

    if !messages.is_empty() {
        info!("Received {} messages from server", messages.len());
    }

    for message in messages {
        match message {
            shared::protocol::ServerMessage::LoginSuccess { player, character } => {
                info!("✓ Login successful, player ID: {}", player.id);
                connection.logged_in = true;
                connection.player_id = Some(player.id as u64);

                // Store player name from received data
                player_info.temp_player_name = Some(player.family_name.clone());
                info!(
                    "Player '{}' logged in (ID: {})",
                    player.family_name, player.id
                );

                // Store character if provided
                if let Some(character_data) = character {
                    let character_name = if let Some(nickname) = &character_data.nickname {
                        format!(
                            "{} \"{}\" {}",
                            character_data.first_name, nickname, character_data.family_name
                        )
                    } else {
                        format!(
                            "{} {}",
                            character_data.first_name, character_data.family_name
                        )
                    };
                    player_info.temp_character_name = Some(character_name.clone());
                    info!(
                        "Character '{}' loaded (ID: {})",
                        character_name, character_data.id
                    );
                }

                // Switch to game (AppState::InGame) after successful login
                next_app_state.set(AppState::InGame);
            }
            shared::protocol::ServerMessage::LoginError { reason } => {
                warn!("Error while logging in: {}", reason);
                // Error is displayed by the UI system (login_interactions.rs checks LoginError text)
            }
            shared::protocol::ServerMessage::RegisterSuccess { message: msg } => {
                info!("✓ Registration successful: {}", msg);
                // Success message is displayed by UI, then auto-switches to login panel
                // Note: The UI will handle the timer to switch back
            }
            shared::protocol::ServerMessage::RegisterError { reason } => {
                warn!("Registration failed: {}", reason);
                // Error is displayed by the UI system (register_interactions.rs checks RegisterError text)
            }
            shared::protocol::ServerMessage::TerrainChunkData {
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
                    terrain_chunk_data.clone().name,
                    unit_data.len()
                );

                let is_update = cache.insert_terrain(&terrain_chunk_data);

                // If this is an update (chunk already existed), despawn the old terrain entity
                // so it can be re-spawned with the new data (e.g., road textures)
                if is_update {
                    let terrain_name = &terrain_chunk_data.name;
                    let terrain_id = terrain_chunk_data.id;

                    for (entity, terrain) in terrain_query.iter() {
                        if &terrain.name == terrain_name && terrain.id == terrain_id {
                            info!(
                                "Despawning terrain entity for chunk ({},{}) to trigger re-render with updated data",
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

                // Load units and their slot positions into cache
                for unit in unit_data {
                    let cell = unit.current_cell;
                    let unit_id = unit.id;

                    // Add unit to the cache
                    units_cache.add_unit(cell, unit_id);

                    // If unit has a slot position, add it to slot occupancy
                    if let Some(slot_pos) =
                        db_to_slot_position(unit.slot_type.clone(), unit.slot_index)
                    {
                        info!(
                            "Loading unit {} at cell ({},{}) slot {:?}:{}",
                            unit_id, cell.q, cell.r, slot_pos.slot_type, slot_pos.index
                        );
                        units_cache.set_unit_slot(cell, slot_pos, unit_id);
                    }

                    // Store full unit data
                    units_data_cache.insert_unit(unit);
                }
            }
            shared::protocol::ServerMessage::OceanData { ocean_data } => {
                let Some(ref mut cache) = cache else { continue };
                info!("✓ Received ocean data for world: {}", ocean_data.name);
                cache.insert_ocean(ocean_data);
            }
            shared::protocol::ServerMessage::RoadChunkSdfUpdate {
                terrain_name,
                chunk_id,
                road_sdf_data,
            } => {
                let Some(ref mut cache) = cache else { continue };
                info!(
                    "✓ Received road SDF update for chunk ({},{}) in terrain {}",
                    chunk_id.x, chunk_id.y, terrain_name
                );

                // Find the terrain chunk in cache and clone it (ends immutable borrow)
                let storage_key = format!("{}_{}_{}", terrain_name, chunk_id.x, chunk_id.y);
                let terrain_chunk_opt = cache
                    .loaded_terrains()
                    .find(|t| t.get_storage_key() == storage_key)
                    .cloned();

                if let Some(mut updated_terrain) = terrain_chunk_opt {
                    info!(
                        "Updating road SDF for terrain chunk ({},{}) in {}",
                        chunk_id.x, chunk_id.y, terrain_name
                    );

                    updated_terrain.road_sdf_data = Some(road_sdf_data);

                    // Update in cache FIRST
                    cache.insert_terrain(&updated_terrain);

                    // ALWAYS despawn the existing terrain entity to force re-render with updated SDF
                    let terrain_id = updated_terrain.id;
                    let mut despawned = false;
                    for (entity, terrain) in terrain_query.iter() {
                        if terrain.name == terrain_name && terrain.id == terrain_id {
                            info!(
                                "Despawning terrain entity for chunk ({},{}) to re-render with updated roads",
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
            shared::protocol::ServerMessage::TerritoryContourUpdate { chunk_id, contours } => {
                info!(
                    "✓ Received {} territory contours for chunk ({},{})",
                    contours.len(),
                    chunk_id.x,
                    chunk_id.y
                );

                // Add each contour to the cache
                for contour_data in contours {
                    territory_contour_cache.add_contour(
                        chunk_id,
                        contour_data.organization_id,
                        contour_data
                            .segments
                            .iter()
                            .map(|s| s.to_contour_segment())
                            .collect(),
                        Color::linear_rgba(
                            contour_data.border_color.r,
                            contour_data.border_color.g,
                            contour_data.border_color.b,
                            contour_data.border_color.a,
                        ),
                        Color::linear_rgba(
                            contour_data.fill_color.r,
                            contour_data.fill_color.g,
                            contour_data.fill_color.b,
                            contour_data.fill_color.a,
                        ),
                    );
                }
            }
            shared::protocol::ServerMessage::TerritoryBorderSdfUpdate {
                chunk_id,
                border_sdf_data_list,
            } => {
                info!(
                    "✓ Received territory border SDF for chunk ({},{}) [DEPRECATED]",
                    chunk_id.x, chunk_id.y
                );

                // Insert into territory border cache
                // The process_territory_border_sdf_data system will pick it up and spawn the layers
                territory_border_cache
                    .chunks
                    .insert((chunk_id.x, chunk_id.y), border_sdf_data_list);
            }
            shared::protocol::ServerMessage::TerritoryBorderCells {
                organization_id,
                border_cells,
            } => {
                info!(
                    "✓ Received {} border cells for organization {}",
                    border_cells.len(),
                    organization_id
                );

                // Store border cells for visualization
                commands.insert_resource(TerritoryBorderCellsDebug {
                    organization_id,
                    border_cells,
                });
            }
            shared::protocol::ServerMessage::ActionStatusUpdate {
                action_id,
                player_id,
                chunk_id,
                cell,
                status,
                action_type,
                completion_time,
            } => {
                let Some(ref mut action_tracker) = action_tracker else { continue };
                info!(
                    "Action {} status update: {:?} for player {} at chunk ({}, {}) cell ({}, {})",
                    action_id, status, player_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                let tracked_action = TrackedAction {
                    action_id,
                    player_id,
                    chunk_id,
                    cell,
                    action_type,
                    status,
                    completion_time,
                };

                action_tracker.update_action(tracked_action);
            }
            shared::protocol::ServerMessage::ActionCompleted {
                action_id,
                chunk_id,
                cell,
                action_type: _,
            } => {
                info!(
                    "Action {} completed at chunk ({}, {}) cell ({}, {})",
                    action_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                // L'action est terminée, demander au serveur de rafraîchir les données du chunk
                // pour voir le nouveau bâtiment construit (ou autre résultat de l'action)
                info!(
                    "Requesting chunk data refresh for ({}, {})",
                    chunk_id.x, chunk_id.y
                );

                network_client.send_message(
                    shared::protocol::ClientMessage::RequestTerrainChunks {
                        terrain_name: "Gaulyia".to_string(), // TODO: utiliser le vrai nom du terrain
                        terrain_chunk_ids: vec![chunk_id],
                    },
                );
            }
            // Debug organization messages
            shared::protocol::ServerMessage::DebugOrganizationCreated {
                organization_id,
                name,
            } => {
                info!(
                    "✓ Organization '{}' created with ID {}",
                    name, organization_id
                );
            }
            shared::protocol::ServerMessage::DebugOrganizationDeleted { organization_id } => {
                info!("✓ Organization {} deleted", organization_id);
            }
            shared::protocol::ServerMessage::DebugUnitSpawned { unit_data } => {
                let Some(ref mut units_cache) = units_cache else { continue };
                let Some(ref mut units_data_cache) = units_data_cache else { continue };
                info!(
                    "✓ Unit {} spawned at {:?} with slot {:?} {}",
                    unit_data.id,
                    unit_data.current_cell,
                    unit_data.slot_type,
                    unit_data.slot_index.unwrap_or(-1)
                );

                // Add unit to cache
                units_cache.add_unit(unit_data.current_cell, unit_data.id);

                // Store full unit data in units_data_cache
                units_data_cache.insert_unit(unit_data.clone());

                // If we have a slot position, update the slot occupancy
                if let (Some(slot_type_str), Some(slot_index)) =
                    (&unit_data.slot_type, unit_data.slot_index)
                {
                    let slot_type = match slot_type_str.as_str() {
                        "interior" => shared::SlotType::Interior,
                        "exterior" => shared::SlotType::Exterior,
                        _ => {
                            warn!("Unknown slot type: {}", slot_type_str);
                            return;
                        }
                    };

                    let slot_position = shared::SlotPosition {
                        slot_type,
                        index: slot_index as usize,
                    };

                    units_cache.set_unit_slot(unit_data.current_cell, slot_position, unit_data.id);
                    info!(
                        "Set slot {:?} {} for unit {}",
                        slot_type_str, slot_index, unit_data.id
                    );
                }
            }
            shared::protocol::ServerMessage::OrganizationAtCell { cell, organization } => {
                let Some(ref mut current_organization) = current_organization else { continue };
                current_organization.update(cell, organization);
            }
            shared::protocol::ServerMessage::DebugError { reason } => {
                warn!("Debug error: {}", reason);
            }

            shared::protocol::ServerMessage::UnitSlotUpdated {
                unit_id,
                cell,
                slot_position,
            } => {
                let Some(ref mut units_cache) = units_cache else { continue };
                info!(
                    "Unit {} slot updated at cell {:?}: {:?}",
                    unit_id, cell, slot_position
                );

                // Clear old slot for this unit if it exists
                if let Some(old_slot) = units_cache.get_unit_slot(&cell, unit_id) {
                    units_cache.remove_unit_from_slot(cell, old_slot);
                }

                // Set new slot position if provided
                if let Some(new_slot) = slot_position {
                    units_cache.set_unit_slot(cell, new_slot, unit_id);

                    // Find the UI entity de the unit (if spawned)
                    let unit_entity = unit_query
                        .iter()
                        .find(|(_, _, portrait)| portrait.unit_id == unit_id)
                        .map(|(entity, _, _)| entity);

                    // Find the entity of the target slot
                    let slot_entity = slot_query
                        .iter()
                        .find(|(_, indicator)| indicator.position == new_slot)
                        .map(|(entity, _)| entity);

                    // update the relation between both entity
                    if let (Some(unit_entity), Some(slot_entity)) = (unit_entity, slot_entity) {
                        commands.entity(unit_entity).insert(InSlot(slot_entity));
                    }
                }
            }

            shared::protocol::ServerMessage::Pong => {}
            _ => {
                warn!("Unhandled server message: {:?}", message);
            }
        }
    }
}