use bevy::prelude::*;
use shared::{ActionStatusEnum, TerrainChunkId};

use crate::{
    grid::resources::SelectedHexes,
    state::resources::ActionTracker,
    ui::components::CellDetailsPanelMarker,
};

/// Système pour afficher l'état de l'action en cours dans le panneau de détails
pub fn update_cell_action_display(
    action_tracker: Res<ActionTracker>,
    selected_hexes: Res<SelectedHexes>,
    mut panel_query: Query<(&mut Visibility, &Children), With<CellDetailsPanelMarker>>,
    mut text_query: Query<&mut Text>,
) {
    // Vérifier si une cellule est sélectionnée
    if selected_hexes.ids.is_empty() {
        return;
    }

    // Prendre la première cellule sélectionnée
    let Some(selected_cell) = selected_hexes.ids.iter().next() else {
        return;
    };

    // Convertir Hex en GridCell
    let grid_cell = shared::grid::GridCell {
        q: selected_cell.x,
        r: selected_cell.y,
    };

    // TODO: Obtenir le chunk_id de la cellule sélectionnée
    // Pour l'instant, on utilise un chunk par défaut
    let chunk_id = TerrainChunkId { x: 0, y: 0 };

    // Vérifier s'il y a une action sur cette cellule
    if let Some(action) = action_tracker.get_action_on_cell(&chunk_id, &grid_cell) {
        // Il y a une action sur cette cellule
        let action_status_text = match action.status {
            ActionStatusEnum::Pending => "⏸ En attente".to_string(),
            ActionStatusEnum::InProgress => {
                // Calculer le temps restant
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let remaining_seconds = if current_time < action.completion_time {
                    action.completion_time - current_time
                } else {
                    0
                };

                let minutes = remaining_seconds / 60;
                let seconds = remaining_seconds % 60;

                format!("⏳ En cours... {}:{:02}", minutes, seconds)
            }
            ActionStatusEnum::Completed => "✓ Terminée".to_string(),
            ActionStatusEnum::Failed => "✗ Échouée".to_string(),
        };

        let action_type_text = format!("{:?}", action.action_type);

        // Mettre à jour les textes du panneau
        // TODO: Identifier et mettre à jour les bons composants de texte
        // Pour l'instant, on log juste l'info
        // info!(
        //     "Action sur la cellule: {} - {}",
        //     action_type_text, action_status_text
        // );
    }
}

/// Vérifie si une cellule a une action en cours et retourne true si oui
pub fn has_active_action_on_selected_cell(
    action_tracker: &ActionTracker,
    selected_hexes: &SelectedHexes,
) -> bool {
    if selected_hexes.ids.is_empty() {
        return false;
    }

    let Some(selected_cell) = selected_hexes.ids.iter().next() else {
        return false;
    };

    // Convertir Hex en GridCell
    let grid_cell = shared::grid::GridCell {
        q: selected_cell.x,
        r: selected_cell.y,
    };

    // TODO: Obtenir le chunk_id de la cellule sélectionnée
    let chunk_id = TerrainChunkId { x: 0, y: 0 };

    action_tracker.get_action_on_cell(&chunk_id, &grid_cell).is_some()
}

/// Système pour masquer le panneau d'actions quand une action est en cours sur la cellule sélectionnée
pub fn hide_action_panel_during_action(
    action_tracker: Res<ActionTracker>,
    selected_hexes: Res<SelectedHexes>,
    mut action_panel_query: Query<&mut Visibility, With<crate::ui::components::ActionsPanelMarker>>,
) {
    // Ne vérifier que si la sélection a changé ou si une action a été mise à jour
    if !selected_hexes.is_changed() && !action_tracker.is_changed() {
        return;
    }

    let has_action = has_active_action_on_selected_cell(&action_tracker, &selected_hexes);

    for mut visibility in action_panel_query.iter_mut() {
        if has_action {
            // Masquer le panneau d'actions si une action est en cours
            *visibility = Visibility::Hidden;
        } else {
            // Sinon, laisser le panneau visible si une cellule est sélectionnée
            *visibility = if selected_hexes.ids.is_empty() {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}
