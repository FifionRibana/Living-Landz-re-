use super::data::*;
use bevy::prelude::*;
use shared::grid::GridCell;
use std::collections::HashMap;

/// Calcule toutes les intersections d'un chunk à partir de ses segments
pub fn compute_intersections(
    segments: &[RoadSegment],
    config: &RoadConfig,
) -> Vec<Intersection> {
    // Étape 1 : Collecter les connexions par cellule (nœud)
    let mut cell_connections: HashMap<GridCell, Vec<CellConnection>> = HashMap::new();
    let mut cell_positions: HashMap<GridCell, Vec2> = HashMap::new();

    for segment in segments {
        // Connexion au nœud de départ
        let start_dir = segment.start_direction();
        cell_connections
            .entry(segment.start_cell)
            .or_default()
            .push(CellConnection {
                direction: start_dir,
                importance: segment.importance,
                other_cell: segment.end_cell,
            });
        cell_positions.insert(segment.start_cell, segment.points[0]);

        // Connexion au nœud de fin
        let end_dir = segment.end_direction();
        cell_connections
            .entry(segment.end_cell)
            .or_default()
            .push(CellConnection {
                direction: end_dir,
                importance: segment.importance,
                other_cell: segment.start_cell,
            });
        cell_positions.insert(segment.end_cell, *segment.points.last().unwrap());
    }

    // Étape 2 : Créer les intersections
    let mut intersections = Vec::new();

    for (cell, connections) in cell_connections {
        // Un terminus ou une simple continuation sans vraie intersection
        if connections.len() < 2 {
            continue;
        }

        let position = cell_positions[&cell];
        let directions: Vec<Vec2> = connections.iter().map(|c| c.direction).collect();
        let max_importance = connections.iter().map(|c| c.importance).max().unwrap_or(0);

        let intersection_type = IntersectionType::classify(&directions, config.fork_angle_threshold);
        let radius = calculate_intersection_radius(
            &intersection_type,
            connections.len(),
            max_importance,
            config,
        );

        intersections.push(Intersection {
            position,
            cell,
            intersection_type,
            connected_directions: directions,
            radius,
            importance: max_importance,
        });
    }

    intersections
}

/// Données temporaires pour une connexion à une cellule
struct CellConnection {
    direction: Vec2,
    importance: u8,
    other_cell: GridCell,
}

/// Calcule le rayon d'une intersection
fn calculate_intersection_radius(
    intersection_type: &IntersectionType,
    num_connections: usize,
    importance: u8,
    config: &RoadConfig,
) -> f32 {
    let base = config.intersection_base_radius;
    let per_conn = config.intersection_radius_per_connection;
    let importance_factor = 1.0 + importance as f32 * 0.15;
    let type_factor = intersection_type.radius_factor();

    let connection_bonus = match num_connections {
        0..=2 => 0.0,
        3 => per_conn,
        4 => per_conn * 2.0,
        n => per_conn * (n - 2) as f32,
    };

    (base + connection_bonus) * type_factor * importance_factor
}

/// Calcule la tangente à un point d'une polyline
pub fn calculate_tangent(points: &[Vec2], index: usize) -> Vec2 {
    let n = points.len();

    if n < 2 {
        return Vec2::X;
    }

    if index == 0 {
        // Premier point : direction vers le suivant
        (points[1] - points[0]).normalize_or_zero()
    } else if index >= n - 1 {
        // Dernier point : direction depuis le précédent
        (points[n - 1] - points[n - 2]).normalize_or_zero()
    } else {
        // Point intermédiaire : moyenne des directions adjacentes
        let prev_dir = (points[index] - points[index - 1]).normalize_or_zero();
        let next_dir = (points[index + 1] - points[index]).normalize_or_zero();
        ((prev_dir + next_dir) * 0.5).normalize_or_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_classification() {
        // Deux directions opposées = continuation
        let dirs = vec![Vec2::X, -Vec2::X];
        assert_eq!(
            IntersectionType::classify(&dirs, 0.4),
            IntersectionType::Continuation
        );

        // Deux directions à 90° = fourche
        let dirs = vec![Vec2::X, Vec2::Y];
        assert_eq!(
            IntersectionType::classify(&dirs, 0.4),
            IntersectionType::Fork
        );

        // Trois directions = junction
        let dirs = vec![Vec2::X, Vec2::Y, -Vec2::X];
        assert_eq!(
            IntersectionType::classify(&dirs, 0.4),
            IntersectionType::Junction
        );

        // Quatre directions = crossroad
        let dirs = vec![Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y];
        assert_eq!(
            IntersectionType::classify(&dirs, 0.4),
            IntersectionType::Crossroad
        );
    }

    #[test]
    fn test_tangent_calculation() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
        ];

        // Direction au départ : vers la droite
        let start_tangent = calculate_tangent(&points, 0);
        assert!((start_tangent - Vec2::X).length() < 0.01);

        // Direction à la fin : vers le haut
        let end_tangent = calculate_tangent(&points, 2);
        assert!((end_tangent - Vec2::Y).length() < 0.01);
    }
}
