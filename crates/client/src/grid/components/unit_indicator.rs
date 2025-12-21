use bevy::prelude::*;
use shared::grid::GridCell;

/// Composant pour marquer une entité comme indicateur d'unité sur une cellule
#[derive(Component)]
pub struct UnitIndicator {
    pub cell: GridCell,
    pub unit_count: usize,
}
