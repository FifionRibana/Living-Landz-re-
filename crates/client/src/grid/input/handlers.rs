use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::grid::resources::SelectedHexes;

use shared::grid::GridConfig;

pub fn handle_hexagon_selection(
    mut selected_hexes: ResMut<SelectedHexes>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    grid_config: Res<GridConfig>,
    // Check if cursor is over UI - now all UI elements (buttons and panels) have Interaction
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
) -> Result {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single()?;

        // Check if cursor is over any UI element that should block lower layers
        let is_over_ui = ui_interaction_query.iter().any(|(interaction, pickable)| {
            pickable.should_block_lower
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        });

        // Don't select hexagons if cursor is over UI
        if is_over_ui {
            return Ok(());
        }
        let (camera, camera_transform) = cameras.single()?;

        if let Some(position) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
        {
            let hovered_hex = grid_config.layout.world_pos_to_hex(position);
            // ✨ Ctrl + Click = Multi-select
            if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
                selected_hexes.toggle(hovered_hex);
            } else {
                // Single select
                if selected_hexes.is_selected(hovered_hex) && selected_hexes.selection_count() == 1
                {
                    selected_hexes.remove(hovered_hex);
                } else {
                    selected_hexes.clear();
                    selected_hexes.add(hovered_hex);
                }
            }
        }
    }

    // Delete = Deselect all
    if keyboard.just_pressed(KeyCode::Delete) {
        selected_hexes.clear();
    }
    Ok(())
}
