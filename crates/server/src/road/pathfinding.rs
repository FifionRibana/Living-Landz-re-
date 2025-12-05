use shared::grid::GridCell;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Nœud pour l'algorithme A*
#[derive(Clone, Debug)]
struct Node {
    cell: GridCell,
    g_cost: f32,  // Coût du chemin depuis le départ
    h_cost: f32,  // Heuristique (distance estimée jusqu'à l'arrivée)
}

impl Node {
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverser l'ordre pour avoir un min-heap au lieu d'un max-heap
        other.f_cost().partial_cmp(&self.f_cost())
            .unwrap_or(Ordering::Equal)
    }
}

/// Trouve le chemin le plus court entre deux cellules hexagonales en utilisant A*
pub fn find_path(start: &GridCell, end: &GridCell) -> Option<Vec<GridCell>> {
    if start == end {
        return Some(vec![start.clone()]);
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from: HashMap<GridCell, GridCell> = HashMap::new();
    let mut g_costs: HashMap<GridCell, f32> = HashMap::new();

    // Ajouter le nœud de départ
    open_set.push(Node {
        cell: start.clone(),
        g_cost: 0.0,
        h_cost: heuristic_distance(start, end),
    });
    g_costs.insert(start.clone(), 0.0);

    while let Some(current_node) = open_set.pop() {
        let current = current_node.cell;

        // Arrivée trouvée
        if current == *end {
            return Some(reconstruct_path(&came_from, &current));
        }

        // Déjà visité
        if closed_set.contains(&current) {
            continue;
        }

        closed_set.insert(current.clone());

        // Explorer les voisins
        for neighbor in current.neighbors() {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g_cost = current_node.g_cost + 1.0; // Coût uniforme pour passer à une cellule voisine

            let current_g_cost = g_costs.get(&neighbor).copied().unwrap_or(f32::INFINITY);

            if tentative_g_cost < current_g_cost {
                // Ce chemin est meilleur
                came_from.insert(neighbor.clone(), current.clone());
                g_costs.insert(neighbor.clone(), tentative_g_cost);

                open_set.push(Node {
                    cell: neighbor.clone(),
                    g_cost: tentative_g_cost,
                    h_cost: heuristic_distance(&neighbor, end),
                });
            }
        }
    }

    // Aucun chemin trouvé
    None
}

/// Heuristique de distance entre deux cellules hexagonales (distance de Manhattan sur les coordonnées hexagonales)
fn heuristic_distance(a: &GridCell, b: &GridCell) -> f32 {
    let dq = (a.q - b.q).abs();
    let dr = (a.r - b.r).abs();
    let ds = (a.q + a.r - b.q - b.r).abs();

    ((dq + dr + ds) / 2) as f32
}

/// Reconstruit le chemin à partir de la table came_from
fn reconstruct_path(came_from: &HashMap<GridCell, GridCell>, current: &GridCell) -> Vec<GridCell> {
    let mut path = vec![current.clone()];
    let mut current = current.clone();

    while let Some(previous) = came_from.get(&current) {
        path.push(previous.clone());
        current = previous.clone();
    }

    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_path_same_cell() {
        let start = GridCell { q: 0, r: 0 };
        let end = GridCell { q: 0, r: 0 };

        let path = find_path(&start, &end);
        assert_eq!(path, Some(vec![start]));
    }

    #[test]
    fn test_find_path_adjacent() {
        let start = GridCell { q: 0, r: 0 };
        let end = GridCell { q: 1, r: 0 };

        let path = find_path(&start, &end);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], start);
        assert_eq!(path[1], end);
    }

    #[test]
    fn test_find_path_distant() {
        let start = GridCell { q: 0, r: 0 };
        let end = GridCell { q: 3, r: 3 };

        let path = find_path(&start, &end);
        assert!(path.is_some());
        let path = path.unwrap();

        // Vérifier que le chemin commence et finit aux bons endroits
        assert_eq!(*path.first().unwrap(), start);
        assert_eq!(*path.last().unwrap(), end);

        // Vérifier que chaque étape est adjacente à la suivante
        for i in 0..(path.len() - 1) {
            let neighbors = path[i].neighbors();
            assert!(neighbors.contains(&path[i + 1]));
        }
    }

    #[test]
    fn test_heuristic_distance() {
        let a = GridCell { q: 0, r: 0 };
        let b = GridCell { q: 3, r: 3 };

        let distance = heuristic_distance(&a, &b);
        assert!(distance > 0.0);
        assert!(distance < 10.0); // Distance raisonnable
    }
}
