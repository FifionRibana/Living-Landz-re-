use std::collections::HashSet;

use hexx::*;

use crate::world::components::{BorderEdge, DiagonalTransitionCurvature, DiagonalTransitionInfo, HexRelation, hex_relation};
use crate::utils;

/// Parcourt le contour et retourne les arêtes extérieures dans l'ordre
pub fn trace_border_edges(territory: &HashSet<Hex>) -> Vec<BorderEdge> {
    if territory.is_empty() {
        return Vec::new();
    }

    let start = *territory.iter().min_by_key(|h| (h.x(), h.y())).unwrap();

    let start_dir = (0..6)
        .find(|&d| !territory.contains(&(start + Hex::NEIGHBORS_COORDS[d])))
        .expect("L'hex de départ doit avoir une arête extérieure");

    let mut result = Vec::new();
    let mut current_hex = start;
    let mut current_dir = start_dir;
    let mut is_first = true;

    loop {
        // Vérifier si on a bouclé (retour au point de départ)
        if !is_first && current_hex == start && current_dir == start_dir {
            break;
        }
        is_first = false;

        // Ajouter cette arête au résultat
        result.push(BorderEdge {
            hex: current_hex,
            dir: current_dir,
        });

        // Trouver la prochaine arête extérieure
        (current_hex, current_dir) = utils::hex::next_exterior_edge(current_hex, current_dir, territory);

        // Sécurité anti-boucle infinie
        if result.len() > territory.len() * 6 {
            panic!("Boucle infinie dans trace_border_edges");
        }
    }

    result
}

/// Simplifie le contour en utilisant les diagonales.
pub fn simplify_contour_with_diagonals(
    layout: &HexLayout,
    edges: &[BorderEdge],
    territory: &HashSet<Hex>,
) -> Vec<Vec2> {
    if edges.len() < 3 {
        return edges
            .iter()
            .map(|e| utils::hex::edge_midpoint(layout, e.hex, e.dir))
            .collect();
    }

    let n = edges.len();

    // Étape 1 : Identifier les jonctions et les arêtes qui génèrent un sommet
    let mut is_junction = vec![false; n];
    let mut vertex_diag_index: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        let prev2_idx = (i + n - 2) % n;
        let prev_idx = (i + n - 1) % n;
        let next_idx = (i + 1) % n;
        let next2_idx = (i + 2) % n;

        let hex_prev2 = edges[prev2_idx].hex;
        let hex_prev = edges[prev_idx].hex;
        let hex_curr = edges[i].hex;
        let hex_next = edges[next_idx].hex;
        let hex_next2 = edges[next2_idx].hex;

        // Vérifier si c'est une transition diagonale
        // Concave case
        if let Some(diag_info) = detect_diagonal_transition(hex_prev, hex_curr, hex_next, territory)
        {
            // tracing::info!("Transition ({:?}): ({},{}) -> ({},{})", diag_info.curvature, hex_prev.x, hex_prev.y, hex_next.x, hex_next.y);
            // edges[i] est une jonction : ne génère pas de point
            is_junction[i] = true;
            is_junction[(i + 2) % n] = true;

            // edges[prev_idx] doit générer un sommet diagonal
            // On stocke l'index de la diagonale pour savoir quel sommet générer
            vertex_diag_index[prev_idx] = Some(diag_info.diag_index);
        }

        // Convex case
        if let Some(diag_info) = detect_diagonal_transition(hex_prev2, hex_curr, hex_next2, territory)
        {
            // tracing::info!("Transition ({:?}): ({},{}) -> ({},{})", diag_info.curvature, hex_prev2.x, hex_prev2.y, hex_next2.x, hex_next2.y);
            // edges[i] est une jonction : ne génère pas de point
            is_junction[i] = true;

            // vertex_diag_index[pre]
        }
    }

    // Étape 2 : Générer les points
    let mut points = Vec::new();

    for i in 0..n {
        // Sauter les jonctions
        if is_junction[i] {
            continue;
        }

        if let Some(diag_index) = vertex_diag_index[i] {
            // Cette arête génère un sommet diagonal
            points.push(utils::hex::edge_midpoint(layout, edges[i].hex, diag_index));
        } else {
            // Cette arête génère un milieu d'arête normal
            points.push(utils::hex::edge_midpoint(layout, edges[i].hex, edges[i].dir));
        }
    }

    points
}

/// Détecte si le triplet (hex_prev, hex_curr, hex_next) forme une transition diagonale.
/// hex_curr est potentiellement la jonction entre hex_prev et hex_next.
pub fn detect_diagonal_transition(
    hex_prev: Hex,
    hex_curr: Hex,
    hex_next: Hex,
    territory: &HashSet<Hex>,
) -> Option<DiagonalTransitionInfo> {
    // Les 3 hexagones doivent être distincts
    if hex_prev == hex_curr || hex_curr == hex_next || hex_prev == hex_next {
        return None;
    }

    // hex_prev et hex_next doivent être en relation diagonale
    let relation = hex_relation(hex_prev, hex_next);

    match relation {
        HexRelation::Diagonal {
            diag_index,
            junction_a,
            junction_b,
        } => {
            // hex_curr doit être exactement une des jonctions
            if hex_curr != junction_a && hex_curr != junction_b {
                return None;
            }

            // L'AUTRE jonction doit être dans le territoire
            // (c'est ce qui permet de "couper" par la diagonale)
            let other_junction = if hex_curr == junction_a {
                junction_b
            } else {
                junction_a
            };

            if territory.contains(&other_junction) {
                return Some(DiagonalTransitionInfo { diag_index, curvature: DiagonalTransitionCurvature::Convexe });
            }
            
            Some(DiagonalTransitionInfo { diag_index, curvature: DiagonalTransitionCurvature::Concave })
        }
        _ => None,
    }
}

pub fn build_contour(
    layout: &HexLayout,
    territory: &HashSet<Hex>,
    jitter_amplitude: f32, // ex: 2.0 pixels
    jitter_seed: u64,      // ex: 12345 - même seed pour tous les territoires
) -> Vec<Vec2> {
    let edges = trace_border_edges(territory);
    let mut points = simplify_contour_with_diagonals(layout, &edges, territory);

    // Appliquer le jitter avant la suppression des points colinéaires
    // pour que le jitter puisse créer de légères variations
    utils::jittering::apply_jitter(&mut points, jitter_amplitude, jitter_seed);

    points
}