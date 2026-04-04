use bevy::prelude::*;

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

/// Disconnect: reset auth, return to Login.
pub fn handle_disconnect_click(
    query: Query<&Interaction, (Changed<Interaction>, With<DisconnectButton>)>,
    mut connection: ResMut<ConnectionStatus>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Player chose to disconnect");
            connection.reset_auth();
            next_overlay.set(Overlay::None);
            next_app_state.set(AppState::Login);
        }
    }
}
