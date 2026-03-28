use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::mist::materials::MistMaterial;
use crate::states::AppState;

pub use super::systems;

pub struct MistPlugin;

impl Plugin for MistPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MistMaterial>::default())
            .add_systems(
                Update,
                (
                    systems::request_exploration_map,
                    systems::spawn_mist,
                    systems::update_mist_texture,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}