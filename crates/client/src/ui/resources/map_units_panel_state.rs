use bevy::prelude::*;
use shared::grid::GridCell;

/// Contrôle la sidebar d'unités en vue Map
#[derive(Resource)]
pub struct MapUnitsPanelState {
    /// Mode d'affichage du panel
    pub mode: MapUnitsPanelMode,
    /// Rayon de recherche en hexes
    pub scan_radius: u32,
}

impl Default for MapUnitsPanelState {
    fn default() -> Self {
        Self {
            mode: MapUnitsPanelMode::OnSelection,
            scan_radius: 2,
        }
    }
}

/// Mode d'affichage du panel d'unités sur la map
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapUnitsPanelMode {
    /// Toujours visible — montre les unités autour du centre de l'écran
    AlwaysVisible,
    /// Visible seulement quand un hex est sélectionné
    #[default]
    OnSelection,
}

/// Unités visibles dans le rayon du panel, recalculées quand la sélection change
#[derive(Resource, Default, Debug)]
pub struct VisibleUnitsInRange {
    /// (unit_id, cell) des unités trouvées
    pub units: Vec<(u64, GridCell)>,
}
