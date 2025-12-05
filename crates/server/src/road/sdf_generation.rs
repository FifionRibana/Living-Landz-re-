use super::data::*;
use super::intersection::*;
use bevy::prelude::*;
use rayon::prelude::*;
use shared::RoadChunkSdfData;

// ============================================================================
// FONCTIONS SDF
// ============================================================================

/// Distance signée à un segment de ligne
/// Retourne (distance, t_paramètre, longueur_segment)
fn sdf_segment(p: Vec2, a: Vec2, b: Vec2) -> (f32, f32, f32) {
    let pa = p - a;
    let ba = b - a;
    let len_sq = ba.length_squared();

    if len_sq < 0.0001 {
        return (pa.length(), 0.0, 0.0);
    }

    let t = (pa.dot(ba) / len_sq).clamp(0.0, 1.0);
    let closest = a + ba * t;
    (p.distance(closest), t, len_sq.sqrt())
}

/// Distance signée à un cercle
fn sdf_circle(p: Vec2, center: Vec2, radius: f32) -> f32 {
    p.distance(center) - radius
}

/// Smooth minimum pour des unions naturelles
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = ((k - (a - b).abs()) / k).clamp(0.0, 1.0);
    a.min(b) - h * h * k * 0.25
}

/// Bruit de hash simple (pour les bords organiques)
fn hash21(p: Vec2) -> f32 {
    let p3 = Vec3::new(p.x, p.y, p.x) % 0.1031;
    let p3 = Vec3::new(
        p3.x + p3.dot(Vec3::new(p3.y, p3.z, p3.x) + 33.33),
        p3.y + p3.dot(Vec3::new(p3.y, p3.z, p3.x) + 33.33),
        p3.z + p3.dot(Vec3::new(p3.y, p3.z, p3.x) + 33.33),
    );
    ((p3.x + p3.y) * p3.z).fract()
}

/// Bruit 2D simple
fn noise2d(p: Vec2) -> f32 {
    let i = p.floor();
    let f = p.fract();
    let u = f * f * (Vec2::splat(3.0) - 2.0 * f);

    let a = hash21(i);
    let b = hash21(i + Vec2::new(1.0, 0.0));
    let c = hash21(i + Vec2::new(0.0, 1.0));
    let d = hash21(i + Vec2::new(1.0, 1.0));

    a * (1.0 - u.x) * (1.0 - u.y)
        + b * u.x * (1.0 - u.y)
        + c * (1.0 - u.x) * u.y
        + d * u.x * u.y
}

// ============================================================================
// GÉNÉRATION SDF CPU
// ============================================================================

/// Structure pour les données intermédiaires d'un pixel
#[derive(Clone)]
struct PixelData {
    distance: f32,
    tangent: Vec2,
    importance: f32,
    in_intersection: bool,
    has_tracks: bool,
}

impl Default for PixelData {
    fn default() -> Self {
        Self {
            distance: 99999.0,
            tangent: Vec2::X,
            importance: 0.0,
            in_intersection: false,
            has_tracks: false,
        }
    }
}

