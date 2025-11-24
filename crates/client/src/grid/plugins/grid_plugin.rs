use bevy::prelude::*;

use crate::grid::systems;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (systems::setup_grid_config, systems::setup_meshes).chain(),
        )
        .add_systems(
            Update,
            (
                systems::update_action_indicators,
                systems::animate_in_progress_indicators,
                systems::cleanup_completed_indicators,
            ),
        );
    }
}
