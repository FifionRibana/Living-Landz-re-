use bevy::prelude::*;

use crate::grid::systems;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (systems::setup_grid_config, systems::setup_meshes).chain(),
        );        // Mandatory for many other systems
    }
}
