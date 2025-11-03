// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::systems;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_debug_ui)
            .add_systems(Update, (systems::update_debug_ui));
    }
}
