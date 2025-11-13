// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_ui)
            .add_systems(Update, (systems::update_ui));
    }
}
