// =============================================================================
// NETWORKING - Plugin
// =============================================================================

use bevy::prelude::*;

/// Networking plugin — all communication now goes through lightyear (LightyearClientPlugin).
/// This plugin is kept as a placeholder for any future non-lightyear networking needs.
pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, _app: &mut App) {
        // All networking systems are now in LightyearClientPlugin.
        // This plugin is intentionally empty after the tungstenite removal (#138).
    }
}
