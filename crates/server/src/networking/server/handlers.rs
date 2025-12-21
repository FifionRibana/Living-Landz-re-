use futures::{SinkExt, StreamExt};
use shared::{
    ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum, BuildBuildingAction, BuildRoadAction, CraftResourceAction, HarvestResourceAction, MoveUnitAction, SendMessageAction, SpecificAction, SpecificActionData, TerrainChunkData
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::action_processor::{ActionInfo, ActionProcessor};
use crate::database::client::DatabaseTables;
use crate::units::NameGenerator;
use shared::protocol::{ClientMessage, ServerMessage};

use super::super::Sessions;

/// Helper function to add action to both DB and cache
async fn add_action_and_cache(
    action_table: &crate::database::tables::ScheduledActionsTable,
    action_processor: &ActionProcessor,
    action_data: &ActionData,
    action_type: ActionTypeEnum,
) -> Result<u64, String> {
    // Add to database
    let action_id = action_table.add_scheduled_action(action_data).await?;

    // Add to cache
    let completion_time = action_data.base_data.start_time + (action_data.base_data.duration_ms / 1000);
    action_processor.add_action(ActionInfo {
        action_id,
        player_id: action_data.base_data.player_id,
        chunk_id: action_data.base_data.chunk.clone(),
        cell: action_data.base_data.cell.clone(),
        action_type,
        status: ActionStatusEnum::Pending,
        start_time: action_data.base_data.start_time,
        duration_ms: action_data.base_data.duration_ms,
        completion_time,
    }).await;

    Ok(action_id)
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sessions: Sessions,
    db_tables: Arc<DatabaseTables>,
    action_processor: Arc<ActionProcessor>,
    name_generator: Arc<NameGenerator>,
) {
    tracing::info!("New connection from {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!("WebSocket handshake error: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let session_id = rand::random::<u64>();

    // Créer un channel pour les messages asynchrones (depuis action_processor)
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    sessions.insert(session_id, addr, tx).await;

    // Traiter les messages entrants et sortants
    loop {
        tokio::select! {
            // Messages entrants du client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        tracing::info!("Received message from {}: {} bytes", addr, data.len());
                        match bincode::decode_from_slice(&data[..], bincode::config::standard()) {
                            Ok((client_msg, _)) => {
                                tracing::debug!("Received: {:?}", client_msg);

                                let responses =
                                    handle_client_message(client_msg, session_id, &sessions, &db_tables, &action_processor, &name_generator).await;

                                // Envoyer les réponses DIRECTEMENT (comme avant)
                                for response in responses {
                                    let response_data =
                                        bincode::encode_to_vec(&response, bincode::config::standard())
                                            .unwrap();
                                    if let Err(e) = write.send(Message::Binary(response_data.into())).await {
                                        tracing::error!("Failed to send direct response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to deserialize message from {}\n{}", addr, e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            // Messages asynchrones depuis action_processor
            Some(async_message) = rx.recv() => {
                // Log le type de message sans afficher les données complètes (évite de logger des MB de SDF)
                let message_type = match &async_message {
                    ServerMessage::LoginSuccess { .. } => "LoginSuccess",
                    ServerMessage::LoginError { .. } => "LoginError",
                    ServerMessage::TerrainChunkData { .. } => "TerrainChunkData",
                    ServerMessage::OceanData { .. } => "OceanData",
                    ServerMessage::RoadChunkSdfUpdate { chunk_id, .. } => {
                        tracing::info!("Sending RoadChunkSdfUpdate to session {} for chunk ({},{})", session_id, chunk_id.x, chunk_id.y);
                        "RoadChunkSdfUpdate"
                    },
                    ServerMessage::ActionStatusUpdate { .. } => "ActionStatusUpdate",
                    ServerMessage::ActionCompleted { .. } => "ActionCompleted",
                    ServerMessage::ActionSuccess { .. } => "ActionSuccess",
                    ServerMessage::ActionError { .. } => "ActionError",
                    ServerMessage::DebugOrganizationCreated { .. } => "DebugOrganizationCreated",
                    ServerMessage::DebugOrganizationDeleted { .. } => "DebugOrganizationDeleted",
                    ServerMessage::DebugUnitSpawned { .. } => "DebugUnitSpawned",
                    ServerMessage::OrganizationAtCell { .. } => "OrganizationAtCell",
                    ServerMessage::DebugError { .. } => "DebugError",
                    ServerMessage::UnitSlotUpdated { .. } => "UnitSlotUpdated",
                    ServerMessage::Pong => "Pong",
                };

                if !matches!(async_message, ServerMessage::RoadChunkSdfUpdate { .. }) {
                    tracing::debug!("Sending async {} to session {}", message_type, session_id);
                }

                let data = bincode::encode_to_vec(&async_message, bincode::config::standard()).unwrap();
                if let Err(e) = write.send(Message::Binary(data.into())).await {
                    tracing::error!("Failed to send async message: {}", e);
                    break;
                }
            }
        }
    }

    sessions.remove(&session_id).await;
    tracing::info!("Connection closed: {}", addr);
}

async fn handle_client_message(
    msg: ClientMessage,
    session_id: u64,
    sessions: &Sessions,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    name_generator: &NameGenerator,
) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::Login { username } => {
            tracing::info!("Session {} attempting to log in as {}", session_id, username);

            // Try to get or create the player in the database
            match shared::types::game::methods::get_or_create_player(
                &db_tables.pool,
                &username,
                1, // Default language_id (could be configurable)
                "default_location", // Default origin location
                None, // No motto by default
            ).await {
                Ok(player) => {
                    tracing::info!("Player {} logged in successfully with DB ID {}", username, player.id);

                    // Associate session with player_id
                    sessions.authenticate_session(session_id, player.id as u64).await;

                    // Get or create a default character
                    match shared::types::game::methods::get_or_create_default_character(
                        &db_tables.pool,
                        player.id,
                        &player.family_name,
                    ).await {
                        Ok(character) => {
                            tracing::info!("Character {} {} loaded/created", character.first_name, character.family_name);

                            // Convert to protocol types (without timestamps)
                            let player_data = shared::protocol::PlayerData {
                                id: player.id,
                                family_name: player.family_name,
                                language_id: player.language_id,
                                coat_of_arms_id: player.coat_of_arms_id,
                                motto: player.motto,
                                origin_location: player.origin_location,
                            };

                            let character_data = shared::protocol::CharacterData {
                                id: character.id,
                                player_id: character.player_id,
                                first_name: character.first_name,
                                family_name: character.family_name,
                                second_name: character.second_name,
                                nickname: character.nickname,
                                coat_of_arms_id: character.coat_of_arms_id,
                                image_id: character.image_id,
                                motto: character.motto,
                            };

                            vec![ServerMessage::LoginSuccess {
                                player: player_data,
                                character: Some(character_data),
                            }]
                        }
                        Err(e) => {
                            tracing::error!("Failed to get/create character: {}", e);
                            vec![ServerMessage::LoginError {
                                reason: format!("Character creation error: {}", e)
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get/create player {}: {}", username, e);
                    vec![ServerMessage::LoginError {
                        reason: format!("Database error: {}", e)
                    }]
                }
            }
        }
        ClientMessage::RequestTerrainChunks {
            terrain_name,
            terrain_chunk_ids,
        } => {
            let mut responses = Vec::new();
            let terrain_name_ref = &terrain_name;
            for terrain_chunk_id in terrain_chunk_ids.iter() {
                let cell_data = match db_tables.cells.load_chunk_cells(terrain_chunk_id).await {
                    Ok(cells_data) => cells_data,
                    _ => {
                        vec![]
                    }
                };
                let building_data = match db_tables
                    .buildings
                    .load_chunk_buildings(terrain_chunk_id)
                    .await
                {
                    Ok(building_data) => building_data,
                    _ => {
                        vec![]
                    }
                };
                let unit_data = match db_tables
                    .units
                    .load_chunk_units(*terrain_chunk_id)
                    .await
                {
                    Ok(units) => {
                        tracing::info!("Loaded {} units for chunk ({},{})", units.len(), terrain_chunk_id.x, terrain_chunk_id.y);
                        units
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load units for chunk ({},{}): {}", terrain_chunk_id.x, terrain_chunk_id.y, e);
                        vec![]
                    }
                };
                let (mut terrain_chunk_data, biome_chunk_data) = match db_tables
                    .terrains
                    .load_terrain(terrain_name_ref, terrain_chunk_id)
                    .await
                {
                    Ok((Some(terrain_chunk_data), Some(biome_chunk_data))) => {
                        (terrain_chunk_data, biome_chunk_data)
                    }
                    Ok((Some(terrain_chunk_data), None)) => (terrain_chunk_data, vec![]),
                    Ok((None, Some(biome_chunk_data))) => (
                        TerrainChunkData {
                            name: terrain_name.clone(),
                            id: terrain_chunk_id.clone(),
                            ..TerrainChunkData::default()
                        },
                        biome_chunk_data,
                    ),
                    Ok((None, None)) => {
                        tracing::error!(
                            "DB error for chunk ({},{}) in terrain {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            terrain_name_ref
                        );

                        (
                            TerrainChunkData {
                                name: terrain_name.clone(),
                                id: terrain_chunk_id.clone(),
                                ..TerrainChunkData::default()
                            },
                            vec![],
                        )
                    }
                    Err(e) => {
                        tracing::error!(
                            "DB error for chunk ({},{}) in terrain {}: {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            terrain_name_ref,
                            e
                        );

                        (
                            TerrainChunkData {
                                name: terrain_name.clone(),
                                id: terrain_chunk_id.clone(),
                                ..TerrainChunkData::default()
                            },
                            vec![],
                        )
                    }
                };

                // Send terrain data first (without roads to keep message size down)
                responses.push(ServerMessage::TerrainChunkData {
                    terrain_chunk_data: terrain_chunk_data.clone(),
                    biome_chunk_data,
                    cell_data,
                    building_data,
                    unit_data,
                });

                // Charger et générer les routes pour ce chunk (envoyé séparément)
                // Utilise la table de visibilité pour charger tous les segments qui traversent ce chunk
                tracing::info!("Attempting to load road segments for chunk ({},{})", terrain_chunk_id.x, terrain_chunk_id.y);
                match db_tables.road_segments
                    .load_road_segments_by_chunk_new(terrain_chunk_id.x, terrain_chunk_id.y)
                    .await
                {
                    Ok(road_segments) if !road_segments.is_empty() => {
                        tracing::info!(
                            "Generating road SDF for chunk ({},{}) with {} segments",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            road_segments.len()
                        );

                        use crate::road::{RoadConfig, compute_intersections, generate_road_sdf};

                        let config = RoadConfig::default();
                        let intersections = compute_intersections(&road_segments, &config);
                        let road_sdf = generate_road_sdf(&road_segments, &intersections, &config, terrain_chunk_id.x, terrain_chunk_id.y);

                        tracing::info!(
                            "✓ Road SDF generated: {}x{} with {} intersections",
                            config.sdf_resolution.x,
                            config.sdf_resolution.y,
                            intersections.len()
                        );

                        // Send road data separately to avoid message size limits
                        responses.push(ServerMessage::RoadChunkSdfUpdate {
                            terrain_name: terrain_name.clone(),
                            chunk_id: terrain_chunk_id.clone(),
                            road_sdf_data: road_sdf,
                        });
                    }
                    Ok(road_segments) => {
                        tracing::info!("No road segments found for chunk ({},{}) (loaded {} segments)", terrain_chunk_id.x, terrain_chunk_id.y, road_segments.len());
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load road segments for chunk ({},{}): {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            e
                        );
                    }
                }
            }

            responses
        }

        ClientMessage::ActionBuildBuilding {
            player_id,
            chunk_id,
            cell,
            building_type,
        } => {
            let building_specific_type = building_type.to_specific_type();
            tracing::info!(
                "Player {} requested to build {:?} ({:?}) at chunk ({},{}) cell ({},{})",
                player_id,
                building_type,
                building_specific_type,
                chunk_id.x,
                chunk_id.y,
                cell.q,
                cell.r
            );
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildBuilding(BuildBuildingAction {
                player_id,
                chunk_id: chunk_id.clone(),
                cell: cell.clone(),
                building_type,
                building_specific_type,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),
                    action_type: ActionTypeEnum::BuildBuilding,
                    action_specific_type: ActionSpecificTypeEnum::BuildBuilding,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::BuildBuilding).await {
                Ok(action_id) => {
                    tracing::info!("Scheduled build building action with ID {}", action_id);

                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::BuildBuilding,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionBuildRoad {
            player_id,
            start_cell,
            end_cell,
        } => {
            tracing::info!(
                "Player {} requested to build road from ({},{}) to ({},{})",
                player_id,
                start_cell.q,
                start_cell.r,
                end_cell.q,
                end_cell.r
            );
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildRoad(BuildRoadAction {
                player_id,
                start_cell: start_cell.clone(),
                end_cell: end_cell.clone(),
            });

            // Calculer le chunk à partir de la cellule de départ
            use crate::database::tables::RoadSegmentsTable;
            let chunk_id = RoadSegmentsTable::cell_to_chunk_id(&start_cell);

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: start_cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: start_cell.clone(),
                    action_type: ActionTypeEnum::BuildRoad,
                    action_specific_type: ActionSpecificTypeEnum::BuildRoad,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::BuildRoad).await {
                Ok(action_id) => {
                    tracing::info!("Scheduled build road action {}", action_id);

                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: start_cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::BuildRoad,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionCraftResource {
            player_id,
            chunk_id,
            cell,
            recipe_id,
            quantity,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::CraftResource(CraftResourceAction {
                player_id,
                recipe_id,
                chunk_id: chunk_id.clone(),
                cell: cell.clone(),
                quantity,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),
                    action_type: ActionTypeEnum::CraftResource,
                    action_specific_type: ActionSpecificTypeEnum::CraftResource,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::CraftResource).await {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::CraftResource,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionHarvestResource {
            player_id,
            chunk_id,
            cell,
            resource_specific_type,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::HarvestResource(HarvestResourceAction {
                player_id,
                chunk_id: chunk_id.clone(),
                cell: cell.clone(),
                resource_specific_type,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),
                    action_type: ActionTypeEnum::HarvestResource,
                    action_specific_type: ActionSpecificTypeEnum::HarvestResource,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::HarvestResource).await {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::HarvestResource,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionMoveUnit {
            player_id,
            unit_id,
            chunk_id,
            cell,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::MoveUnit(MoveUnitAction {
                player_id,
                unit_id,
                chunk_id: chunk_id.clone(),
                cell: cell.clone(),
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),
                    action_type: ActionTypeEnum::MoveUnit,
                    action_specific_type: ActionSpecificTypeEnum::MoveUnit,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::MoveUnit).await {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::MoveUnit,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::MoveUnitToSlot {
            unit_id,
            cell,
            from_slot,
            to_slot,
        } => {
            let mut responses = Vec::new();

            tracing::info!(
                "Moving unit {} from slot {:?} to {:?} at cell {:?}",
                unit_id,
                from_slot,
                to_slot,
                cell
            );

            // Convert SlotPosition to DB format
            let slot_type_str = match to_slot.slot_type {
                shared::SlotType::Interior => "interior",
                shared::SlotType::Exterior => "exterior",
            };
            let slot_index = to_slot.index as i32;

            // Update database
            match db_tables.units.update_slot_position(
                unit_id,
                Some(slot_type_str.to_string()),
                Some(slot_index),
            ).await {
                Ok(_) => {
                    tracing::info!("Unit {} slot updated in database", unit_id);

                    // Broadcast success to all clients
                    responses.push(ServerMessage::UnitSlotUpdated {
                        unit_id,
                        cell: cell.clone(),
                        slot_position: Some(to_slot),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to update unit slot in database: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to update slot position: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::AssignUnitToSlot {
            unit_id,
            cell,
            slot,
        } => {
            let mut responses = Vec::new();

            tracing::info!(
                "Assigning unit {} to slot {:?}:{} at cell {:?}",
                unit_id,
                slot.slot_type,
                slot.index,
                cell
            );

            // Convert SlotPosition to DB format
            let slot_type_str = match slot.slot_type {
                shared::SlotType::Interior => "interior",
                shared::SlotType::Exterior => "exterior",
            };
            let slot_index = slot.index as i32;

            // Update database
            match db_tables.units.update_slot_position(
                unit_id,
                Some(slot_type_str.to_string()),
                Some(slot_index),
            ).await {
                Ok(_) => {
                    tracing::info!("Unit {} slot assigned in database", unit_id);

                    // Broadcast success to all clients
                    responses.push(ServerMessage::UnitSlotUpdated {
                        unit_id,
                        cell: cell.clone(),
                        slot_position: Some(slot),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to assign unit slot in database: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to assign slot position: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionSendMessage {
            player_id,
            chunk_id,
            cell,
            receivers,
            content,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::SendMessage(SendMessageAction {
                player_id,
                receivers,
                content,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell.clone(),
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),
                    action_type: ActionTypeEnum::SendMessage,
                    action_specific_type: ActionSpecificTypeEnum::SendMessage,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(action_table, action_processor, &action_data, ActionTypeEnum::SendMessage).await {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::SendMessage,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::RequestOceanData { world_name } => {
            tracing::info!("=== CLIENT REQUESTED OCEAN DATA ===");
            tracing::info!("World name: {}", world_name);

            match db_tables.ocean_data.load_ocean_data(&world_name).await {
                Ok(Some(ocean_data)) => {
                    tracing::info!("Ocean data loaded from database:");
                    tracing::info!("  - width: {}", ocean_data.width);
                    tracing::info!("  - height: {}", ocean_data.height);
                    tracing::info!("  - sdf_values: {} bytes", ocean_data.sdf_values.len());
                    tracing::info!("  - heightmap_values: {} bytes", ocean_data.heightmap_values.len());

                    tracing::info!("Encoding ocean data with bincode for network transmission...");
                    let encoded_size = bincode::encode_to_vec(&ocean_data, bincode::config::standard())
                        .map(|v| v.len())
                        .unwrap_or(0);
                    tracing::info!("Encoded size: {} bytes ({:.2} MB)", encoded_size, encoded_size as f64 / 1_000_000.0);

                    tracing::info!("✓ Sending ocean data to client");
                    vec![ServerMessage::OceanData { ocean_data }]
                }
                Ok(None) => {
                    tracing::warn!("No ocean data found for world: {}", world_name);
                    vec![]
                }
                Err(e) => {
                    tracing::error!("Failed to load ocean data: {}", e);
                    vec![]
                }
            }
        }

        // ====================================================================
        // DEBUG COMMANDS
        // ====================================================================

        ClientMessage::DebugCreateOrganization {
            name,
            organization_type,
            cell,
            parent_organization_id,
        } => {
            tracing::info!("DEBUG: Creating organization '{}' of type {:?} at {:?}", name, organization_type, cell);

            // First, create a leader unit for the organization
            let (first_name, last_name, gender) = {
                use rand::Rng;
                let mut rng = rand::rng();

                // Generate random gender
                let is_male = rng.gen_bool(0.5);
                let gender_str = if is_male { "male" } else { "female" };

                // Use NameGenerator to get realistic names based on gender
                let (first_name, last_name) = name_generator.generate_random_name(Some(is_male));

                (first_name, last_name, gender_str.to_string())
            };

            // Create the leader unit
            let founder_unit_result = db_tables.units.create_unit(
                None,
                first_name.clone(),
                last_name.clone(),
                gender.clone(),
                cell.clone(),
                shared::TerrainChunkId { x: 0, y: 0 },
                shared::ProfessionEnum::Merchant,
            ).await;

            match founder_unit_result {
                Ok(founder_unit_id) => {
                    let request = shared::CreateOrganizationRequest {
                        name: name.clone(),
                        organization_type,
                        headquarters_cell: Some(cell.clone()),
                        parent_organization_id,
                        founder_unit_id,
                    };

                    match db_tables.organizations.create_organization(request).await {
                        Ok(org_id) => {
                            tracing::info!("✓ Organization created with ID {}", org_id);

                            // Add some territory cells around headquarters
                            for neighbor in cell.neighbors() {
                                let _ = db_tables.organizations.add_territory_cell(org_id, &neighbor).await;
                            }
                            let _ = db_tables.organizations.add_territory_cell(org_id, &cell).await;

                            vec![ServerMessage::DebugOrganizationCreated {
                                organization_id: org_id,
                                name,
                            }]
                        }
                        Err(e) => {
                            tracing::error!("✗ Failed to create organization: {}", e);
                            vec![ServerMessage::DebugError {
                                reason: format!("Failed to create organization: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("✗ Failed to create leader unit: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to create leader unit: {}", e),
                    }]
                }
            }
        }

        ClientMessage::DebugDeleteOrganization { organization_id } => {
            tracing::info!("DEBUG: Deleting organization ID {}", organization_id);

            match sqlx::query("DELETE FROM organizations.organizations WHERE id = $1")
                .bind(organization_id as i64)
                .execute(&db_tables.pool)
                .await
            {
                Ok(_) => {
                    tracing::info!("✓ Organization {} deleted", organization_id);
                    vec![ServerMessage::DebugOrganizationDeleted { organization_id }]
                }
                Err(e) => {
                    tracing::error!("✗ Failed to delete organization: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to delete organization: {}", e),
                    }]
                }
            }
        }

        ClientMessage::DebugSpawnUnit { cell } => {
            tracing::info!("DEBUG: Spawning random unit at {:?}", cell);

            // Generate random unit data using NameGenerator
            let (first_name, last_name, gender, profession) = {
                use rand::Rng;
                let mut rng = rand::rng();

                // Generate random gender (true = male, false = female)
                let is_male = rng.gen_bool(0.5);
                let gender_str = if is_male { "male" } else { "female" };

                // Use NameGenerator to get realistic names based on gender
                let (first_name, last_name) = name_generator.generate_random_name(Some(is_male));

                // Generate random profession
                let profession_id: i16 = rng.gen_range(1..=16);
                let profession = shared::ProfessionEnum::from_id(profession_id)
                    .unwrap_or(shared::ProfessionEnum::Farmer);

                (first_name, last_name, gender_str.to_string(), profession)
            };

            match db_tables.units.create_unit(
                None, // No player
                first_name.clone(),
                last_name.clone(),
                gender.clone(),
                cell.clone(),
                shared::TerrainChunkId { x: 0, y: 0 }, // Default chunk
                profession,
            ).await {
                Ok(unit_id) => {
                    tracing::info!("✓ Unit spawned: {} {} ({}, ID: {})", first_name, last_name, gender, unit_id);
                    vec![ServerMessage::DebugUnitSpawned {
                        unit_id,
                        cell,
                    }]
                }
                Err(e) => {
                    tracing::error!("✗ Failed to spawn unit: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to spawn unit: {}", e),
                    }]
                }
            }
        }

        ClientMessage::RequestOrganizationAtCell { cell } => {
            tracing::debug!("Checking organization at cell {:?}", cell);

            // Query to find which organization owns this cell
            match sqlx::query_as::<_, (i64,)>(
                "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
            )
            .bind(cell.q)
            .bind(cell.r)
            .fetch_optional(&db_tables.pool)
            .await
            {
                Ok(Some((org_id,))) => {
                    // Load organization summary
                    match db_tables.organizations.load_organization(org_id as u64).await {
                        Ok(org_data) => {
                            let summary = shared::OrganizationSummary {
                                id: org_data.id,
                                name: org_data.name,
                                organization_type: org_data.organization_type,
                                leader_unit_id: org_data.leader_unit_id,
                                population: org_data.population,
                                emblem_url: org_data.emblem_url,
                            };
                            vec![ServerMessage::OrganizationAtCell {
                                cell,
                                organization: Some(summary),
                            }]
                        }
                        Err(e) => {
                            tracing::error!("Failed to load organization: {}", e);
                            vec![ServerMessage::OrganizationAtCell {
                                cell,
                                organization: None,
                            }]
                        }
                    }
                }
                Ok(None) => {
                    // No organization at this cell
                    vec![ServerMessage::OrganizationAtCell {
                        cell,
                        organization: None,
                    }]
                }
                Err(e) => {
                    tracing::error!("Database error checking organization: {}", e);
                    vec![ServerMessage::OrganizationAtCell {
                        cell,
                        organization: None,
                    }]
                }
            }
        }

        ClientMessage::Ping => vec![ServerMessage::Pong],
        _ => vec![ServerMessage::Pong],
    }
}

pub async fn broadcast_message(sessions: Sessions, msg: ServerMessage) {
    let count = sessions.count().await;
    tracing::debug!("Broadcasting message to {} sessions: {:?}", count, msg);
    // TODO: implement proper broadcasting
}
