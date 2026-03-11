use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::networking::client::NetworkClient;
use crate::networking::events::ServerEvent;
use crate::state::resources::{ConnectionStatus, GameDataCache, PlayerInfo};
use crate::states::AppState;

/// Handles authentication responses (login/register) AND lord lifecycle.
/// Runs at all times since auth happens before InGame.
pub fn handle_auth_events(
    mut events: MessageReader<ServerEvent>,
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut network_client: Option<ResMut<NetworkClient>>,
    mut game_data_cache: ResMut<GameDataCache>,
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

                // Ne PAS naviguer ici — on attend LordData pour décider
                // (le serveur envoie LordData juste après LoginSuccess)
            }

            // ── NOUVEAU : LordData reçu après LoginSuccess ──
            ServerMessage::LordData { lord } => {
                if let Some(lord_data) = lord {
                    info!(
                        "✓ Lord loaded: {} (ID: {}) at ({},{})",
                        lord_data.full_name(),
                        lord_data.id,
                        lord_data.current_cell.q,
                        lord_data.current_cell.r,
                    );
                    player_info.set_lord(lord_data.clone());

                    // Demander l'inventaire du Lord
                    if let Some(ref mut client) = network_client {
                        client.send_message(ClientMessage::RequestInventory {
                            unit_id: lord_data.id,
                        });
                        info!("Requested inventory for lord {}", lord_data.id);
                    }

                    next_app_state.set(AppState::InGame);
                } else {
                    info!("No lord found — entering character creation");
                    next_app_state.set(AppState::CharacterCreation);
                }
            }

            // ── NOUVEAU : Lord créé avec succès ──
            ServerMessage::LordCreated { unit_data } => {
                info!(
                    "✓ Lord created: {} (ID: {})",
                    unit_data.full_name(),
                    unit_data.id,
                );
                player_info.set_lord(unit_data.clone());

                // Demander l'inventaire du Lord nouvellement créé
                if let Some(ref mut client) = network_client {
                    client.send_message(ClientMessage::RequestInventory {
                        unit_id: unit_data.id,
                    });
                }

                next_app_state.set(AppState::InGame);
            }

            // ── NOUVEAU : Échec de création du lord ──
            ServerMessage::LordCreateError { reason } => {
                warn!("Failed to create lord: {}", reason);
                // Rester sur CharacterCreation — TODO: afficher l'erreur dans l'UI
            }

            ServerMessage::PlayerOrganizationData { organization } => {
                if let Some(org) = organization {
                    info!(
                        "✓ Player organization loaded: {} (ID: {})",
                        org.name, org.id
                    );
                    player_info.organization = Some(org.clone());
                } else {
                    info!("No organization found for player");
                }
            }

            ServerMessage::GameData { payload } => {
                let item_count = payload.items.len();
                let recipe_count = payload.recipes.len();
                game_data_cache.load_from_payload(payload.clone());
                info!(
                    "✓ Game data loaded: {} items, {} recipes, {} costs, {} yields",
                    item_count,
                    recipe_count,
                    game_data_cache.construction_costs.len(),
                    game_data_cache.harvest_yields.len(),
                );
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
