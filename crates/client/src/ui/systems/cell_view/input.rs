use bevy::prelude::*;
use crate::ui::components::CellViewBackButton;
use crate::ui::resources::CellViewState;

/// Handle cell view exit via ESC key or back button
pub fn handle_cell_view_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cell_view_state: ResMut<CellViewState>,
    back_button_query: Query<&Interaction, (Changed<Interaction>, With<CellViewBackButton>)>,
) {
    let should_exit = keyboard.just_pressed(KeyCode::Escape)
        || back_button_query.iter().any(|i| matches!(i, Interaction::Pressed));

    if should_exit && cell_view_state.is_active {
        info!("Exiting cell view mode");
        cell_view_state.exit_view();
    }
}
