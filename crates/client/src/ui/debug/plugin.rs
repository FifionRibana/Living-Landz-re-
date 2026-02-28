// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{input_handler, systems};
use crate::states::AppState;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_debug_ui)
            .add_systems(
                Update,
                (systems::update_debug_ui, input_handler::handle_debug_input)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
