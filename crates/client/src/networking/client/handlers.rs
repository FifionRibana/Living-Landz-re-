use bevy::prelude::*;

use super::NetworkClient;
use crate::state::resources::{ActionTracker, ConnectionStatus, PlayerInfo, TrackedAction, WorldCache};

pub fn handle_server_message(
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
    mut cache: ResMut<WorldCache>,
    mut action_tracker: ResMut<ActionTracker>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    time: Res<Time>,
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
                info!("Player '{}' logged in (ID: {})", player.family_name, player.id);

                // Store character if provided
                if let Some(character_data) = character {
                    let character_name = if let Some(nickname) = &character_data.nickname {
                        format!("{} \"{}\" {}", character_data.first_name, nickname, character_data.family_name)
                    } else {
                        format!("{} {}", character_data.first_name, character_data.family_name)
                    };
                    player_info.temp_character_name = Some(character_name.clone());
                    info!("Character '{}' loaded (ID: {})", character_name, character_data.id);
                }
            }
            shared::protocol::ServerMessage::LoginError { reason } => {
                warn!("Error while logging in: {}", reason);
            }
            shared::protocol::ServerMessage::TerrainChunkData {
                terrain_chunk_data,
                biome_chunk_data,
                cell_data,
                building_data,
            } => {
                info!("✓ Received terrain: {}", terrain_chunk_data.clone().name);
                cache.insert_terrain(&terrain_chunk_data);

                for chunk_data in biome_chunk_data.iter() {
                    cache.insert_biome(chunk_data);
                }

                cache.insert_cells(&cell_data);
                cache.insert_buildings(&building_data);
            }
            shared::protocol::ServerMessage::OceanData { ocean_data } => {
                info!("✓ Received ocean data for world: {}", ocean_data.name);
                cache.insert_ocean(ocean_data);
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
                action_type,
            } => {
                info!(
                    "Action {} completed at chunk ({}, {}) cell ({}, {})",
                    action_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                // L'action est terminée, demander au serveur de rafraîchir les données du chunk
                // pour voir le nouveau bâtiment construit (ou autre résultat de l'action)
                info!("Requesting chunk data refresh for ({}, {})", chunk_id.x, chunk_id.y);

                network_client.send_message(shared::protocol::ClientMessage::RequestTerrainChunks {
                    terrain_name: "main".to_string(), // TODO: utiliser le vrai nom du terrain
                    terrain_chunk_ids: vec![chunk_id],
                });
            }
            shared::protocol::ServerMessage::Pong => {}
            _ => {
                warn!("Unhandled server message: {:?}", message);
            }
        }
    }
}
