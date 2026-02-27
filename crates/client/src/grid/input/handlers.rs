use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::grid::resources::SelectedHexes;
use crate::state::resources::WorldCache;
use crate::states::GameView;
use crate::ui::resources::CellState;
use crate::camera::MainCamera;

use hexx::Hex;
use shared::grid::{GridCell, GridConfig};

pub fn handle_hexagon_selection(
    mut selected_hexes: ResMut<SelectedHexes>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    grid_config: Res<GridConfig>,
    game_view: Option<Res<State<GameView>>>,
    // Check if cursor is over UI - now all UI elements (buttons and panels) have Interaction
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
) -> Result {
    let Some(gv) = game_view else { return Ok(()) };
    if *gv.get() != GameView::Map {
        return Ok(());
    }

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

/// Detect double-click on hexagons to enter cell view mode
pub fn handle_cell_view_entry(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cell_state: ResMut<CellState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_config: Res<GridConfig>,
    time: Res<Time>,
    // Check if cursor is over UI
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
    mut last_click: Local<Option<(Hex, f32)>>,
    mut next_view: ResMut<NextState<GameView>>,
    game_view: Option<Res<State<GameView>>>,
    world_cache: Res<WorldCache>,
) {
    let Some(gv) = game_view else { return };
    // Only process clicks when NOT in cell view mode
    if *gv.get() == GameView::Cell {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };

        // Check if cursor is over any UI element that should block lower layers
        let is_over_ui = ui_interaction_query.iter().any(|(interaction, pickable)| {
            pickable.should_block_lower
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        });

        // Don't process clicks on UI
        if is_over_ui {
            return;
        }

        let Ok((camera, camera_transform)) = cameras.single() else {
            return;
        };

        if let Some(position) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
        {
            let clicked_hex = grid_config.layout.world_pos_to_hex(position);
            let current_time = time.elapsed_secs();

            // Check for double-click
            if let Some((last_hex, last_time)) = *last_click
                && last_hex == clicked_hex
                && (current_time - last_time) < 0.3
            {
                // Double-click detected!
                let cell = GridCell::from_hex(&clicked_hex);
                info!("Double-click detected on cell: q={}, r={}", cell.q, cell.r);

                cell_state.enter_view(
                    world_cache.get_cell(&cell).cloned(),
                    world_cache.get_building(&cell).cloned(),
                );
                next_view.set(GameView::Cell);
                *last_click = None;
                return;
            }

            // Store this click for potential double-click
            *last_click = Some((clicked_hex, current_time));
        }
    }
}
