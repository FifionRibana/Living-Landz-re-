use bevy::prelude::*;

use crate::networking::client::NetworkClient;
use crate::state::resources::ConnectionStatus;
use crate::states::{AppState, Overlay};

use super::setup::{DisconnectButton, ResumeButton};

/// Resume: close the pause overlay.
pub fn handle_resume_click(
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Resuming game");
            next_overlay.set(Overlay::None);
        }
    }
}

/// Disconnect: reset auth, drop connection, return to Login.
pub fn handle_disconnect_click(
    query: Query<&Interaction, (Changed<Interaction>, With<DisconnectButton>)>,
    mut connection: ResMut<ConnectionStatus>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    network_client: Option<Res<NetworkClient>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Player chose to disconnect");

            // Reset auth
            connection.reset_auth();

            // Remove network client resource (connection thread will eventually stop)
            if network_client.is_some() {
                commands.remove_resource::<NetworkClient>();
            }

            // Close overlay first, then switch to Login
            next_overlay.set(Overlay::None);
            next_app_state.set(AppState::Login);
        }
    }
}
