use bevy::prelude::*;
use crate::ui::components::SlotIndicator;
use crate::ui::resources::CellViewState;
use crate::ui::systems::SlotBorderOverlay;

/// Update slot visual feedback based on interaction state
pub fn update_slot_visual_feedback(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<(&Interaction, &SlotIndicator, &mut ImageNode), Changed<Interaction>>,
) {
    if !cell_view_state.is_active {
        return;
    }

    let is_dragging = cell_view_state.is_dragging();

    for (interaction, slot_indicator, mut image_node) in &mut slot_query {
        let is_occupied = slot_indicator.occupied_by.is_some();

        // For drag & drop, use overlay colors
        // For normal hover, adjust opacity
        match interaction {
            Interaction::Hovered => {
                if is_dragging {
                    // During drag & drop, show color feedback
                    if is_occupied {
                        // Cannot drop on occupied slot - red tint
                        image_node.color = Color::srgba(1.0, 0.5, 0.5, 0.6);
                    } else {
                        // Valid drop target - green tint
                        image_node.color = Color::srgba(0.5, 1.0, 0.5, 0.6);
                    }
                } else {
                    // Normal hover - increase opacity to 40%
                    let hover_opacity = slot_indicator.state.get_hover_opacity(slot_indicator.is_occupied());
                    image_node.color = Color::srgba(1.0, 1.0, 1.0, hover_opacity);
                }
            }
            Interaction::Pressed => {
                // Slight brightness increase on press
                let opacity = slot_indicator.state.get_opacity(slot_indicator.is_occupied());
                image_node.color = Color::srgba(1.2, 1.2, 1.2, opacity.min(1.0));
            }
            Interaction::None => {
                // Return to normal opacity
                let opacity = slot_indicator.state.get_opacity(slot_indicator.is_occupied());
                image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
            }
        };
    }
}

/// Update slot visual feedback based on interaction state
pub fn update_slot_overlay_visual_feedback(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<(&Interaction, &SlotBorderOverlay, &mut ImageNode), Changed<Interaction>>,
) {
    if !cell_view_state.is_active {
        return;
    }

    let is_dragging = cell_view_state.is_dragging();

    for (interaction, _slot_border, mut image_node) in &mut slot_query {

        // For drag & drop, use overlay colors
        // For normal hover, adjust opacity
        match interaction {
            Interaction::Hovered => {
                if is_dragging {
                    // During drag & drop, show color feedback
                    image_node.color = Color::srgba(1.0, 0.5, 0.5, 0.6);
                } else {
                    // Normal hover - increase opacity to 40%
                    let hover_opacity = 0.9;
                    image_node.color = Color::srgba(1.0, 1.0, 1.0, hover_opacity);
                }
            }
            Interaction::Pressed => {
                // Slight brightness increase on press
                let opacity = 1.0;
                image_node.color = Color::srgba(1.2, 1.2, 1.2, opacity);
            }
            Interaction::None => {
                // Return to normal opacity
                let opacity = 0.75;
                image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
            }
        };
    }
}
