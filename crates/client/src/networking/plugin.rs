// =============================================================================
// NETWORKING - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{events::ServerEvent, handlers, systems};
use crate::states::AppState;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ServerEvent>()
            .add_systems(Startup, systems::setup_network_client)
            .add_systems(
                Update,
                (
                    // 1. Poll server → fires ServerEvents
                    systems::poll_server,
                    // 2. Auth handler — always active (login happens before InGame)
                    handlers::auth::handle_auth_events,
                    // 3. World data handlers — only InGame
                    handlers::world::handle_world_events
                        .run_if(in_state(AppState::InGame)),
                    handlers::territory::handle_territory_events
                        .run_if(in_state(AppState::InGame)),
                    handlers::actions::handle_action_events
                        .run_if(in_state(AppState::InGame)),
                    handlers::units::handle_unit_events
                        .run_if(in_state(AppState::InGame)),
                    // 4. Debug — always active (lightweight)
                    handlers::debug::handle_debug_events,
                    // 5. Connection monitoring
                    systems::detect_disconnection,
                )
                    .chain(),
            )
            // 6. Reconnection attempts — only when at Login without a client
            .add_systems(
                Update,
                systems::attempt_reconnection.run_if(in_state(AppState::Login)),
            );
    }
}
