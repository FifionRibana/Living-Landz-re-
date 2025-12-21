use bevy::prelude::*;
use hexx::Hex;

use crate::{
    grid::components::UnitIndicator,
    state::resources::UnitsCache,
};
use shared::grid::GridConfig;

/// Système pour dessiner les indicateurs d'unités avec des cercles blancs
pub fn draw_unit_indicators(
    mut gizmos: Gizmos,
    units_cache: Res<UnitsCache>,
    grid_config: Res<GridConfig>,
) {
    for (cell, units) in units_cache.get_all_cells_with_units() {
        let hex = Hex::new(cell.q, cell.r);
        let base_pos = grid_config.layout.hex_to_world_pos(hex);

        let unit_count = units.len();

        // Calculer les positions pour afficher plusieurs cercles
        // Si 1 unité: centre
        // Si 2 unités: gauche et droite
        // Si 3+ unités: disposition en cercle
        let positions = calculate_circle_positions(unit_count, 0.4);

        for offset in positions {
            let pos = base_pos + offset;

            // Dessiner plusieurs cercles pour créer une bordure épaisse
            // Rayon extérieur: 0.25, rayon intérieur: 0.18
            let outer_radius = 0.25;
            let inner_radius = 0.18;
            let steps = 10; // Nombre de cercles pour l'épaisseur

            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let radius = inner_radius + (outer_radius - inner_radius) * t;
                gizmos.circle_2d(pos, radius, Color::WHITE);
            }
        }
    }
}

/// Calcule les positions des cercles en fonction du nombre d'unités
fn calculate_circle_positions(count: usize, radius: f32) -> Vec<Vec2> {
    match count {
        0 => vec![],
        1 => vec![Vec2::ZERO],
        2 => vec![
            Vec2::new(-radius / 2.0, 0.0),
            Vec2::new(radius / 2.0, 0.0),
        ],
        3 => vec![
            Vec2::new(0.0, radius * 0.6),
            Vec2::new(-radius * 0.5, -radius * 0.3),
            Vec2::new(radius * 0.5, -radius * 0.3),
        ],
        _ => {
            // Pour 4+ unités, disposition en cercle
            let mut positions = Vec::new();
            let angle_step = std::f32::consts::TAU / count as f32;

            for i in 0..count {
                let angle = angle_step * i as f32;
                positions.push(Vec2::new(
                    angle.cos() * radius,
                    angle.sin() * radius,
                ));
            }

            positions
        }
    }
}
