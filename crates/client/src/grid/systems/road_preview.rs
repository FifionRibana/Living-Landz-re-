use crate::grid::resources::{RoadPreview, SelectedHexes};
use crate::ui::components::ActionCategory;
use crate::ui::resources::ActionState;
use bevy::prelude::*;
use hexx::{Hex, HexLayout};
use shared::grid::{
    GridCell,
    pathfinding::{NeighborType, PathfindingOptions, find_path},
};

/// Système qui met à jour le preview de route en fonction des cellules sélectionnées
pub fn update_road_preview(
    selected_hexes: Res<SelectedHexes>,
    mut road_preview: ResMut<RoadPreview>,
    action_state: Res<ActionState>,
) {
    // Vérifier que nous sommes en mode construction de route
    let is_road_mode = matches!(action_state.selected_category, Some(ActionCategory::Roads));

    if !is_road_mode {
        // Si nous ne sommes pas en mode route, effacer le preview
        if !road_preview.is_empty() {
            road_preview.clear();
        }
        return;
    }

    let selected: Vec<Hex> = selected_hexes.ids.iter().copied().collect();

    // Si moins de 1 cellule sélectionnée, pas de preview
    if selected.is_empty() {
        if !road_preview.is_empty() {
            road_preview.clear();
        }
        return;
    }

    // Si une seule cellule, afficher juste un point
    if selected.len() == 1 {
        let cell = GridCell::from_hex(&selected[0]);
        road_preview.path = vec![cell];
        road_preview.world_points = vec![cell_to_world_pos(&cell)];
        road_preview.is_valid = true;
        return;
    }

    // Utiliser la première et la dernière cellule sélectionnée
    let start_hex = selected.first().unwrap();
    let end_hex = selected.last().unwrap();

    let start_cell = GridCell::from_hex(start_hex);
    let end_cell = GridCell::from_hex(end_hex);

    // Calculer le chemin avec pathfinding
    let path = if start_cell == end_cell {
        // Cas spécial: même cellule
        vec![start_cell]
    } else {
        // Vérifier si les cellules sont voisines
        let is_direct_neighbor = start_cell.neighbors().contains(&end_cell);
        let is_indirect_neighbor = start_cell.indirect_neighbors().contains(&end_cell);

        if is_direct_neighbor || is_indirect_neighbor {
            // Voisins: chemin direct
            vec![start_cell, end_cell]
        } else {
            // Pathfinding
            match find_path(
                start_cell,
                end_cell,
                PathfindingOptions {
                    neighbor_type: NeighborType::Both,
                    ..Default::default()
                },
            ) {
                Some(p) => p,
                None => {
                    // Pas de chemin trouvé
                    road_preview.path.clear();
                    road_preview.world_points.clear();
                    road_preview.is_valid = false;
                    return;
                }
            }
        }
    };

    // Convertir le chemin en positions monde
    let cell_positions: Vec<Vec3> = path.iter().map(cell_to_world_pos).collect();

    // Générer une spline lisse pour le preview (interpolation Catmull-Rom simple)
    let world_points = generate_preview_spline(&cell_positions, 8);

    // Mettre à jour le preview
    road_preview.path = path;
    road_preview.world_points = world_points;
    road_preview.is_valid = true;
}

/// Génère une spline lisse pour le preview (version simplifiée de Catmull-Rom)
fn generate_preview_spline(control_points: &[Vec3], samples_per_segment: usize) -> Vec<Vec3> {
    if control_points.len() < 2 {
        return control_points.to_vec();
    }

    if control_points.len() == 2 {
        // Interpolation linéaire pour 2 points
        let mut result = Vec::new();
        for i in 0..=samples_per_segment {
            let t = i as f32 / samples_per_segment as f32;
            result.push(control_points[0].lerp(control_points[1], t));
        }
        return result;
    }

    // Pour 3+ points, utiliser Catmull-Rom
    let mut result = Vec::new();
    let mut extended = Vec::with_capacity(control_points.len() + 2);
    extended.push(control_points[0]); // Dupliquer premier point
    extended.extend_from_slice(control_points);
    extended.push(control_points[control_points.len() - 1]); // Dupliquer dernier point

    for i in 0..(extended.len() - 3) {
        let p0 = extended[i];
        let p1 = extended[i + 1];
        let p2 = extended[i + 2];
        let p3 = extended[i + 3];

        for j in 0..samples_per_segment {
            let t = j as f32 / samples_per_segment as f32;
            let point = catmull_rom_point_vec3(p0, p1, p2, p3, t);
            result.push(point);
        }
    }

    result.push(*control_points.last().unwrap());
    result
}

