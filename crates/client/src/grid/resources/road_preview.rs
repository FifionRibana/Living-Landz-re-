use bevy::prelude::*;
use shared::grid::GridCell;

/// Preview d'une route en cours de construction
#[derive(Resource, Default)]
pub struct RoadPreview {
    /// Les cellules qui composent le chemin du preview
    pub path: Vec<GridCell>,
    /// Les points du monde pour afficher la spline
    pub world_points: Vec<Vec3>,
    /// Indique si le preview est valide (chemin trouvé)
    pub is_valid: bool,
}

impl RoadPreview {
    pub fn clear(&mut self) {
        self.path.clear();
        self.world_points.clear();
        self.is_valid = false;
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}
