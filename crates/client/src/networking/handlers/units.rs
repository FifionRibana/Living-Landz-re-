use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// Unit messages are now handled by lightyear (#137).
pub fn handle_unit_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::UnitSlotUpdated { .. } => {}
            ServerMessage::UnitProfessionChanged { .. } => {}
            ServerMessage::UnitPositionUpdated { .. } => {}
            ServerMessage::UnitWorkStatusUpdate { .. } => {}
            ServerMessage::PopulationChanged { .. } => {}
            _ => {}
        }
    }
}
