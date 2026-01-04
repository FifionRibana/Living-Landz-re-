use std::collections::HashSet;

use hexx::*;

/// Milieu de l'arête dans la direction donnée
pub fn edge_midpoint(layout: &HexLayout, hex: Hex, dir_index: usize) -> Vec2 {
    let center = layout.hex_to_world_pos(hex);
    let neighbor = hex + Hex::NEIGHBORS_COORDS[dir_index];
    let neighbor_center = layout.hex_to_world_pos(neighbor);
    (center + neighbor_center) * 0.5
}

pub fn next_exterior_edge(hex: Hex, dir: usize, territory: &HashSet<Hex>) -> (Hex, usize) {
    // Le voisin dans la direction dir (extérieur par définition)
    // On tourne en sens horaire pour trouver la prochaine arête extérieure

    // D'abord, vérifier si le voisin "au coin" (sens horaire) est dans le territoire
    let next_dir_cw = (dir + 5) % 6; // sens horaire = index décroissant
    let corner_hex = hex + Hex::NEIGHBORS_COORDS[next_dir_cw];

    if territory.contains(&corner_hex) {
        // On passe sur cet hexagone et on cherche son arête extérieure
        // La direction "entrante" sur corner_hex est l'opposée de next_dir_cw
        let opposite = (next_dir_cw + 3) % 6;

        // Chercher la prochaine arête extérieure en tournant horaire depuis opposite
        for offset in 1..6 {
            let test_dir = (opposite + 6 - offset) % 6; // sens horaire
            if !territory.contains(&(corner_hex + Hex::NEIGHBORS_COORDS[test_dir])) {
                return (corner_hex, test_dir);
            }
        }
    }

    // Sinon, rester sur le même hex et prendre l'arête suivante (sens horaire)
    for offset in 1..6 {
        let test_dir = (dir + 6 - offset) % 6;
        if !territory.contains(&(hex + Hex::NEIGHBORS_COORDS[test_dir])) {
            return (hex, test_dir);
        }
    }

    // Ne devrait pas arriver
    (hex, dir)
}
