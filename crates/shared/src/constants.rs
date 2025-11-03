use bevy::prelude::*;

pub const HEX_SIZE: f32 = 24.0; // total width = 2 * size = 2 * 24px = 48px
pub const HEX_RATIO: Vec2 = Vec2::new(1.0, 0.866); // isometric ratio (√3/2)

pub const CHUNK_SIZE: Vec2 = Vec2::new(600., 503.);