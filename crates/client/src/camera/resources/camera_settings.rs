use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraSettings {
    pub speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            speed: 500.0,
            zoom_speed: 1.,
            min_zoom: 0.5,
            max_zoom: 20.,
        }
    }
}