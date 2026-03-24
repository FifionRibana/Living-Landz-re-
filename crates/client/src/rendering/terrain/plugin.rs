// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::terrain::materials::{TerrainMaterial, TreeMaterial};
use crate::states::AppState;

use super::debug;
pub use super::systems;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TerrainMaterial>::default())
            .add_plugins(Material2dPlugin::<TreeMaterial>::default())
            .init_resource::<debug::ChunkDebugEnabled>()
            .add_systems(
                Update,
                (
                    systems::initialize_terrain,
                    systems::build_tree_atlas,
                    systems::request_terrain_global_data,
                    systems::create_terrain_global_textures,
                    systems::spawn_terrain,
                    systems::rebuild_tree_mesh,
                    systems::spawn_building,
                    debug::toggle_chunk_debug,
                    debug::draw_chunk_gizmos,
                    debug::draw_outline_points,
                    debug::update_chunk_debug_text,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
