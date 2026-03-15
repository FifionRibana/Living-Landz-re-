use bevy::prelude::*;

use crate::states::AppState;
use crate::states::GameView;

use super::components;
use super::input;
use super::resources;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::CameraSettings>()
            .init_resource::<resources::CellSceneRenderTarget>()
            .add_systems(
                Startup,
                (
                    components::setup_camera,
                    components::setup_cell_scene_camera,
                ),
            )
            .add_systems(
                Update,
                (input::camera_movement, input::camera_zoom).run_if(in_state(GameView::Map)),
            )
            .add_systems(
                Update,
                components::toggle_cell_scene_camera.run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::InGame), components::center_camera_on_lord);
    }
}
