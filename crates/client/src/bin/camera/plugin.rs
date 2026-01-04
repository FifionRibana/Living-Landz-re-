use bevy::prelude::*;

use super::camera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<camera::CameraSettings>()
            .add_systems(Startup, camera::setup_camera)
            .add_systems(Update, camera::camera_movement);
    }
}