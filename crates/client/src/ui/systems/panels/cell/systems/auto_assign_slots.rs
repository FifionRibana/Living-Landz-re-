use bevy::prelude::*;
use crate::ui::resources::{CellState, PanelEnum, UIState};
use crate::state::resources::{UnitsCache, WorldCache};
use crate::networking::client::NetworkClient;
use shared::{SlotPosition, SlotType, SlotConfiguration, BiomeTypeEnum};
use shared::protocol::ClientMessage;
// use shared::grid::GridCell;

/// Auto-assign units without slots to random free slots when entering cell view
pub fn auto_assign_unslotted_units(
    ui_state: Res<UIState>,
    cell_state: Res<CellState>,
    world_cache: Res<WorldCache>,
    mut units_cache: ResMut<UnitsCache>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    // Only run when cell view is active
    if ui_state.panel_state != PanelEnum::CellView {
        return;
    }

    let Some(viewed_cell) = cell_state.cell() else {
        return;
    };

    // Get all units at this cell
    let Some(cell_units) = units_cache.get_units_at_cell(&viewed_cell) else {
        return;
    };

    // Find units without slot assignments
    let unslotted_units: Vec<u64> = cell_units
        .iter()
        .filter(|&&unit_id| {
            units_cache.get_unit_slot(&viewed_cell, unit_id).is_none()
        })
        .copied()
        .collect();

    if unslotted_units.is_empty() {
        return;
    }

    info!(
        "Found {} units without slots at cell ({}, {}), auto-assigning...",
        unslotted_units.len(), viewed_cell.q, viewed_cell.r
    );

    // Get slot configuration for this cell based on building type or terrain
    let cell_data = world_cache.get_cell(&viewed_cell);
    let building = world_cache.get_building(&viewed_cell);

    let biome = cell_data
        .map(|c| c.biome)
        .unwrap_or(BiomeTypeEnum::Undefined);

    let slot_config = if let Some(building_data) = building {
        // Try to get slot config from building type first
        if let Some(building_type) = building_data.to_building_type() {
            SlotConfiguration::for_building_type(building_type)
        } else {
            // Fallback to terrain type for trees or unknown buildings
            SlotConfiguration::for_terrain_type(biome)
        }
    } else {
        // No building, use terrain type
        SlotConfiguration::for_terrain_type(biome)
    };

    // Get currently occupied slots
    let occupied_slots = units_cache.get_occupied_slots(&viewed_cell);
    let occupied_positions: Vec<SlotPosition> = occupied_slots
        .iter()
        .map(|(pos, _)| *pos)
        .collect();

    // Generate list of all possible slots
    let mut available_slots = Vec::new();

    // Add interior slots
    for i in 0..slot_config.interior_slots() {
        let slot_pos = SlotPosition {
            slot_type: SlotType::Interior,
            index: i,
        };
        if !occupied_positions.contains(&slot_pos) {
            available_slots.push(slot_pos);
        }
    }

    // Add exterior slots
    for i in 0..slot_config.exterior_slots() {
        let slot_pos = SlotPosition {
            slot_type: SlotType::Exterior,
            index: i,
        };
        if !occupied_positions.contains(&slot_pos) {
            available_slots.push(slot_pos);
        }
    }

    // Assign units to available slots
    for (idx, &unit_id) in unslotted_units.iter().enumerate() {
        if idx >= available_slots.len() {
            warn!(
                "Not enough slots to auto-assign all units! {} units need slots but only {} available",
                unslotted_units.len(),
                available_slots.len()
            );
            break;
        }

        let slot_pos = available_slots[idx];

        // Update local cache
        units_cache.set_unit_slot(viewed_cell, slot_pos, unit_id);

        info!(
            "Auto-assigned unit {} to slot {:?}:{}",
            unit_id, slot_pos.slot_type, slot_pos.index
        );

        // Send to server for persistence
        if let Some(ref mut client) = network_client {
            client.send_message(ClientMessage::AssignUnitToSlot {
                unit_id,
                cell: viewed_cell,
                slot: slot_pos,
            });
        }
    }
}
