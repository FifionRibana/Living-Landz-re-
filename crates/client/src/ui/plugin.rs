// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use crate::state::resources;
use super::{resources::ChatState, systems};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatState::default())
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            .add_systems(
                Startup,
                (resources::setup_gauge_atlas, resources::setup_moon_atlas, systems::setup_ui).chain(),
            )
            .add_systems(
                Update,
                (
                    systems::update_ui,
                    systems::update_clock,
                    systems::update_moon_phase_image,
                    systems::update_player_info,
                    systems::handle_menu_button_interactions,
                    systems::handle_action_button_interactions,
                    systems::handle_chat_send_button,
                    systems::handle_chat_toggle_button,
                    systems::handle_chat_icon_button,
                    systems::update_chat_visibility,
                    systems::update_chat_notification_badge,
                    // systems::update_actions_panel_layout,
                ),
            );
    }
}
