use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::networking::client::NetworkClient;
use crate::networking::events::ServerEvent;
use crate::state::resources::PlayerInfo;
use crate::states::AppState;

/// Handles tungstenite auth responses that are NOT handled by lightyear.
/// LoginSuccess, LordData, GameData, PlayerOrganizationData are now handled
/// by lightyear receivers in lightyear_client.rs (#141).
/// This handler keeps: LordCreated, LordCreateError, LoginError, RegisterSuccess, RegisterError.
pub fn handle_auth_events(
    mut events: MessageReader<ServerEvent>,
    mut player_info: ResMut<PlayerInfo>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    for event in events.read() {
        match &event.0 {
            // Login data is now handled by lightyear Messages (#141)
            ServerMessage::LoginSuccess { .. } => {}
            ServerMessage::LordData { .. } => {}
            ServerMessage::PlayerOrganizationData { .. } => {}
            ServerMessage::GameData { .. } => {}

            // Lord creation still goes through tungstenite (character creation flow)
            ServerMessage::LordCreated { unit_data } => {
                info!(
                    "✓ Lord created: {} (ID: {})",
                    unit_data.full_name(),
                    unit_data.id,
                );
                player_info.set_lord(unit_data.clone());

                // Request inventory for the newly created lord
                if let Some(ref mut client) = network_client {
                    client.send_message(ClientMessage::RequestInventory {
                        unit_id: unit_data.id,
                    });
                }

                next_app_state.set(AppState::InGame);
            }

            ServerMessage::LordCreateError { reason } => {
                warn!("Failed to create lord: {}", reason);
            }

            ServerMessage::LoginError { reason } => {
                warn!("Error while logging in: {}", reason);
            }
            ServerMessage::RegisterSuccess { message: msg } => {
                info!("✓ Registration successful: {}", msg);
            }
            ServerMessage::RegisterError { reason } => {
                warn!("Registration failed: {}", reason);
            }
            _ => {}
        }
    }
}
