use bevy::prelude::*;
use hexx::Hex;

use crate::{
    // grid::components::UnitIndicator,
    state::resources::{PlayerInfo, UnitsCache, UnitsDataCache},
    ui::resources::UnitSelectionState,
};
use shared::grid::GridConfig;

/// Système pour dessiner les indicateurs d'unités avec des cercles blancs
/// Les unités sélectionnées sont affichées en vert.
pub fn draw_unit_indicators(
    mut gizmos: Gizmos,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    unit_selection: Res<UnitSelectionState>,
    player_info: Res<PlayerInfo>,
    grid_config: Res<GridConfig>,
) {
    let lord_id = player_info.lord.as_ref().map(|l| l.id);

    for (cell, units) in units_cache.get_all_cells_with_units() {
        let hex = Hex::new(cell.q, cell.r);
        let base_pos = grid_config.layout.hex_to_world_pos(hex);

        let unit_count = units.len();

        // Calculer les positions pour afficher plusieurs cercles
        // Si 1 unité: centre
        // Si 2 unités: gauche et droite
        // Si 3+ unités: disposition en cercle
        let positions = calculate_circle_positions(unit_count, 0.4);

        for (i, offset) in positions.iter().enumerate() {
            let pos = base_pos + *offset;
            let unit_id = units[i];

            let is_selected = unit_selection.is_selected(unit_id);
            let is_lord = lord_id == Some(unit_id);

            let color = if is_selected {
                Color::srgb(0.3, 1.0, 0.3) // Vert = sélectionné
            } else if is_lord {
                Color::srgb(0.9, 0.75, 0.2) // Or = lord
            } else {
                Color::WHITE // Blanc = NPC
            };

            // Dessiner plusieurs cercles pour créer une bordure épaisse
            let outer_radius = if is_selected { 0.30 } else { 0.25 };
            let inner_radius = if is_selected { 0.20 } else { 0.18 };
            let steps = 10;

            for s in 0..steps {
                let t = s as f32 / steps as f32;
                let radius = inner_radius + (outer_radius - inner_radius) * t;
                gizmos.circle_2d(pos, radius, color);
            }

            // Croix au centre pour le lord (pour mieux le distinguer)
            if is_lord {
                let cross_size = 0.12;
                let lord_color = Color::srgb(0.9, 0.75, 0.2);
                gizmos.line_2d(
                    pos + Vec2::new(-cross_size, 0.0),
                    pos + Vec2::new(cross_size, 0.0),
                    lord_color,
                );
                gizmos.line_2d(
                    pos + Vec2::new(0.0, -cross_size),
                    pos + Vec2::new(0.0, cross_size),
                    lord_color,
                );
            }
        }
    }
}

/// Calcule les positions des cercles en fonction du nombre d'unités
fn calculate_circle_positions(count: usize, radius: f32) -> Vec<Vec2> {
    match count {
        0 => vec![],
        1 => vec![Vec2::ZERO],
        2 => vec![Vec2::new(-radius / 2.0, 0.0), Vec2::new(radius / 2.0, 0.0)],
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
                positions.push(Vec2::new(angle.cos() * radius, angle.sin() * radius));
            }

            positions
        }
    }
}
