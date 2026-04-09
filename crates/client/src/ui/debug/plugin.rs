// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{input_handler, overlay_panel, overlay_state, systems};
use crate::states::AppState;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<overlay_state::DebugOverlayState>()
            .add_systems(
                Startup,
                (systems::setup_debug_ui, overlay_panel::setup_debug_panel),
            )
            .add_systems(
                Update,
                (
                    systems::update_debug_ui,
                    input_handler::handle_debug_input,
                    overlay_panel::toggle_debug_panel,
                    overlay_panel::handle_debug_panel_clicks,
                    overlay_panel::update_debug_button_visuals,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
