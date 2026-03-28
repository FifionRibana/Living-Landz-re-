use bevy::prelude::*;

#[derive(Resource)]
pub struct StreamingConfig {
    pub view_radius: i32,
    pub unload_distance: i32,
    pub request_cooldown: f32,
    pub request_timeout: f32,
    pub last_request: f32
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            view_radius: 4,
            unload_distance: 6,
            request_cooldown: 0.3,
            request_timeout: 3.0,
            last_request: -999.0,
        }
    }
}