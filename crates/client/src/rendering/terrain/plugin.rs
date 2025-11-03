// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::prelude::*;

pub use super::systems;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (systems::initialize_terrain, systems::spawn_terrain),
        );
    }
}
