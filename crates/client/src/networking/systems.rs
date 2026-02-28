// =============================================================================
// NETWORKING - Systems
// =============================================================================

use bevy::prelude::*;

use super::client::NetworkClient;
use super::events::ServerEvent;
use crate::state::resources::ConnectionStatus;
use crate::states::AppState;

/// Startup system: attempt to connect to the game server.
pub fn setup_network_client(mut commands: Commands) {
    info!("Attempting connection...");
    match NetworkClient::connect("ws://127.0.0.1:9001") {
        Ok(client) => {
            info!("Connected to server");
            commands.insert_resource(client);
        }
        Err(e) => {
            warn!("Failed to connect: {}", e);
        }
    }
}

/// Polls the NetworkClient for incoming messages and fires a `ServerEvent` for each.
/// Runs every frame, regardless of AppState (auth messages arrive during Login).
pub fn poll_server(
    network_client_opt: Option<ResMut<NetworkClient>>,
    mut events: MessageWriter<ServerEvent>,
) {
    let Some(mut client) = network_client_opt else {
        return;
    };

    let messages = client.poll_messages();

    for msg in messages {
        events.write(ServerEvent(msg));
    }
}

/// Detects when the network connection drops and transitions back to Login.
/// Also resets ConnectionStatus so the player must re-authenticate.
pub fn detect_disconnection(
    network_client_opt: Option<Res<NetworkClient>>,
    mut connection: ResMut<ConnectionStatus>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let connected = network_client_opt
        .as_ref()
        .map(|c| c.is_connected())
        .unwrap_or(false);

    // Update connection status
    if connection.connected && !connected {
        warn!("Connection to server lost!");
        connection.connected = false;
    } else if !connection.connected && connected {
        info!("Connection to server established");
        connection.connected = true;
    }

    // If we lost connection while InGame, go back to Login
    if !connected && *app_state.get() == AppState::InGame && connection.logged_in {
        warn!("Lost connection while in-game, returning to login screen");
        connection.reset_auth();
        next_app_state.set(AppState::Login);
    }
}

/// When at the Login screen without a NetworkClient, try to reconnect periodically.
pub fn attempt_reconnection(
    network_client_opt: Option<Res<NetworkClient>>,
    mut commands: Commands,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    // Already have a client — nothing to do
    if network_client_opt.is_some() {
        *timer = 0.0;
        return;
    }

    *timer += time.delta_secs();

    // Retry every 3 seconds
    if *timer < 3.0 {
        return;
    }
    *timer = 0.0;

    info!("Attempting to reconnect...");
    match NetworkClient::connect("ws://127.0.0.1:9001") {
        Ok(client) => {
            info!("Reconnected to server");
            commands.insert_resource(client);
        }
        Err(e) => {
            warn!("Reconnection failed: {}", e);
        }
    }
}
