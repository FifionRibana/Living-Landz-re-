use bevy::prelude::*;
use shared::BiomeChunkId;

#[derive(Component)]
pub struct Biome {
    pub name: String,
    pub id: BiomeChunkId,
}