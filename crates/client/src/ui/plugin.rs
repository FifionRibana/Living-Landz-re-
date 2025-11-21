// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use crate::state::resources;
use super::systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (resources::setup_gauge_atlas, systems::setup_ui).chain(),
        )
        .add_systems(
            Update,
            (
                systems::update_ui,
                systems::update_clock,
                systems::update_player_info,
                systems::handle_menu_button_interactions,
            ),
        );
    }
}
