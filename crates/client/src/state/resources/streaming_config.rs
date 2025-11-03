use bevy::prelude::*;

#[derive(Resource)]
pub struct StreamingConfig {
    pub view_radius: i32,
    pub unload_distance: i32,
    pub request_cooldown: f32,
    pub last_request: f32
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            view_radius: 2,
            unload_distance: 3,
            request_cooldown: 0.5,
            last_request: -999.0
        }
    }
}