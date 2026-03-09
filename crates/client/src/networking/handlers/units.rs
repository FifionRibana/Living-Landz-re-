use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::state::resources::{PlayerInfo, UnitsCache, UnitsDataCache};
use crate::ui::components::{SlotIndicator, SlotUnitPortrait};
use crate::ui::systems::panels::InSlot;

/// Handles unit-related messages (slot updates, debug spawns).
pub fn handle_unit_events(
    mut events: MessageReader<ServerEvent>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut player_info: ResMut<PlayerInfo>,
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
                let Some(ref mut units_cache) = units_cache else {
                    continue;
                };
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
                let Some(ref mut units_cache) = units_cache else {
                    continue;
                };
                let Some(ref mut units_data_cache) = units_data_cache else {
                    continue;
                };
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

            ServerMessage::UnitProfessionChanged {
                unit_id,
                new_profession,
            } => {
                let Some(ref mut units_data_cache) = units_data_cache else {
                    continue;
                };
                info!(
                    "✓ Unit {} profession changed to {:?}",
                    unit_id, new_profession
                );

                // Update the cached unit data with the new profession
                if let Some(unit) = units_data_cache.get_unit_mut(*unit_id) {
                    unit.profession = *new_profession;
                    info!(
                        "Updated cached unit {} ({} {}) profession to {:?}",
                        unit_id, unit.first_name, unit.last_name, new_profession
                    );
                } else {
                    warn!("Unit {} not found in cache for profession update", unit_id);
                }
            }
            ServerMessage::UnitPositionUpdated {
                unit_id,
                from_cell,
                from_chunk,
                to_cell,
                to_chunk,
            } => {
                info!(
                    "Unit {} moved from ({},{}) to ({},{})",
                    unit_id, from_cell.q, from_cell.r, to_cell.q, to_cell.r
                );

                // Mettre à jour le cache des unités par cellule
                if let Some(ref mut cache) = units_cache {
                    cache.remove_unit(*unit_id);
                    cache.add_unit(*to_cell, *unit_id);
                }

                // Mettre à jour les données de l'unité
                if let Some(ref mut data_cache) = units_data_cache
                    && let Some(unit) = data_cache.get_unit_mut(*unit_id)
                {
                    unit.current_cell = *to_cell;
                    unit.current_chunk = *to_chunk;
                }

                // Si c'est le lord, mettre à jour PlayerInfo
                if let Some(ref mut lord) = player_info.lord
                    && lord.id == *unit_id
                {
                    lord.current_cell = *to_cell;
                    lord.current_chunk = *to_chunk;
                    info!("Lord position updated to ({},{})", to_cell.q, to_cell.r);
                }
            }

            ServerMessage::PopulationChanged {
                organization_id,
                new_population,
                immigrant,
            } => {
                info!(
                    "Population changed for org {}: now {} inhabitants",
                    organization_id, new_population
                );

                // Mettre à jour l'organisation du joueur si c'est la sienne
                if let Some(ref mut org) = player_info.organization {
                    if org.id == *organization_id {
                        org.population = *new_population;
                        info!("Updated player org population to {}", new_population);
                    }
                }

                // L'unité immigrante est aussi envoyée via DebugUnitSpawned
                // (qui est déjà géré par ce handler), donc pas besoin de la traiter ici
            }
            
            _ => {}
        }
    }
}
