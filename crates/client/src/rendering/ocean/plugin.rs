// =============================================================================
// OCEAN - Plugin
// =============================================================================

use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::ocean::materials::OceanMaterial;
use crate::states::AppState;

pub use super::systems;

pub struct OceanPlugin;

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<OceanMaterial>::default())
            .add_systems(
                Update,
                (
                    systems::request_ocean_data,
                    systems::spawn_ocean,
                    systems::update_ocean_time,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