/// Génère le SDF des routes pour un chunk
/// Utilise Rayon pour paralléliser les calculs par ligne
pub fn generate_road_sdf(
    segments: &[RoadSegment],
    intersections: &[Intersection],
    config: &RoadConfig,
    chunk_x: i32,
    chunk_y: i32,
) -> RoadChunkSdfData {
    let width = config.sdf_resolution.x as u16;
    let height = config.sdf_resolution.y as u16;

    // Calculer l'offset du chunk dans l'espace monde
    let chunk_offset = Vec2::new(
        chunk_x as f32 * config.chunk_size.x,
        chunk_y as f32 * config.chunk_size.y,
    );

    let chunk_min = chunk_offset;
    let chunk_max = chunk_offset + config.chunk_size;

    // Debug: Log chunk bounds and segment positions
    tracing::info!(
        "Chunk ({},{}) bounds: ({:.1},{:.1}) -> ({:.1},{:.1})",
        chunk_x, chunk_y, chunk_min.x, chunk_min.y, chunk_max.x, chunk_max.y
    );

    for (i, seg) in segments.iter().enumerate() {
        if !seg.points.is_empty() {
            let first = seg.points[0];
            let last = seg.points[seg.points.len() - 1];
            tracing::info!(
                "  Segment {}: ({:.1},{:.1}) -> ({:.1},{:.1}), cells ({},{}) -> ({},{})",
                i, first.x, first.y, last.x, last.y,
                seg.start_cell.q, seg.start_cell.r,
                seg.end_cell.q, seg.end_cell.r
            );
        }
    }

    // Créer la grille de résultat
    let mut sdf_data = RoadChunkSdfData::new(width, height);

    // Calculer les données SDF en parallèle par ligne
    let rows: Vec<Vec<PixelData>> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![PixelData::default(); width as usize];

            for x in 0..width {
                let pixel_data = compute_pixel_sdf(
                    x,
                    y,
                    width,
                    height,
                    segments,
                    intersections,
                    config,
                    chunk_offset,
                );
                row[x as usize] = pixel_data;
            }

            row
        })
        .collect();

    // Encoder les données dans le format RG16
    for y in 0..height {
        for x in 0..width {
            let pixel = &rows[y as usize][x as usize];

            // Encoder la distance (R16)
            // Distance normalisée : -max_dist -> 0 -> +max_dist  =>  0 -> 32768 -> 65535
            let max_dist = 50.0;
            let normalized_dist = ((pixel.distance / max_dist).clamp(-1.0, 1.0) + 1.0) * 0.5;
            let distance_raw = (normalized_dist * 65535.0) as u16;

            // Encoder les métadonnées (G16)
            // Bits 0-7: Importance * 64
            // Bit 8: Flag tracks
            // Bit 9: Flag intersection
            let mut metadata_raw: u16 = ((pixel.importance * 64.0) as u16) & 0xFF;
            if pixel.has_tracks {
                metadata_raw |= 0x100; // Bit 8
            }
            if pixel.in_intersection {
                metadata_raw |= 0x200; // Bit 9
            }

            sdf_data.set_pixel(x, y, distance_raw, metadata_raw);
        }
    }

    // Debug: Analyze generated SDF
    let total_pixels = (width as usize) * (height as usize);
    let mut min_dist = u16::MAX;
    let mut max_dist = u16::MIN;
    let mut non_max_count = 0;

    for y in 0..height {
        for x in 0..width {
            if let Some((dist, _meta)) = sdf_data.get_pixel(x, y) {
                min_dist = min_dist.min(dist);
                max_dist = max_dist.max(dist);
                if dist < u16::MAX {
                    non_max_count += 1;
                }
            }
        }
    }

    tracing::info!(
        "Generated road SDF: {}x{}, min_dist: {}, max_dist: {}, non_max_pixels: {}/{}, {} segments, {} intersections",
        width, height, min_dist, max_dist, non_max_count, total_pixels,
        segments.len(), intersections.len()
    );

    sdf_data
}

