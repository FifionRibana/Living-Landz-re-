use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// Inventory messages are now handled by lightyear (#137).
pub fn handle_inventory_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::InventoryData { .. } => {}
            ServerMessage::InventoryUpdate { .. } => {}
            _ => {}
        }
    }
}
