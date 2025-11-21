use bevy::prelude::*;

use super::NetworkClient;
use crate::state::resources::{ConnectionStatus, PlayerInfo, WorldCache};

pub fn handle_server_message(
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
    mut cache: ResMut<WorldCache>,
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
            shared::protocol::ServerMessage::Pong => {}
            _ => {
                warn!("Unhandled server message: {:?}", message);
            }
        }
    }
}
