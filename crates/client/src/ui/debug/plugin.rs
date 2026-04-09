// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{input_handler, layer_panel, layer_state, layer_toggles, overlay_panel, overlay_state, systems};
use crate::states::AppState;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<overlay_state::DebugOverlayState>()
            .init_resource::<layer_state::DebugLayerVisibility>()
            .add_systems(
                Startup,
                (
                    systems::setup_debug_ui,
                    overlay_panel::setup_debug_panel,
                    layer_panel::setup_layer_panel,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::update_debug_ui,
                    input_handler::handle_debug_input,
                    overlay_panel::toggle_debug_panel,
                    overlay_panel::handle_debug_panel_clicks,
                    overlay_panel::update_debug_button_visuals,
                    layer_panel::toggle_layer_panel,
                    layer_panel::handle_layer_panel_clicks,
                    layer_panel::update_layer_button_visuals,
                    layer_toggles::apply_layer_visibility,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
