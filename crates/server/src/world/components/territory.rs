use hexx::*;

/// Une arête de bordure identifiée par son hexagone et sa direction
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BorderEdge {
    pub hex: Hex,
    pub dir: usize,
}

/// Information sur une transition diagonale
pub struct DiagonalTransitionInfo {
    pub diag_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HexRelation {
    /// Voisins directs (partagent une arête)
    Adjacent(usize),
    /// En diagonale (partagent un sommet)
    Diagonal {
        diag_index: usize,
        junction_a: Hex, // via dir[(diag_index+5)%6]
        junction_b: Hex, // via dir[diag_index]
    },
    /// Non connectés ou même hexagone
    Other,
}

pub fn hex_relation(a: Hex, b: Hex) -> HexRelation {
    let delta = b - a;

    // Voisin direct ?
    for (i, &neighbor_delta) in Hex::NEIGHBORS_COORDS.iter().enumerate() {
        if delta == neighbor_delta {
            return HexRelation::Adjacent(i);
        }
    }

    // Diagonale ?
    for (i, &diag_delta) in Hex::DIAGONAL_COORDS.iter().enumerate() {
        if delta == diag_delta {
            let dir_a = (i + 5) % 6;
            let dir_b = i;
            return HexRelation::Diagonal {
                diag_index: i,
                junction_a: a + Hex::NEIGHBORS_COORDS[dir_a],
                junction_b: a + Hex::NEIGHBORS_COORDS[dir_b],
            };
        }
    }

    HexRelation::Other
}
