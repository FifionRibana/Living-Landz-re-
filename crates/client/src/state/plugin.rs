// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::prelude::*;

use crate::states::AppState;

pub use super::resources;
pub use super::systems;

pub struct ClientStatePlugin;

impl Plugin for ClientStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // ─── Global resources — needed before InGame ─────────────────
            .init_resource::<resources::ConnectionStatus>()
            .init_resource::<resources::PlayerInfo>()
            .insert_resource(resources::GameTimeConfig::default())
            .insert_resource(resources::StreamingConfig::default())
            .add_systems(Startup, (resources::setup_tree_atlas, resources::setup_building_atlas))
            // ─── World data — scoped to InGame ──────────────────────────
            .add_systems(OnEnter(AppState::InGame), init_world_resources)
            .add_systems(OnExit(AppState::InGame), cleanup_world_resources)
            // ─── Streaming — only InGame ────────────────────────────────
            .add_systems(
                Update,
                (
                    systems::unload_distant_chunks,
                    systems::request_chunks_around_camera,
                    // systems::track_hovered_cell_organization,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn init_world_resources(mut commands: Commands) {
    commands.insert_resource(resources::WorldCache::default());
    commands.insert_resource(resources::UnitsCache::default());
    commands.insert_resource(resources::UnitsDataCache::default());
    commands.insert_resource(resources::ActionTracker::default());
    commands.insert_resource(resources::CurrentOrganization::default());
}

fn cleanup_world_resources(mut commands: Commands) {
    commands.remove_resource::<resources::WorldCache>();
    commands.remove_resource::<resources::UnitsCache>();
    commands.remove_resource::<resources::UnitsDataCache>();
    commands.remove_resource::<resources::ActionTracker>();
    commands.remove_resource::<resources::CurrentOrganization>();
}
