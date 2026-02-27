use bevy::prelude::*;

use crate::{
    state::resources::UnitsCache,
    ui::{
        components::SlotIndicator,
        resources::CellState,
    },
};

/// Update SlotIndicator occupied_by field based on UnitsCache
pub fn update_slot_occupancy(
    cell_state: Res<CellState>,
    units_cache: Res<UnitsCache>,
    mut slot_query: Query<&mut SlotIndicator>,
) {
    let Some(viewed_cell) = cell_state.cell() else {
        return;
    };

    // Update each slot indicator
    for mut slot_indicator in &mut slot_query {
        slot_indicator.occupied_by =
            units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position);
    }
}
