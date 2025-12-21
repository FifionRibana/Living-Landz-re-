use bevy::prelude::*;
use crate::ui::resources::CellViewState;

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
