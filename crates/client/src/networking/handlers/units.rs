use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::state::resources::{UnitsCache, UnitsDataCache};
use crate::ui::components::{SlotIndicator, SlotUnitPortrait};
use crate::ui::systems::panels::InSlot;

/// Handles unit-related messages (slot updates, debug spawns).
pub fn handle_unit_events(
    mut events: MessageReader<ServerEvent>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut commands: Commands,
    unit_query: Query<(Entity, &InSlot, &SlotUnitPortrait)>,
    slot_query: Query<(Entity, &SlotIndicator)>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::UnitSlotUpdated {
                unit_id,
                cell,
                slot_position,
            } => {
                let Some(ref mut units_cache) = units_cache else { continue };
                info!(
                    "Unit {} slot updated at cell {:?}: {:?}",
                    unit_id, cell, slot_position
                );

                // Clear old slot for this unit if it exists
                if let Some(old_slot) = units_cache.get_unit_slot(cell, *unit_id) {
                    units_cache.remove_unit_from_slot(*cell, old_slot);
                }

                // Set new slot position if provided
                if let Some(new_slot) = slot_position {
                    units_cache.set_unit_slot(*cell, *new_slot, *unit_id);

                    let unit_entity = unit_query
                        .iter()
                        .find(|(_, _, portrait)| portrait.unit_id == *unit_id)
                        .map(|(entity, _, _)| entity);

                    let slot_entity = slot_query
                        .iter()
                        .find(|(_, indicator)| indicator.position == *new_slot)
                        .map(|(entity, _)| entity);

                    if let (Some(unit_entity), Some(slot_entity)) = (unit_entity, slot_entity) {
                        commands.entity(unit_entity).insert(InSlot(slot_entity));
                    }
                }
            }

            ServerMessage::DebugUnitSpawned { unit_data } => {
                let Some(ref mut units_cache) = units_cache else { continue };
                let Some(ref mut units_data_cache) = units_data_cache else { continue };
                info!(
                    "✓ Unit {} spawned at {:?} with slot {:?} {}",
                    unit_data.id,
                    unit_data.current_cell,
                    unit_data.slot_type,
                    unit_data.slot_index.unwrap_or(-1)
                );

                units_cache.add_unit(unit_data.current_cell, unit_data.id);
                units_data_cache.insert_unit(unit_data.clone());

                if let (Some(slot_type_str), Some(slot_index)) =
                    (&unit_data.slot_type, unit_data.slot_index)
                {
                    let slot_type = match slot_type_str.as_str() {
                        "interior" => shared::SlotType::Interior,
                        "exterior" => shared::SlotType::Exterior,
                        _ => {
                            warn!("Unknown slot type: {}", slot_type_str);
                            continue;
                        }
                    };

                    let slot_position = shared::SlotPosition {
                        slot_type,
                        index: slot_index as usize,
                    };

                    units_cache.set_unit_slot(unit_data.current_cell, slot_position, unit_data.id);
                    info!(
                        "Set slot {:?} {} for unit {}",
                        slot_type_str, slot_index, unit_data.id
                    );
                }
            }

            _ => {}
        }
    }
}
