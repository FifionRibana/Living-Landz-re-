use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::mist::materials::{GroundFogMaterial, MistMaterial};
use crate::states::AppState;

pub use super::systems;

pub struct MistPlugin;

impl Plugin for MistPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MistMaterial>::default())
            .add_plugins(Material2dPlugin::<GroundFogMaterial>::default())
            .add_systems(
                Update,
                (
                    systems::spawn_mist,
                    systems::spawn_ground_fog,
                    systems::update_mist_texture,
                    systems::update_mist_sdf,
                    systems::update_mist_camera,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}