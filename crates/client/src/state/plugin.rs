// =============================================================================
// STATE - Plugin
// =============================================================================

use bevy::prelude::*;

pub use super::resources;
pub use super::systems;

pub struct ClientStatePlugin;

impl Plugin for ClientStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::ConnectionStatus>()
            .init_resource::<resources::WorldCache>()
            .init_resource::<resources::PlayerInfo>()
            .init_resource::<resources::ActionTracker>()
            .init_resource::<resources::CurrentOrganization>()
            .init_resource::<resources::UnitsCache>()
            .init_resource::<resources::UnitsDataCache>()
            .insert_resource(resources::GameTimeConfig::default())
            .insert_resource(resources::StreamingConfig::default())
            .add_systems(Startup, (resources::setup_tree_atlas, resources::setup_building_atlas))
            .add_systems(
                Update,
                (
                    systems::unload_distant_chunks,
                    systems::request_chunks_around_camera,
                    systems::track_hovered_cell_organization,
                )
                    .chain(),
            );
    }
}
