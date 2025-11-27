// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::terrain::materials::TerrainMaterial;

pub use super::systems;
use super::debug;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TerrainMaterial>::default())
            .init_resource::<debug::ChunkDebugEnabled>()
            // app.add_systems(Startup, systems::setup_default_terrain_material);
            .add_systems(
                Update,
                (
                    systems::initialize_terrain,
                    systems::spawn_terrain,
                    systems::spawn_building,
                    debug::toggle_chunk_debug,
                    debug::draw_chunk_gizmos,
                    debug::update_chunk_debug_text,
                ),
            );
    }
}
