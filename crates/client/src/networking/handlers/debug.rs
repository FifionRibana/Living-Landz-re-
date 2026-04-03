use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// Debug messages are now handled by lightyear (#137).
pub fn handle_debug_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::DebugOrganizationCreated { .. } => {}
            ServerMessage::DebugOrganizationDeleted { .. } => {}
            ServerMessage::DebugError { .. } => {}
            ServerMessage::Pong => {}
            _ => {}
        }
    }
}
