use bevy::prelude::*;
use shared::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId, grid::GridCell};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TrackedAction {
    pub action_id: u64,
    pub player_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub action_type: ActionTypeEnum,
    pub status: ActionStatusEnum,
    pub completion_time: u64,
}

/// Resource pour suivre les actions en cours côté client
#[derive(Resource, Default)]
pub struct ActionTracker {
    /// Toutes les actions par ID
    actions: HashMap<u64, TrackedAction>,

    /// Index des actions par cellule pour un accès rapide
    actions_by_cell: HashMap<(TerrainChunkId, GridCell), u64>,
}

impl ActionTracker {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            actions_by_cell: HashMap::new(),
        }
    }

    /// Ajoute ou met à jour une action
    pub fn update_action(&mut self, action: TrackedAction) {
        let cell_key = (action.chunk_id.clone(), action.cell.clone());

        // Si l'action est complétée, on peut la retirer après un délai
        // Pour l'instant on la garde pour afficher la coche verte

        self.actions_by_cell.insert(cell_key, action.action_id);
        self.actions.insert(action.action_id, action);
    }

    /// Récupère une action par son ID
    pub fn get_action(&self, action_id: u64) -> Option<&TrackedAction> {
        self.actions.get(&action_id)
    }

    /// Récupère l'action en cours sur une cellule
    pub fn get_action_on_cell(&self, chunk_id: &TerrainChunkId, cell: &GridCell) -> Option<&TrackedAction> {
        let cell_key = (chunk_id.clone(), cell.clone());
        self.actions_by_cell.get(&cell_key)
            .and_then(|action_id| self.actions.get(action_id))
    }

    /// Supprime une action
    pub fn remove_action(&mut self, action_id: u64) {
        if let Some(action) = self.actions.remove(&action_id) {
            let cell_key = (action.chunk_id, action.cell);
            self.actions_by_cell.remove(&cell_key);
        }
    }

    /// Supprime les actions complétées plus anciennes qu'un certain temps
    pub fn cleanup_completed_actions(&mut self, current_time: u64, retention_seconds: u64) {
        let mut to_remove = Vec::new();

        for (action_id, action) in self.actions.iter() {
            if action.status == ActionStatusEnum::Completed {
                // Garder les actions complétées pendant retention_seconds secondes
                if current_time > action.completion_time + retention_seconds {
                    to_remove.push(*action_id);
                }
            }
        }

        for action_id in to_remove {
            self.remove_action(action_id);
        }
    }

    /// Récupère toutes les actions
    pub fn get_all_actions(&self) -> impl Iterator<Item = &TrackedAction> {
        self.actions.values()
    }

    /// Récupère toutes les actions d'un chunk
    pub fn get_chunk_actions(&self, chunk_id: &TerrainChunkId) -> Vec<&TrackedAction> {
        self.actions.values()
            .filter(|action| &action.chunk_id == chunk_id)
            .collect()
    }
}
