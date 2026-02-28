use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// Handles debug and misc messages (organization debug, errors, pong).
pub fn handle_debug_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::DebugOrganizationCreated {
                organization_id,
                name,
            } => {
                info!(
                    "✓ Organization '{}' created with ID {}",
                    name, organization_id
                );
            }
            ServerMessage::DebugOrganizationDeleted { organization_id } => {
                info!("✓ Organization {} deleted", organization_id);
            }
            ServerMessage::DebugError { reason } => {
                warn!("Debug error: {}", reason);
            }
            ServerMessage::Pong => {}
            _ => {}
        }
    }
}
