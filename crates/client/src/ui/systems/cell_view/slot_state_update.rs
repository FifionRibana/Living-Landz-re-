use bevy::prelude::*;
use crate::ui::components::{SlotIndicator, SlotState};
use crate::ui::resources::CellViewState;

/// System to update slot visual states based on changes
/// This handles sprite changes and opacity updates when slot state changes
pub fn update_slot_visual_state(
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&SlotIndicator, &Interaction, &mut ImageNode), Changed<SlotIndicator>>,
) {
    for (slot_indicator, interaction, mut image_node) in &mut slot_query {
        // Get the appropriate sprite for the current state
        let sprite_path = slot_indicator.state.get_sprite_path(slot_indicator.is_occupied());

        // Load and update the image
        image_node.image = asset_server.load(&sprite_path);

        // Update opacity based on state, BUT respect current interaction state
        // Don't overwrite hover/pressed colors set by update_slot_visual_feedback
        let opacity = match interaction {
            Interaction::Hovered => slot_indicator.state.get_hover_opacity(slot_indicator.is_occupied()),
            Interaction::Pressed => slot_indicator.state.get_opacity(slot_indicator.is_occupied()).min(1.0),
            Interaction::None => slot_indicator.state.get_opacity(slot_indicator.is_occupied()),
        };
        image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    }
}

/// System to handle slot selection on click
pub fn handle_slot_click(
    mut cell_view_state: ResMut<CellViewState>,
    mut slot_query: Query<(&Interaction, &mut SlotIndicator), Changed<Interaction>>,
) {
    if !cell_view_state.is_active {
        return;
    }

    for (interaction, mut slot_indicator) in &mut slot_query {
        if *interaction == Interaction::Pressed {
            // Toggle selection
            match slot_indicator.state {
                SlotState::Normal => {
                    // Deselect any previously selected slot first
                    slot_indicator.state = SlotState::Selected;
                    cell_view_state.select_slot(slot_indicator.position);
                    info!("Slot selected: {:?}", slot_indicator.position);
                }
                SlotState::Selected => {
                    slot_indicator.state = SlotState::Normal;
                    cell_view_state.deselect_slot();
                    info!("Slot deselected: {:?}", slot_indicator.position);
                }
                _ => {
                    // Can't select disabled/invalid slots
                    warn!("Cannot select slot in state: {:?}", slot_indicator.state);
                }
            }
        }
    }
}

/// System to ensure only one slot is selected at a time
pub fn enforce_single_selection(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<&mut SlotIndicator>,
) {
    if !cell_view_state.is_changed() {
        return;
    }

    let selected_slot = cell_view_state.selected_slot;

    for mut slot_indicator in &mut slot_query {
        // If this slot is not the selected one and it's currently marked as selected, deselect it
        if Some(slot_indicator.position) != selected_slot && slot_indicator.state == SlotState::Selected {
            slot_indicator.state = SlotState::Normal;
        }
        // If this slot IS the selected one and it's not marked as selected, select it
        else if Some(slot_indicator.position) == selected_slot && slot_indicator.state != SlotState::Selected {
            slot_indicator.state = SlotState::Selected;
        }
    }
}

/// System to update slot occupied state when units are assigned/removed
pub fn update_slot_occupation(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<&mut SlotIndicator>,
) {
    if !cell_view_state.is_active {
        return;
    }

    // This will be triggered by UnitsCache changes
    // The occupation is already handled by the auto_assign and drag_drop systems
    // This system just ensures visual consistency

    for mut slot_indicator in &mut slot_query {
        // If occupied, ensure we use the _empty variant
        if slot_indicator.is_occupied() && slot_indicator.state == SlotState::Normal {
            // Force a visual update by touching the component
            slot_indicator.set_changed();
        }
    }
}
