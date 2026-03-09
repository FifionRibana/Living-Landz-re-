use bevy::prelude::*;
use shared::grid::GridCell;
use shared::TerrainChunkId;

/// Actions disponibles dans le menu contextuel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    Move,
    Found,
    Build(shared::BuildingTypeEnum),
    // Futures actions :
    // Harvest,
}

impl ContextMenuAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Move => "Déplacer",
            Self::Found => "Fonder un hameau",
            Self::Build(bt) => bt.to_name_lowercase(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Move => "➤",
            Self::Found => "⛫",
            Self::Build(_) => "🔨",
        }
    }
}

/// État du menu contextuel
#[derive(Resource, Default, Debug)]
pub struct ContextMenuState {
    /// Visible ?
    pub open: bool,
    /// Position à l'écran (viewport coords) pour le spawn
    pub screen_position: Vec2,
    /// Cellule hex cible (là où le joueur a cliqué droit)
    pub target_cell: Option<GridCell>,
    /// Chunk cible
    pub target_chunk: Option<TerrainChunkId>,
    /// Actions disponibles (calculées au moment de l'ouverture)
    pub available_actions: Vec<ContextMenuAction>,
}

impl ContextMenuState {
    pub fn open_at(
        &mut self,
        screen_pos: Vec2,
        cell: GridCell,
        chunk: TerrainChunkId,
        actions: Vec<ContextMenuAction>,
    ) {
        self.open = true;
        self.screen_position = screen_pos;
        self.target_cell = Some(cell);
        self.target_chunk = Some(chunk);
        self.available_actions = actions;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.target_cell = None;
        self.target_chunk = None;
        self.available_actions.clear();
    }
}