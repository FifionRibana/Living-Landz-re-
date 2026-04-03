use bevy::prelude::*;
use shared::TerrainChunkId;

#[derive(Component)]
pub struct TreeChunkMesh {
    pub chunk_id: TerrainChunkId,
}
