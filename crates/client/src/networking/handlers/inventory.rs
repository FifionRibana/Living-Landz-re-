use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::state::resources::GameDataCache;
use crate::state::resources::InventoryCache;

/// Handles inventory-related server messages.
pub fn handle_inventory_events(
    mut events: MessageReader<ServerEvent>,
    mut cache: ResMut<InventoryCache>,
    game_data: Res<GameDataCache>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::InventoryData { unit_id, items } => {
                cache.set_inventory(*unit_id, items.clone());
                info!(
                    "Received inventory for unit {} ({} item types)",
                    unit_id,
                    items.len()
                );
            }

            ServerMessage::InventoryUpdate {
                unit_id,
                item_id,
                quantity_delta,
                new_total,
            } => {
                // Resolve item info from GameDataCache for proper display
                let item_info = game_data.get_item(*item_id);
                let item_name = game_data.item_name(*item_id, 1); // FR
                let item_type = item_info
                    .map(|i| {
                        shared::ItemTypeEnum::from_id(i.item_type_id)
                            .unwrap_or(shared::ItemTypeEnum::Unknown)
                    })
                    .unwrap_or(shared::ItemTypeEnum::Unknown);
                let weight = item_info.map(|i| i.weight_kg).unwrap_or(0.0);

                cache.apply_update_with_info(
                    *unit_id,
                    *item_id,
                    *new_total,
                    &item_name,
                    item_type,
                    weight,
                );
                info!(
                    "Inventory updated: unit {} — {} (id={}) delta={} total={}",
                    unit_id, item_name, item_id, quantity_delta, new_total
                );
            }

            _ => {}
        }
    }
}
