// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{
    resources::{ActionState, CellViewState, ChatState},
    systems,
};
use crate::state::resources;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatState::default())
            .insert_resource(ActionState::default())
            .insert_resource(CellViewState::default())
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            .add_systems(
                Startup,
                (
                    resources::setup_gauge_atlas,
                    resources::setup_moon_atlas,
                    systems::setup_ui,
                    systems::setup_cell_view_ui,
                    systems::setup_unit_details_panel,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    // Cell view input
                    systems::handle_cell_view_exit,
                    // UI visibility updates
                    systems::update_cell_view_visibility,
                    systems::update_cell_view_content,
                    // Auto-assign units without slots (must run after content is created)
                    systems::auto_assign_unslotted_units.after(systems::update_cell_view_content),
                    // Cell view unit sprites (must run after content is created and auto-assignment)
                    systems::update_slot_occupancy.after(systems::auto_assign_unslotted_units),
                    systems::update_unit_sprites.after(systems::auto_assign_unslotted_units),
                    // Cell view slot visual feedback
                    systems::setup_slot_hover_feedback,
                    systems::update_slot_visual_feedback,
                    systems::update_cell_details_visibility,
                    systems::update_cell_details_content,
                    systems::update_organization_info,
                        // .before(systems::update_chat_notification_badge)
                        // .before(systems::update_player_info),
                    systems::update_action_bar_visibility,
                    systems::update_action_panel_visibility,
                    systems::update_chat_visibility,
                    systems::update_chat_notification_badge,
                    // Dynamic updates
                    systems::update_clock,
                    systems::update_moon_phase_image,
                    systems::update_player_info,
                    systems::update_cell_action_display,
                    systems::hide_action_panel_during_action,
                ),
            )
            .add_systems(
                Update,
                (
                    // Button interactions
                    systems::handle_menu_button_interactions,
                    systems::handle_action_category_button_interactions,
                    systems::update_action_category_button_appearance,
                    systems::handle_action_tab_button_interactions,
                    systems::handle_action_button_interactions,
                    systems::handle_chat_send_button,
                    systems::handle_chat_toggle_button,
                    systems::handle_chat_icon_button,
                    // Unit selection and details panel
                    systems::handle_unit_deselect,
                    systems::update_panel_visibility,
                    systems::update_panel_content,
                    systems::handle_close_button,
                    // Cell view drag & drop
                    systems::handle_slot_drag_start,
                    systems::update_drag_visual,
                    systems::handle_slot_drop,
                    systems::cancel_drag_on_escape,
                    // Action panel content updates
                    systems::update_action_panel_content,
                    systems::handle_building_button_interactions,
                    systems::handle_action_run_button,
                ),
            );
    }
}
