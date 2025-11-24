use bevy::prelude::*;
use shared::{ActionStatusEnum, TerrainChunkId, grid::GridCell};

/// Composant pour marquer une entité comme indicateur d'action sur une cellule
#[derive(Component)]
pub struct ActionIndicator {
    pub action_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub status: ActionStatusEnum,
}

/// Marker pour les différents types d'indicateurs
#[derive(Component)]
pub struct PendingIndicator;

#[derive(Component)]
pub struct InProgressIndicator;

#[derive(Component)]
pub struct CompletedIndicator;
