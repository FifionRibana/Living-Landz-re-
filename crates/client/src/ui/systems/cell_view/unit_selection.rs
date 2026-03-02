use bevy::prelude::*;
use crate::ui::resources::{CellViewState, UnitSelectionState};
use crate::ui::components::SlotIndicator;
use crate::state::resources::UnitsCache;

/// Deselect unit when pressing ESC
pub fn handle_unit_deselect(
    mut unit_selection: ResMut<UnitSelectionState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !unit_selection.has_selection() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        info!("Unit deselected via ESC");
        unit_selection.clear();
    }
}

/// Deselect unit when clicking on an empty slot
pub fn handle_empty_slot_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut unit_selection: ResMut<UnitSelectionState>,
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    slot_query: Query<(&Interaction, &SlotIndicator), Changed<Interaction>>,
) {
    if cell_view_state.has_potential_drag() || cell_view_state.is_dragging() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    if mouse_button.just_released(MouseButton::Left) {
        for (interaction, slot_indicator) in &slot_query {
            if matches!(interaction, Interaction::Pressed) {
                if units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position).is_none() {
                    if unit_selection.has_selection() {
                        info!("Unit deselected by clicking empty slot");
                        unit_selection.clear();
                    }
                    break;
                }
            }
        }
    }
}
