use bevy::prelude::*;

/// Génère une spline de Catmull-Rom passant par les points de contrôle donnés
/// Retourne une liste de points échantillonnés le long de la courbe
pub fn generate_catmull_rom_spline(control_points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if control_points.len() < 2 {
        return control_points.to_vec();
    }

    if control_points.len() == 2 {
        // Interpolation linéaire simple pour 2 points
        return interpolate_linear(&control_points[0], &control_points[1], samples_per_segment);
    }

    let mut result = Vec::new();

    // Pour une spline de Catmull-Rom, on a besoin de 4 points pour chaque segment
    // On étend le début et la fin en dupliquant les points extrêmes
    let mut extended_points = Vec::with_capacity(control_points.len() + 2);
    extended_points.push(control_points[0]); // Duplicat du premier point
    extended_points.extend_from_slice(control_points);
    extended_points.push(control_points[control_points.len() - 1]); // Duplicat du dernier point

    // Générer les segments de courbe
    for i in 0..(extended_points.len() - 3) {
        let p0 = extended_points[i];
        let p1 = extended_points[i + 1];
        let p2 = extended_points[i + 2];
        let p3 = extended_points[i + 3];

        // Échantillonner le segment
        for j in 0..samples_per_segment {
            let t = j as f32 / samples_per_segment as f32;
            let point = catmull_rom_point(p0, p1, p2, p3, t);
            result.push(point);
        }
    }

    // Ajouter le dernier point
    result.push(*control_points.last().unwrap());

    result
}

/// Calcule un point sur une courbe de Catmull-Rom
fn catmull_rom_point(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    // Formule de Catmull-Rom avec tension 0.5
    let coef0 = -0.5 * t3 + t2 - 0.5 * t;
    let coef1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let coef2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let coef3 = 0.5 * t3 - 0.5 * t2;

    p0 * coef0 + p1 * coef1 + p2 * coef2 + p3 * coef3
}

/// Interpole linéairement entre deux points
fn interpolate_linear(start: &Vec2, end: &Vec2, samples: usize) -> Vec<Vec2> {
    let mut result = Vec::with_capacity(samples + 1);

    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        result.push(start.lerp(*end, t));
    }

    result
}

/// Génère une courbe de Bézier quadratique (utile pour les virages doux)
pub fn generate_bezier_quadratic(p0: Vec2, p1_control: Vec2, p2: Vec2, samples: usize) -> Vec<Vec2> {
    let mut result = Vec::with_capacity(samples + 1);

    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let point = bezier_quadratic_point(p0, p1_control, p2, t);
        result.push(point);
    }

    result
}

/// Calcule un point sur une courbe de Bézier quadratique
fn bezier_quadratic_point(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let one_minus_t = 1.0 - t;
    let coef0 = one_minus_t * one_minus_t;
    let coef1 = 2.0 * one_minus_t * t;
    let coef2 = t * t;

    p0 * coef0 + p1 * coef1 + p2 * coef2
}

/// Génère une spline continue de Catmull-Rom sur un chemin de cellules
/// Retourne une courbe lisse et harmonieuse passant par tous les centres des cellules
pub fn generate_path_spline(cell_positions: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if cell_positions.len() < 2 {
        return cell_positions.to_vec();
    }

    if cell_positions.len() == 2 {
        // Pour 2 points, utiliser une courbe de Bézier simple avec variation organique
        let start = cell_positions[0];
        let end = cell_positions[1];

        // Point de contrôle au milieu avec léger décalage
        let mid = (start + end) * 0.5;
        let dir = (end - start).normalize_or_zero();
        let perp = Vec2::new(-dir.y, dir.x);

        // Décalage proportionnel à la distance
        let distance = start.distance(end);
        let offset = distance * 0.1; // 10% de décalage

        let control = mid + perp * offset;

        return generate_bezier_quadratic(start, control, end, samples_per_segment);
    }

    // Pour 3+ points, utiliser Catmull-Rom pour une courbe C1 continue
    generate_catmull_rom_spline(cell_positions, samples_per_segment)
}

