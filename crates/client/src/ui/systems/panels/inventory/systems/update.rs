use bevy::prelude::*;

use crate::state::resources::{InventoryCache, PlayerInfo};
use crate::ui::systems::panels::components::InventoryItemRow;

/// Update inventory item quantities and weights in real-time.
/// Runs every frame when inventory panel is visible and cache has changed.
pub fn update_inventory_panel(
    inventory_cache: Res<InventoryCache>,
    player_info: Res<PlayerInfo>,
    row_query: Query<(&InventoryItemRow, &Children)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
) {
    if !inventory_cache.is_changed() {
        return;
    }

    let Some(lord) = &player_info.lord else {
        return;
    };

    let Some(items) = inventory_cache.get_inventory(lord.id) else {
        return;
    };

    for (row, row_children) in &row_query {
        // Find the matching item in cache
        let Some(item) = items.iter().find(|i| i.item_id == row.item_id) else {
            continue;
        };

        // The row has two children: left column (name + type) and right column (qty + weight)
        // We need to update the right column's texts
        for child in row_children.iter() {
            // Right column is the second child
            let Ok(grandchildren) = children_query.get(child) else {
                continue;
            };

            let mut text_index = 0;
            for grandchild in grandchildren.iter() {
                if let Ok(mut text) = text_query.get_mut(grandchild) {
                    text_index += 1;
                    // In the right column: first text is quantity, second is weight
                    // But we also match left column texts - check by content pattern
                    let current = text.as_str();
                    if current.starts_with('x') {
                        // Quantity text like "x8"
                        **text = format!("x{}", item.quantity);
                    } else if current.ends_with("kg") {
                        // Weight text like "12.0 kg"
                        **text = format!(
                            "{:.1} kg",
                            item.weight_kg * item.quantity as f32
                        );
                    }
                }
            }
        }
    }
}