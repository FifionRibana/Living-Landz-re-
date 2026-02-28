use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::state::resources::{ConnectionStatus, PlayerInfo};
use crate::states::AppState;

/// Handles authentication responses (login/register).
/// Runs at all times since auth happens before InGame.
pub fn handle_auth_events(
    mut events: MessageReader<ServerEvent>,
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::LoginSuccess { player, character } => {
                info!("✓ Login successful, player ID: {}", player.id);
                connection.logged_in = true;
                connection.player_id = Some(player.id as u64);

                player_info.temp_player_name = Some(player.family_name.clone());
                info!(
                    "Player '{}' logged in (ID: {})",
                    player.family_name, player.id
                );

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

                next_app_state.set(AppState::InGame);
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
