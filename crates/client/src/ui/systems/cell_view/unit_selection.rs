use bevy::prelude::*;
use crate::ui::resources::CellViewState;
use crate::ui::components::SlotIndicator;
use crate::state::resources::UnitsCache;

/// Deselect unit when pressing ESC
pub fn handle_unit_deselect(
    mut cell_view_state: ResMut<CellViewState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Only process when a unit is selected
    if cell_view_state.selected_unit.is_none() {
        return;
    }

    // Deselect on ESC key
    if keyboard.just_pressed(KeyCode::Escape) {
        info!("Unit deselected via ESC");
        cell_view_state.selected_unit = None;
    }
}

/// Deselect unit when clicking on an empty slot
pub fn handle_empty_slot_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cell_view_state: ResMut<CellViewState>,
    units_cache: Res<UnitsCache>,
    slot_query: Query<(&Interaction, &SlotIndicator), Changed<Interaction>>,
) {
    // Only process in cell view mode and when not in drag mode
    if !cell_view_state.is_active || cell_view_state.has_potential_drag() || cell_view_state.is_dragging() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    // Check for mouse press on an empty slot
    if mouse_button.just_released(MouseButton::Left) {
        for (interaction, slot_indicator) in &slot_query {
            if matches!(interaction, Interaction::Pressed) {
                // Check if this slot is empty
                if units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position).is_none() {
                    // Deselect any selected unit
                    if cell_view_state.selected_unit.is_some() {
                        info!("Unit deselected by clicking empty slot");
                        cell_view_state.selected_unit = None;
                    }
                    break;
                }
            }
        }
    }
}
