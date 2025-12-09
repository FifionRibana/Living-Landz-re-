use super::grid_cell::GridCell;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Noeud pour l'algorithme A*
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    cell: GridCell,
    g_cost: u32,  // Coût depuis le départ
    h_cost: u32,  // Heuristique jusqu'à l'arrivée
}

impl PathNode {
    fn f_cost(&self) -> u32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverser pour avoir un min-heap
        other.f_cost().cmp(&self.f_cost())
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Calcule la distance hexagonale entre deux cellules
fn hex_distance(a: &GridCell, b: &GridCell) -> u32 {
    let dx = (a.q - b.q).abs();
    let dy = (a.r - b.r).abs();
    let dz = ((-a.q - a.r) - (-b.q - b.r)).abs();
    ((dx + dy + dz) / 2) as u32
}

/// Type de voisinage pour le pathfinding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborType {
    /// Voisins directs uniquement (distance 1)
    Direct,
    /// Voisins indirects uniquement (distance 2)
    Indirect,
    /// Voisins directs ET indirects
    Both,
}

/// Options pour le pathfinding
pub struct PathfindingOptions {
    /// Type de voisinage à utiliser
    pub neighbor_type: NeighborType,
    /// Fonction de coût pour se déplacer d'une cellule à une autre
    /// Par défaut: 10 pour direct, 14 pour indirect
    pub cost_fn: Option<Box<dyn Fn(&GridCell, &GridCell) -> u32>>,
    /// Limite maximale de cellules à explorer (pour éviter les boucles infinies)
    pub max_iterations: usize,
}

impl Default for PathfindingOptions {
    fn default() -> Self {
        Self {
            neighbor_type: NeighborType::Both,
            cost_fn: None,
            max_iterations: 10000,
        }
    }
}

/// Trouve le chemin le plus court entre deux cellules en utilisant A*
///
/// # Arguments
/// * `start` - Cellule de départ
/// * `end` - Cellule d'arrivée
/// * `options` - Options de pathfinding
///
/// # Returns
/// * `Some(Vec<GridCell>)` - Le chemin du départ à l'arrivée (inclus)
/// * `None` - Aucun chemin trouvé
pub fn find_path(start: GridCell, end: GridCell, options: PathfindingOptions) -> Option<Vec<GridCell>> {
    // Si départ = arrivée
    if start == end {
        return Some(vec![start]);
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<GridCell, GridCell> = HashMap::new();
    let mut g_scores: HashMap<GridCell, u32> = HashMap::new();
    let mut closed_set: HashSet<GridCell> = HashSet::new();

    // Fonction de coût par défaut
    let default_cost_fn = |a: &GridCell, b: &GridCell| -> u32 {
        let dist = hex_distance(a, b);
        if dist == 1 {
            10 // Coût direct
        } else {
            14 // Coût indirect (approximation de sqrt(2) * 10)
        }
    };

    let cost_fn = options.cost_fn.as_ref().map(|f| f.as_ref()).unwrap_or(&default_cost_fn);

    g_scores.insert(start, 0);
    open_set.push(PathNode {
        cell: start,
        g_cost: 0,
        h_cost: hex_distance(&start, &end),
    });

    let mut iterations = 0;

    while let Some(current_node) = open_set.pop() {
        iterations += 1;
        if iterations > options.max_iterations {
            // Max iterations reached, no path found
            return None;
        }

        let current = current_node.cell;

        // Arrivée atteinte
        if current == end {
            return Some(reconstruct_path(&came_from, current));
        }

        // Déjà visité
        if !closed_set.insert(current) {
            continue;
        }

        // Explorer les voisins
        let neighbors = match options.neighbor_type {
            NeighborType::Direct => current.neighbors(),
            NeighborType::Indirect => current.indirect_neighbors(),
            NeighborType::Both => current.all_extended_neighbors(),
        };

        for neighbor in neighbors {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g_score = g_scores.get(&current).unwrap_or(&u32::MAX)
                .saturating_add(cost_fn(&current, &neighbor));

            let current_g_score = *g_scores.get(&neighbor).unwrap_or(&u32::MAX);

            if tentative_g_score < current_g_score {
                came_from.insert(neighbor, current);
                g_scores.insert(neighbor, tentative_g_score);

                open_set.push(PathNode {
                    cell: neighbor,
                    g_cost: tentative_g_score,
                    h_cost: hex_distance(&neighbor, &end),
                });
            }
        }
    }

    // Aucun chemin trouvé
    None
}

/// Reconstruit le chemin à partir de la map came_from
fn reconstruct_path(came_from: &HashMap<GridCell, GridCell>, mut current: GridCell) -> Vec<GridCell> {
    let mut path = vec![current];
    while let Some(&previous) = came_from.get(&current) {
        path.push(previous);
        current = previous;
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_neighbors_path() {
        let start = GridCell { q: 0, r: 0 };
        let end = GridCell { q: 2, r: 0 };

        let path = find_path(start, end, PathfindingOptions {
            neighbor_type: NeighborType::Direct,
            ..Default::default()
        });

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&end));
        assert_eq!(path.len(), 3); // (0,0) -> (1,0) -> (2,0)
    }

    #[test]
    fn test_same_start_end() {
        let start = GridCell { q: 5, r: 3 };
        let path = find_path(start, start, PathfindingOptions::default());

        assert_eq!(path, Some(vec![start]));
    }

    #[test]
    fn test_indirect_neighbors_shorter() {
        let start = GridCell { q: 0, r: 0 };
        let end = GridCell { q: 2, r: -1 };

        let path = find_path(start, end, PathfindingOptions {
            neighbor_type: NeighborType::Both,
            ..Default::default()
        });

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&end));
        // Avec indirect, on peut y aller directement
        assert_eq!(path.len(), 2); // (0,0) -> (2,-1)
    }
}
