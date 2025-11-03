use bevy::prelude::*;

use super::client::NetworkClient;

pub fn setup_network_client(mut commands: Commands) {
    info!("Attempting connection...");
    match NetworkClient::connect("ws://127.0.0.1:9001") {
        Ok(mut client) => {
            info!("Connected to server");
            client.send_message(shared::protocol::ClientMessage::Login {
                username: "Player".to_string(),
            });
            commands.insert_resource(client);
        }
        Err(e) => {
             warn!("Failed to connect: {}", e);
        }
    }
}