use bevy::prelude::*;

use super::NetworkClient;
use crate::state::resources::{ConnectionStatus, WorldCache};

pub fn handle_server_message(
    mut connection: ResMut<ConnectionStatus>,
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
            shared::protocol::ServerMessage::LoginSuccess { player_id } => {
                info!("✓ Login successful, player ID: {}", player_id);
                connection.logged_in = true;
                connection.player_id = Some(player_id);
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
