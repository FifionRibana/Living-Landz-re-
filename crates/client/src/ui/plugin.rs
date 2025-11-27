// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{
    resources::{ActionState, ChatState},
    systems,
};
use crate::state::resources;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatState::default())
            .insert_resource(ActionState::default())
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            .add_systems(
                Startup,
                (
                    resources::setup_gauge_atlas,
                    resources::setup_moon_atlas,
                    systems::setup_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    // UI visibility updates
                    systems::update_cell_details_visibility,
                    systems::update_cell_details_content,
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
                    // Action panel content updates
                    systems::update_action_panel_content,
                    systems::handle_building_button_interactions,
                    systems::handle_action_run_button,
                ),
            );
    }
}
