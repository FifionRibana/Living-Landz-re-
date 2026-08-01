// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::debug_voronoi;
use crate::rendering::terrain::materials::{TerrainMaterial, TreeMaterial};
use crate::states::AppState;

use super::debug;
use super::ymir_cliffs;
use super::ymir_rivers;
pub use super::systems;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TerrainMaterial>::default())
            .add_plugins(Material2dPlugin::<TreeMaterial>::default())
            .init_resource::<debug::ChunkDebugEnabled>()
            .init_resource::<debug_voronoi::OrgVoronoiDebug>()
            .init_resource::<debug_voronoi::MistVoronoiDebug>()
            .init_resource::<ymir_cliffs::YmirCliffsCache>()
            .init_resource::<ymir_rivers::YmirRiversCache>();

        debug_voronoi::init_voronoi_debug_channels(app);

        app.add_systems(
                Update,
                (
                    systems::initialize_terrain,
                    systems::build_tree_atlas,
                    systems::create_terrain_global_textures,
                    systems::spawn_terrain,
                    systems::spawn_building,
                    debug::toggle_chunk_debug,
                    debug::sync_debug_uniforms,
                    debug::sync_ocean_debug_uniforms,
                    debug::draw_shore_type_gizmos,
                    debug::draw_chunk_boundary_gizmos,
                    debug::draw_chunk_gizmos,
                    debug::draw_outline_points,
                    debug::update_chunk_debug_text,
                    debug_voronoi::toggle_org_voronoi_debug,
                    debug_voronoi::poll_org_voronoi_seeds,
                    debug_voronoi::poll_territory_cells,
                    debug_voronoi::draw_org_voronoi_debug,
                    debug_voronoi::toggle_mist_voronoi_debug,
                    debug_voronoi::draw_mist_voronoi_debug,
                    debug_voronoi::draw_territory_debug_cells,
                )
                    .run_if(in_state(AppState::InGame)),
            );

        // Separate registration: the tuple above is at Bevy's 20-system limit.
        app.add_systems(
            Update,
            (
                ymir_cliffs::draw_ymir_cliffs_gizmos,
                ymir_rivers::draw_ymir_rivers_gizmos,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
