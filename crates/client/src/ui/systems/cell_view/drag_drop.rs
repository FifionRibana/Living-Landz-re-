use bevy::prelude::*;
use crate::ui::components::SlotIndicator;
use crate::ui::resources::CellViewState;
use crate::state::resources::UnitsCache;
use crate::networking::client::NetworkClient;
use shared::protocol::ClientMessage;
use shared::SlotPosition;

/// Marker component for the visual drag indicator
#[derive(Component)]
pub struct DragIndicator;

/// Handle the start of a drag operation when user clicks on an occupied slot
pub fn handle_slot_drag_start(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cell_view_state: ResMut<CellViewState>,
    units_cache: Res<UnitsCache>,
    slot_query: Query<(&Interaction, &SlotIndicator), Changed<Interaction>>,
) {
    // Only process in cell view mode and when not already dragging
    if !cell_view_state.is_active || cell_view_state.is_dragging() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    // Check for mouse press on an occupied slot
    if mouse_button.just_pressed(MouseButton::Left) {
        for (interaction, slot_indicator) in &slot_query {
            if matches!(interaction, Interaction::Pressed) {
                // Check if this slot has a unit
                if let Some(unit_id) = units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position) {
                    info!(
                        "Starting drag: unit {} from slot {:?}",
                        unit_id, slot_indicator.position
                    );
                    cell_view_state.start_dragging(unit_id, slot_indicator.position);
                    break;
                }
            }
        }
    }
}

/// Update visual feedback during drag operation
pub fn update_drag_visual(
    mut commands: Commands,
    cell_view_state: Res<CellViewState>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    existing_indicator: Query<Entity, With<DragIndicator>>,
) {
    // Remove existing indicator if not dragging
    if !cell_view_state.is_dragging() {
        for entity in &existing_indicator {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(_dragging_unit) = &cell_view_state.dragging_unit else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Remove old indicator
    for entity in &existing_indicator {
        commands.entity(entity).despawn();
    }

    // Spawn new indicator at cursor position
    commands.spawn((
        ImageNode {
            image: asset_server.load("ui/icons/unit_placeholder.png"),
            ..default()
        },
        Node {
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            position_type: PositionType::Absolute,
            left: Val::Px(cursor_pos.x - 24.0),
            top: Val::Px(cursor_pos.y - 24.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        DragIndicator,
    ));
}

/// Handle dropping a unit on a slot
pub fn handle_slot_drop(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cell_view_state: ResMut<CellViewState>,
    units_cache: Res<UnitsCache>,
    slot_query: Query<(&Interaction, &SlotIndicator)>,
    mut network_client: ResMut<NetworkClient>,
) {
    // Only process when dragging and mouse is released
    if !cell_view_state.is_dragging() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    let Some(dragging_unit) = &cell_view_state.dragging_unit else {
        return;
    };

    if mouse_button.just_released(MouseButton::Left) {
        // Find which slot the mouse is over
        let mut dropped_on_slot: Option<SlotPosition> = None;

        for (interaction, slot_indicator) in &slot_query {
            if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                dropped_on_slot = Some(slot_indicator.position);
                break;
            }
        }

        match dropped_on_slot {
            Some(target_slot) => {
                // Check if target slot is different from source
                if target_slot != dragging_unit.from_slot {
                    // Check if target slot is empty
                    if units_cache.is_slot_occupied(&viewed_cell, &target_slot) {
                        warn!(
                            "Cannot drop unit {} on occupied slot {:?}",
                            dragging_unit.unit_id, target_slot
                        );
                    } else {
                        // Send move request to server
                        info!(
                            "Moving unit {} from {:?} to {:?}",
                            dragging_unit.unit_id, dragging_unit.from_slot, target_slot
                        );

                        let message = ClientMessage::MoveUnitToSlot {
                            unit_id: dragging_unit.unit_id,
                            cell: viewed_cell,
                            from_slot: dragging_unit.from_slot,
                            to_slot: target_slot,
                        };

                        network_client.send_message(message);
                    }
                } else {
                    info!("Dropped on same slot, treating as unit selection");
                    // This is a click on the same slot, treat it as a selection
                    cell_view_state.selected_unit = Some(dragging_unit.unit_id);
                }
            }
            None => {
                info!("Dropped outside slot area, canceling drag");
            }
        }

        // Stop dragging
        cell_view_state.stop_dragging();
    }
}

/// Cancel drag if ESC is pressed
pub fn cancel_drag_on_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cell_view_state: ResMut<CellViewState>,
) {
    if cell_view_state.is_dragging() && keyboard.just_pressed(KeyCode::Escape) {
        info!("Drag canceled by ESC key");
        cell_view_state.stop_dragging();
    }
}
