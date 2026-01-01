use crate::ui::components::{CellViewBackButton, CellViewContainer};
use crate::ui::resources::{CellState, PanelEnum, UIState};
use bevy::prelude::*;

/// Handle cell view exit via ESC key or back button
pub fn handle_cell_view_exit(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cell_state: ResMut<CellState>,
    ui_state: Res<UIState>,
    back_button_query: Query<&Interaction, (Changed<Interaction>, With<CellViewBackButton>)>,
    container_query: Query<Entity, With<CellViewContainer>>,
    children_query: Query<&Children>,
) {
    let should_exit = keyboard.just_pressed(KeyCode::Escape)
        || back_button_query
            .iter()
            .any(|i| matches!(i, Interaction::Pressed));

    if should_exit && ui_state.panel_state == PanelEnum::CellView {
        info!("Exiting cell view mode");
        cell_state.exit_view();

        // Clear existing content in container
        for container_entity in &container_query {
            if let Ok(children) = children_query.get(container_entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}
