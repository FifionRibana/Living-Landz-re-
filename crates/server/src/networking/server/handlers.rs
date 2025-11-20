use futures::{SinkExt, StreamExt};
use shared::{
    ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum, BuildBuildingAction, BuildRoadAction, CraftResourceAction, HarvestResourceAction, MoveUnitAction, SendMessageAction, SpecificAction, SpecificActionData, TerrainChunkData
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::database::client::DatabaseTables;
use shared::protocol::{ClientMessage, ServerMessage};

use super::super::Sessions;

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sessions: Sessions,
    db_tables: Arc<DatabaseTables>,
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
    let player_id = rand::random::<u64>();

    sessions.insert(player_id, addr);

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                tracing::info!("Received message from {}: {} bytes", addr, data.len());
                match bincode::decode_from_slice(&data[..], bincode::config::standard()) {
                    Ok((client_msg, _)) => {
                        tracing::debug!("Received: {:?}", client_msg);

                        let responses =
                            handle_client_message(client_msg, player_id, &db_tables).await;

                        for response in responses {
                            let response_data =
                                bincode::encode_to_vec(&response, bincode::config::standard())
                                    .unwrap();
                            let _ = write.send(Message::Binary(response_data.into())).await;
                        }
                    }
                    Err(e) => {
                        // } else {
                        tracing::warn!("Failed to deserialize message from {}\n{}", addr, e);
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    sessions.remove(&player_id).await;
    tracing::info!("Connection closed: {}", addr);
}

async fn handle_client_message(
    msg: ClientMessage,
    player_id: u64,
    db_tables: &DatabaseTables,
) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::Login { username } => {
            tracing::info!("Player {} logged in as {}", player_id, username);
            vec![ServerMessage::LoginSuccess { player_id }]
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
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildBuilding(BuildBuildingAction {
                player_id,
                chunk_id: chunk_id.clone(),
                cell: cell.clone(),
                building_specific_type,
            });

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::BuildBuilding,
                    action_specific_type: ActionSpecificTypeEnum::BuildBuilding,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

            responses
        }
        ClientMessage::ActionBuildRoad {
            player_id,
            chunk_id,
            cell,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildRoad(BuildRoadAction {
                player_id,
                chunk_id,
                cell,
            });

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::BuildRoad,
                    action_specific_type: ActionSpecificTypeEnum::BuildRoad,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

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

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::CraftResource,
                    action_specific_type: ActionSpecificTypeEnum::CraftResource,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

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
                chunk_id,
                cell,
                resource_specific_type,
            });

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::HarvestResource,
                    action_specific_type: ActionSpecificTypeEnum::HarvestResource,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

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

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::MoveUnit,
                    action_specific_type: ActionSpecificTypeEnum::MoveUnit,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

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

            action_table.add_scheduled_action(&ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id.clone(),
                    cell: cell.clone(),

                    action_type: ActionTypeEnum::SendMessage,
                    action_specific_type: ActionSpecificTypeEnum::SendMessage,

                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    duration_ms: specific_data.duration_ms(&ActionContext {
                        player_id,
                        grid_cell: cell.clone(),
                    }),
                    completion_time: 0,

                    // action_type: ActionType::BuildBuilding,
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            });

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