/// Étend une spline existante en ajoutant un nouveau point au début ou à la fin
/// Permet de régénérer N cellules avant/après pour un lissage optimal avec Catmull-Rom
pub fn extend_spline(
    existing_cell_positions: &[Vec2],  // Positions monde des cellules existantes
    existing_points: &[Vec2],          // Points de spline existants
    new_cell_position: Vec2,           // Position monde de la nouvelle cellule
    at_start: bool,                    // Ajouter au début (true) ou à la fin (false)
    samples_per_segment: usize,        // Nombre de points par segment de cellule
    smoothing_influence: usize,        // Nombre de cellules à régénérer avant/après (0 = juste nouveau segment)
) -> Vec<Vec2> {
    // Cas de base : première cellule
    if existing_cell_positions.is_empty() {
        return vec![new_cell_position];
    }

    // Créer le nouveau cell_path complet
    let mut new_cell_positions = existing_cell_positions.to_vec();
    if at_start {
        new_cell_positions.insert(0, new_cell_position);
    } else {
        new_cell_positions.push(new_cell_position);
    }

    // Si smoothing_influence = 0, on utilise juste une connexion Bézier sans régénération
    if smoothing_influence == 0 {
        if at_start {
            let connection_point = existing_points[0];
            let tangent_point = if existing_points.len() > samples_per_segment {
                existing_points[samples_per_segment]
            } else {
                existing_points[existing_points.len() - 1]
            };
            let tangent = (connection_point - tangent_point).normalize_or_zero();
            let control_point = connection_point + tangent * new_cell_position.distance(connection_point) * 0.3;

            let mut new_segment = generate_bezier_quadratic(
                new_cell_position,
                control_point,
                connection_point,
                samples_per_segment
            );
            new_segment.pop();
            new_segment.extend_from_slice(existing_points);
            return new_segment;
        } else {
            let connection_point = existing_points[existing_points.len() - 1];
            let start_idx = if existing_points.len() > samples_per_segment {
                existing_points.len() - samples_per_segment - 1
            } else {
                0
            };
            let tangent_point = existing_points[start_idx];
            let tangent = (connection_point - tangent_point).normalize_or_zero();
            let control_point = connection_point + tangent * new_cell_position.distance(connection_point) * 0.3;

            let mut new_segment = generate_bezier_quadratic(
                connection_point,
                control_point,
                new_cell_position,
                samples_per_segment
            );
            new_segment.remove(0);
            let mut result = existing_points.to_vec();
            result.extend(new_segment);
            return result;
        }
    }

    // SIMPLIFICATION: Régénérer toujours toute la spline pour éviter les artefacts
    // La fusion partielle de splines Catmull-Rom est trop complexe et causait:
    // - Duplication de points (82 au lieu de 49)
    // - Persistance visuelle de l'ancien tracé
    // - Bugs difficiles à déboguer
    //
    // Pour les routes courantes (< 50 cellules), la régénération complète est très rapide
    // Le paramètre smoothing_influence est conservé pour compatibilité mais n'a plus d'effet
    let _ = smoothing_influence;  // Silencer warning unused
    let _ = at_start;  // Silencer warning unused
    let _ = existing_points;  // Silencer warning unused

    generate_path_spline(&new_cell_positions, samples_per_segment)
}

/// Génère une courbe organique entre deux points en ajoutant un point de contrôle décalé
pub fn generate_organic_curve(start: Vec2, end: Vec2, samples: usize, seed: u64) -> Vec<Vec2> {
    if samples < 2 {
        return vec![start, end];
    }

    // Calculer le point milieu
    let mid = (start + end) * 0.5;

    // Direction perpendiculaire à la ligne start-end
    let dir = (end - start).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);

    // Décalage basé sur le seed pour que chaque segment soit unique
    let hash = ((start.x * 73856093.0 + start.y * 19349663.0 + seed as f32) % 1.0) * 2.0 - 1.0;

    // Amplitude du décalage (proportionnelle à la distance entre les points)
    let distance = start.distance(end);
    let offset_amount = distance * 0.15 * hash; // 15% de la distance

    // Point de contrôle décalé perpendiculairement
    let control = mid + perp * offset_amount;

    // Générer une courbe de Bézier quadratique
    generate_bezier_quadratic(start, control, end, samples)
}

/// Ajoute une variation organique subtile aux points de la route
/// en appliquant un petit décalage perpendiculaire à la direction
pub fn add_organic_variation(points: &[Vec2], variation_amount: f32, seed: u64) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]); // Premier point inchangé

    for i in 1..(points.len() - 1) {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];

        // Direction moyenne
        let dir = (next - prev).normalize_or_zero();

        // Perpendiculaire
        let perp = Vec2::new(-dir.y, dir.x);

        // Variation pseudo-aléatoire basée sur la position et le seed
        let hash = (curr.x * 73856093.0 + curr.y * 19349663.0 + seed as f32) % 1.0;
        let offset = (hash - 0.5) * variation_amount;

        result.push(curr + perp * offset);
    }

    result.push(*points.last().unwrap()); // Dernier point inchangé

    result
}

/// Simplifie un chemin en supprimant les points redondants (algorithme de Douglas-Peucker)
pub fn simplify_path(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    douglas_peucker_recursive(points, tolerance)
}

fn douglas_peucker_recursive(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let first = points[0];
    let last = points[points.len() - 1];

    // Trouver le point le plus éloigné de la ligne first-last
    let (max_distance, max_index) = points[1..(points.len() - 1)]
        .iter()
        .enumerate()
        .map(|(i, &p)| (point_line_distance(p, first, last), i + 1))
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();

    if max_distance > tolerance {
        // Récursivement simplifier les deux moitiés
        let mut left = douglas_peucker_recursive(&points[0..=max_index], tolerance);
        let right = douglas_peucker_recursive(&points[max_index..], tolerance);

        // Combiner (sans dupliquer le point du milieu)
        left.pop();
        left.extend(right);
        left
    } else {
        // Tous les points intermédiaires sont proches de la ligne, les supprimer
        vec![first, last]
    }
}

fn point_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    let line_len_sq = line_vec.length_squared();

    if line_len_sq < 0.0001 {
        return point_vec.length();
    }

    let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;

    (point - projection).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_passes_through_points() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
        ];

        let spline = generate_catmull_rom_spline(&points, 10);

        // La spline doit passer par le premier point
        assert!((spline[0] - points[0]).length() < 0.1);

        // La spline doit passer par le dernier point
        assert!((spline[spline.len() - 1] - points[points.len() - 1]).length() < 0.1);
    }

    #[test]
    fn test_linear_interpolation() {
        let points = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)];
        let result = interpolate_linear(&points[0], &points[1], 10);

        assert_eq!(result.len(), 11);
        assert_eq!(result[0], points[0]);
        assert_eq!(result[10], points[1]);
        assert_eq!(result[5], Vec2::new(50.0, 50.0));
    }
}
