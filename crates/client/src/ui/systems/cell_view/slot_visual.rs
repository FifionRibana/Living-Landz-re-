use bevy::prelude::*;
use crate::ui::components::SlotIndicator;
use crate::ui::resources::CellViewState;

/// Update slot visual feedback based on interaction state
pub fn update_slot_visual_feedback(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<(&Interaction, &SlotIndicator, &mut BackgroundColor), Changed<Interaction>>,
) {
    if !cell_view_state.is_active {
        return;
    }

    let is_dragging = cell_view_state.is_dragging();

    for (interaction, slot_indicator, mut bg_color) in &mut slot_query {
        let is_occupied = slot_indicator.occupied_by.is_some();

        // Determine color based on state
        let color = match interaction {
            Interaction::Hovered => {
                if is_dragging {
                    if is_occupied {
                        // Cannot drop on occupied slot - red
                        Color::srgba(0.8, 0.2, 0.2, 0.6)
                    } else {
                        // Valid drop target - green
                        Color::srgba(0.2, 0.8, 0.2, 0.6)
                    }
                } else if is_occupied {
                    // Hovering occupied slot when not dragging - highlight
                    Color::srgba(0.5, 0.5, 0.9, 0.7)
                } else {
                    // Hovering empty slot - subtle highlight
                    Color::srgba(0.3, 0.3, 0.3, 0.5)
                }
            }
            Interaction::Pressed => {
                if is_occupied {
                    // Pressing occupied slot - bright highlight
                    Color::srgba(0.7, 0.7, 1.0, 0.8)
                } else {
                    Color::srgba(0.4, 0.4, 0.4, 0.6)
                }
            }
            Interaction::None => {
                // Default state - transparent
                Color::srgba(0.0, 0.0, 0.0, 0.0)
            }
        };

        *bg_color = BackgroundColor(color);
    }
}

/// Add hover effect to slot buttons on spawn
pub fn setup_slot_hover_feedback(
    mut commands: Commands,
    slot_query: Query<Entity, (With<SlotIndicator>, Without<BackgroundColor>)>,
) {
    for entity in &slot_query {
        commands.entity(entity).insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)));
    }
}
