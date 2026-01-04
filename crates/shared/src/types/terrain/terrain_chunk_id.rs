use bevy::prelude::*;

use bincode::{Decode, Encode};

use crate::constants;

#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode,
)]
pub struct TerrainChunkId {
    pub x: i32,
    pub y: i32,
}

impl TerrainChunkId {
    pub fn from_world_pos(pos: Vec2) -> Self {
        Self {
            x: (pos.x / constants::CHUNK_SIZE.x).floor() as i32,
            y: (pos.y / constants::CHUNK_SIZE.y).floor() as i32,
        }
    }

    pub fn bounds(&self) -> (Vec2, Vec2) {
        let min = Vec2::new(
            self.x as f32 * constants::CHUNK_SIZE.x,
            self.y as f32 * constants::CHUNK_SIZE.y,
        );
        let max = min + constants::CHUNK_SIZE;
        (min, max)
    }
}