/// Calcule le SDF pour un pixel donné
fn compute_pixel_sdf(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    segments: &[RoadSegment],
    intersections: &[Intersection],
    config: &RoadConfig,
    chunk_offset: Vec2,
) -> PixelData {
    // Position dans l'espace monde (locale au chunk + offset du chunk)
    let uv = Vec2::new(
        (x as f32 + 0.5) / width as f32,
        (y as f32 + 0.5) / height as f32,
    );
    let world_pos = uv * config.chunk_size + chunk_offset;

    let mut result = PixelData::default();

    // =========================================
    // Phase 1 : Distance aux segments de route
    // =========================================

    for segment in segments {
        for i in 1..segment.points.len() {
            let prev_pt = segment.points[i - 1];
            let curr_pt = segment.points[i];

            // Calculer la distance au segment
            let (dist, t, _len) = sdf_segment(world_pos, prev_pt, curr_pt);

            // Largeur effective basée sur l'importance
            let width = config.base_width + segment.importance as f32 * config.width_per_importance;

            // Distance signée (négatif = à l'intérieur)
            let signed_dist = dist - width;

            if signed_dist < result.distance {
                result.distance = signed_dist;

                // Interpoler la tangente
                let prev_tangent = calculate_tangent(&segment.points, i - 1);
                let curr_tangent = calculate_tangent(&segment.points, i);
                result.tangent = prev_tangent.lerp(curr_tangent, t).normalize_or(Vec2::X);

                result.importance = segment.importance as f32;
            }
        }
    }

    // =========================================
    // Phase 2 : Intersections (placettes)
    // =========================================

    for inter in intersections {
        let circle_dist = sdf_circle(world_pos, inter.position, inter.radius);

        // Union lisse avec les routes existantes
        let k = config.intersection_smoothness;
        let combined = smin(result.distance, circle_dist, k);

        if combined < result.distance {
            result.distance = combined;

            // Si on est dans le cercle, mettre à jour les attributs
            if circle_dist < 0.0 {
                result.in_intersection = true;
                result.importance = result.importance.max(inter.importance as f32);
            }
        }
    }

    // =========================================
    // Phase 3 : Ornières pour routes importantes
    // =========================================

    if result.importance >= config.double_track_threshold as f32 && !result.in_intersection {
        let mut track_dist = 99999.0;

        for segment in segments {
            if segment.importance < config.double_track_threshold {
                continue;
            }

            for i in 1..segment.points.len() {
                let prev_pt = segment.points[i - 1];
                let curr_pt = segment.points[i];

                // Calculer la tangente et la perpendiculaire
                let prev_tangent = calculate_tangent(&segment.points, i - 1);
                let curr_tangent = calculate_tangent(&segment.points, i);
                let avg_tangent = (prev_tangent + curr_tangent).normalize_or(Vec2::X);
                let perp = Vec2::new(-avg_tangent.y, avg_tangent.x);

                // Ornière gauche
                let left_a = prev_pt + perp * config.track_offset_left;
                let left_b = curr_pt + perp * config.track_offset_left;
                let (dist_l, _, _) = sdf_segment(world_pos, left_a, left_b);
                track_dist = f32::min(track_dist, dist_l - config.track_width);

                // Ornière droite (décalage asymétrique)
                let right_a = prev_pt - perp * config.track_offset_right;
                let right_b = curr_pt - perp * config.track_offset_right;
                let (dist_r, _, _) = sdf_segment(world_pos, right_a, right_b);
                track_dist = f32::min(track_dist, dist_r - config.track_width);
            }
        }

        result.has_tracks = track_dist < 0.0;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::GridCell;

    #[test]
    fn test_sdf_segment() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);

        // Point sur le segment
        let p1 = Vec2::new(5.0, 0.0);
        let (dist1, t1, len1) = sdf_segment(p1, a, b);
        assert!(dist1.abs() < 0.01);
        assert!((t1 - 0.5).abs() < 0.01);
        assert!((len1 - 10.0).abs() < 0.01);

        // Point à côté du segment
        let p2 = Vec2::new(5.0, 3.0);
        let (dist2, t2, _) = sdf_segment(p2, a, b);
        assert!((dist2 - 3.0).abs() < 0.01);
        assert!((t2 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_sdf_circle() {
        let center = Vec2::new(10.0, 10.0);
        let radius = 5.0;

        // Point au centre
        let dist1 = sdf_circle(center, center, radius);
        assert!((dist1 + 5.0).abs() < 0.01);

        // Point sur le cercle
        let p2 = Vec2::new(15.0, 10.0);
        let dist2 = sdf_circle(p2, center, radius);
        assert!(dist2.abs() < 0.01);

        // Point hors du cercle
        let p3 = Vec2::new(20.0, 10.0);
        let dist3 = sdf_circle(p3, center, radius);
        assert!((dist3 - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_road_sdf() {
        let config = RoadConfig::default();

        // Créer un segment simple
        let start_cell = GridCell { q: 0, r: 0 };
        let end_cell = GridCell { q: 1, r: 0 };
        let segment = RoadSegment {
            id: 1,
            start_cell: start_cell.clone(),
            end_cell: end_cell.clone(),
            cell_path: vec![start_cell, end_cell],
            points: vec![
                Vec2::new(100.0, 250.0),
                Vec2::new(500.0, 250.0),
            ],
            importance: 1,
        };

        let segments = vec![segment];
        let intersections = vec![];

        let sdf = generate_road_sdf(&segments, &intersections, &config, 0, 0);

        assert_eq!(sdf.resolution_x, config.sdf_resolution.x as u16);
        assert_eq!(sdf.resolution_y, config.sdf_resolution.y as u16);

        // Vérifier qu'on peut lire les pixels
        let (dist, meta) = sdf.get_pixel(32, 32).unwrap();
        assert!(dist > 0); // Devrait avoir une valeur
    }
}