/// Calcule un point sur une courbe de Catmull-Rom (version Vec3)
fn catmull_rom_point_vec3(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    let coef0 = -0.5 * t3 + t2 - 0.5 * t;
    let coef1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let coef2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let coef3 = 0.5 * t3 - 0.5 * t2;

    p0 * coef0 + p1 * coef1 + p2 * coef2 + p3 * coef3
}

/// Convertit une cellule hexagonale en position monde (Vec3)
fn cell_to_world_pos(cell: &GridCell) -> Vec3 {
    use shared::constants::{HEX_RATIO, HEX_SIZE};

    let layout = HexLayout::flat()
        .with_hex_size(HEX_SIZE)
        .with_scale(Vec2::new(HEX_RATIO.x * HEX_SIZE, HEX_RATIO.y * HEX_SIZE));

    let hex = Hex::new(cell.q, cell.r);
    let world_pos = layout.hex_to_world_pos(hex);

    // Elever légèrement au-dessus du sol pour le preview
    Vec3::new(world_pos.x, 0.5, world_pos.y)
}

/// Système qui dessine le preview de la route avec Gizmos
pub fn draw_road_preview(road_preview: Res<RoadPreview>, mut gizmos: Gizmos, time: Res<Time>) {
    if !road_preview.is_valid || road_preview.world_points.is_empty() {
        return;
    }

    // Couleur du preview - jaune/orange avec animation
    let base_color = if road_preview.is_valid {
        Color::srgba(1.0, 0.8, 0.2, 1.0) // Jaune/orange pour preview valide
    } else {
        Color::srgba(1.0, 0.2, 0.2, 1.0) // Rouge pour preview invalide
    };

    // Animation pulsante
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5;
    let alpha = 0.6 + pulse * 0.4;
    let color = Color::srgba(
        base_color.to_srgba().red,
        base_color.to_srgba().green,
        base_color.to_srgba().blue,
        alpha,
    );

    // Dessiner une ligne pointillée entre chaque point
    for i in 0..road_preview.world_points.len() - 1 {
        let start = road_preview.world_points[i];
        let end = road_preview.world_points[i + 1];

        // Nombre de segments pour la ligne pointillée
        let dash_count = 8;

        for j in 0..dash_count {
            // Alterner entre trait et espace
            if j % 2 == 0 {
                let t_start = j as f32 / dash_count as f32;
                let t_end = (j + 1) as f32 / dash_count as f32;

                let dash_start = start.lerp(end, t_start);
                let dash_end = start.lerp(end, t_end);

                gizmos.line(dash_start, dash_end, color);
            }
        }
    }

    // Dessiner des sphères aux points clés (début, fin, et quelques intermédiaires)
    if let Some(&first) = road_preview.world_points.first() {
        gizmos.sphere(first, 0.3, Color::srgba(0.2, 1.0, 0.2, alpha)); // Vert pour le début
    }

    if road_preview.world_points.len() > 1
        && let Some(&last) = road_preview.world_points.last()
    {
        gizmos.sphere(last, 0.3, Color::srgba(1.0, 0.2, 0.2, alpha)); // Rouge pour la fin
    }

    // Dessiner des petites sphères aux points intermédiaires
    for (i, &point) in road_preview.world_points.iter().enumerate() {
        if i > 0 && i < road_preview.world_points.len() - 1 {
            gizmos.sphere(point, 0.15, color);
        }
    }
}
