use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::state::resources::InventoryCache;

/// Handles inventory-related server messages.
pub fn handle_inventory_events(
    mut events: MessageReader<ServerEvent>,
    mut cache: ResMut<InventoryCache>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::InventoryData { unit_id, items } => {
                cache.set_inventory(*unit_id, items.clone());
                info!("Received inventory for unit {} ({} item types)", unit_id, items.len());
            }

            ServerMessage::InventoryUpdate {
                unit_id,
                item_id,
                quantity_delta,
                new_total,
            } => {
                cache.apply_update(*unit_id, *item_id, *new_total);
                info!(
                    "Inventory updated: unit {} item {} delta={} total={}",
                    unit_id, item_id, quantity_delta, new_total
                );
            }

            _ => {}
        }
    }
}
