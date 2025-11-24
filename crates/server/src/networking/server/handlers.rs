use futures::{SinkExt, StreamExt};
use shared::{
    ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum, BuildBuildingAction, BuildRoadAction, CraftResourceAction, HarvestResourceAction, MoveUnitAction, SendMessageAction, SpecificAction, SpecificActionData, TerrainChunkData
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::action_processor::{ActionInfo, ActionProcessor};
use crate::database::client::DatabaseTables;
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
                                    handle_client_message(client_msg, session_id, &sessions, &db_tables, &action_processor).await;

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
                tracing::info!("Sending async message to session {}: {:?}", session_id, async_message);
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
                let (terrain_chunk_data, biome_chunk_data) = match db_tables
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
                responses.push(ServerMessage::TerrainChunkData {
                    terrain_chunk_data,
                    biome_chunk_data,
                    cell_data,
                    building_data,
                });
            }

            responses
        }

        ClientMessage::ActionBuildBuilding {
            player_id,
            chunk_id,
            cell,
            building_specific_type,
        } => {
            tracing::info!(
                "Player {} requested to build {:?} at chunk ({},{}) cell ({},{})",
                player_id,
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
            chunk_id,
            cell,
        } => {
            tracing::info!(
                "Player {} requested to build road at chunk ({},{}) cell ({},{})",
                player_id,
                chunk_id.x,
                chunk_id.y,
                cell.q,
                cell.r
            );
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildRoad(BuildRoadAction {
                player_id,
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
                        cell: cell.clone(),
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
        ClientMessage::Ping => vec![ServerMessage::Pong],
        _ => vec![ServerMessage::Pong],
    }
}

pub async fn broadcast_message(sessions: Sessions, msg: ServerMessage) {
    let count = sessions.count().await;
    tracing::debug!("Broadcasting message to {} sessions: {:?}", count, msg);
    // TODO: implement proper broadcasting
}
