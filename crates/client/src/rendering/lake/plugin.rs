use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::rendering::lake::materials::LakeMaterial;
use crate::states::AppState;

pub use super::systems;

pub struct LakePlugin;

impl Plugin for LakePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<LakeMaterial>::default())
            .add_systems(
                Update,
                (
                    systems::request_lake_data,
                    systems::create_lake_textures,
                    systems::spawn_lake,
                    systems::update_lake_time,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}