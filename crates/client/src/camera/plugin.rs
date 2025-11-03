use bevy::prelude::*;

use super::components;
use super::input;
use super::resources;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<resources::CameraSettings>()
            .add_systems(Startup, components::setup_camera)
            .add_systems(Update, (input::camera_movement, input::camera_zoom));
    }
}