use bevy::prelude::*;

use crate::{
    state::resources::UnitsCache,
    ui::{
        components::SlotIndicator,
        resources::{CellState, PanelEnum, UIState},
    },
};

/// Update SlotIndicator occupied_by field based on UnitsCache
pub fn update_slot_occupancy(
    cell_state: Res<CellState>,
    ui_state: Res<UIState>,
    units_cache: Res<UnitsCache>,
    mut slot_query: Query<&mut SlotIndicator>,
) {
    // Only update when cell view is active and changed
    if ui_state.panel_state != PanelEnum::CellView {
        return;
    }

    let Some(viewed_cell) = cell_state.cell() else {
        return;
    };

    // Update each slot indicator
    for mut slot_indicator in &mut slot_query {
        slot_indicator.occupied_by =
            units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position);
    }
}
