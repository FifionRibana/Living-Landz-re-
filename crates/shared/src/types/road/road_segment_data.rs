use bincode::{Decode, Encode};
use crate::grid::GridCell;

/// Identifiant unique d'un segment de route
pub type RoadSegmentId = i64;

/// Données d'un segment de route pour la synchronisation réseau
#[derive(Debug, Clone, Encode, Decode)]
pub struct RoadSegmentData {
    /// ID unique du segment
    pub id: RoadSegmentId,

    /// Cellule hexagonale de départ
    pub start_cell: GridCell,

    /// Cellule hexagonale d'arrivée
    pub end_cell: GridCell,

    /// Points de la polyline (coordonnées monde en pixels)
    /// Le premier point est sur start_cell, le dernier sur end_cell
    /// Les points intermédiaires permettent des courbes naturelles
    pub points: Vec<[f32; 2]>,

    /// Importance du segment (0-3)
    /// 0 = sentier, 1 = chemin, 2 = route, 3 = route principale
    pub importance: u8,
}

impl RoadSegmentData {
    /// Crée un segment droit entre deux cellules
    pub fn straight(
        id: RoadSegmentId,
        start_cell: GridCell,
        end_cell: GridCell,
        start_pos: [f32; 2],
        end_pos: [f32; 2],
        importance: u8,
    ) -> Self {
        Self {
            id,
            start_cell,
            end_cell,
            points: vec![start_pos, end_pos],
            importance,
        }
    }

    /// Crée un segment avec points intermédiaires (courbe)
    pub fn curved(
        id: RoadSegmentId,
        start_cell: GridCell,
        end_cell: GridCell,
        points: Vec<[f32; 2]>,
        importance: u8,
    ) -> Self {
        assert!(points.len() >= 2, "Un segment doit avoir au moins 2 points");
        Self {
            id,
            start_cell,
            end_cell,
            points,
            importance,
        }
    }
}
