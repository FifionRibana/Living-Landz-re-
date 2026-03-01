use bevy::prelude::*;
use crate::ui::components::{SlotIndicator, SlotUnitSprite};
use crate::ui::resources::{CellViewState, UnitSelectionState};
use crate::ui::components::SlotBorderOverlay;
use crate::state::resources::UnitsCache;

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

/// Tint the border overlay of slots whose unit is selected in UnitSelectionState.
/// Runs every frame when the selection changes to apply/remove green tint.
pub fn update_unit_selection_slot_visuals(
    cell_view_state: Res<CellViewState>,
    unit_selection: Res<UnitSelectionState>,
    units_cache: Res<UnitsCache>,
    mut border_query: Query<(&SlotBorderOverlay, &mut ImageNode, &Interaction)>,
) {
    if !cell_view_state.is_active {
        return;
    }
    // Only reprocess when selection or cache changes
    if !unit_selection.is_changed() && !units_cache.is_changed() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    for (border, mut image_node, interaction) in &mut border_query {
        let unit_at_slot = units_cache.get_unit_at_slot(&viewed_cell, &border.slot_position);
        let is_selected = unit_at_slot
            .map(|uid| unit_selection.is_selected(uid))
            .unwrap_or(false);

        // Don't override hover/pressed state set by update_slot_overlay_visual_feedback
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            continue;
        }

        if is_selected {
            // Green tint for selected units
            image_node.color = Color::srgba(0.4, 1.0, 0.4, 0.9);
        } else {
            // Normal opacity
            image_node.color = Color::srgba(1.0, 1.0, 1.0, 0.75);
        }
    }
}

/// Tint unit portrait sprites with a light blue overlay when selected.
/// This gives immediate visual feedback on which units are selected.
pub fn update_unit_selection_portrait_tint(
    cell_view_state: Res<CellViewState>,
    unit_selection: Res<UnitSelectionState>,
    mut sprite_query: Query<(&SlotUnitSprite, &mut ImageNode)>,
) {
    if !cell_view_state.is_active {
        return;
    }
    if !unit_selection.is_changed() {
        return;
    }

    for (sprite, mut image_node) in &mut sprite_query {
        if unit_selection.is_selected(sprite.unit_id) {
            // Light blue tint on portrait
            image_node.color = Color::srgba(0.7, 0.85, 1.0, 1.0);
        } else {
            // Normal — no tint
            image_node.color = Color::WHITE;
        }
    }
}
