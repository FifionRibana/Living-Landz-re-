use bevy::prelude::*;
use shared::TerrainChunkId;

#[derive(Component)]
pub struct Terrain {
    pub name: String,
    pub id: TerrainChunkId,
}