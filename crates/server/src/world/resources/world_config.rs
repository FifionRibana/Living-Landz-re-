use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct WorldConfig {
    pub map_width: u32,
    pub map_height: u32,
    pub chunks_x: u32,
    pub chunks_y: u32,
    pub seed: u32,
}