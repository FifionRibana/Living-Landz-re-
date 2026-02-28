use bevy::prelude::*;
// use hexx::Hex;

use crate::{
    grid::resources::SelectedHexes, networking::client::NetworkClient,
    state::resources::CurrentOrganization,
};
use shared::{
    OrganizationType,
    grid::{GridCell, GridConfig},
    protocol::ClientMessage,
};

/// Système pour gérer les raccourcis clavier de debug
pub fn handle_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_hexes: Res<SelectedHexes>,
    current_organization: Option<Res<CurrentOrganization>>,
    _grid_config: Res<GridConfig>,
    mut network_client: ResMut<NetworkClient>,
) {
    // Vérifier si Shift est maintenu
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if !shift_pressed {
        return;
    }

    // Shift + H: Créer un Hamlet (hameau)
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell {
                q: selected_hex.x,
                r: selected_hex.y,
            };

            network_client.send_message(ClientMessage::DebugCreateOrganization {
                name: format!("Hamlet_{}", selected_hex.x),
                organization_type: OrganizationType::Hamlet,
                cell: cell.clone(),
                parent_organization_id: None,
            });
            info!("Debug: Creating Hamlet at ({}, {})", cell.q, cell.r);
        } else {
            warn!("Debug: No cell selected to create organization");
        }
    }

    // Shift + V: Créer un Village
    if keyboard.just_pressed(KeyCode::KeyV) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell {
                q: selected_hex.x,
                r: selected_hex.y,
            };

            network_client.send_message(ClientMessage::DebugCreateOrganization {
                name: format!("Village_{}", selected_hex.x),
                organization_type: OrganizationType::Village,
                cell: cell.clone(),
                parent_organization_id: None,
            });
            info!("Debug: Creating Village at ({}, {})", cell.q, cell.r);
        } else {
            warn!("Debug: No cell selected to create organization");
        }
    }

    // Shift + T: Créer une Town (ville)
    if keyboard.just_pressed(KeyCode::KeyT) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell {
                q: selected_hex.x,
                r: selected_hex.y,
            };

            network_client.send_message(ClientMessage::DebugCreateOrganization {
                name: format!("Town_{}", selected_hex.x),
                organization_type: OrganizationType::Town,
                cell: cell.clone(),
                parent_organization_id: None,
            });
            info!("Debug: Creating Town at ({}, {})", cell.q, cell.r);
        } else {
            warn!("Debug: No cell selected to create organization");
        }
    }

    // Shift + C: Créer une City (cité)
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell {
                q: selected_hex.x,
                r: selected_hex.y,
            };

            network_client.send_message(ClientMessage::DebugCreateOrganization {
                name: format!("City_{}", selected_hex.x),
                organization_type: OrganizationType::City,
                cell: cell.clone(),
                parent_organization_id: None,
            });
            info!("Debug: Creating City at ({}, {})", cell.q, cell.r);
        } else {
            warn!("Debug: No cell selected to create organization");
        }
    }

    // Shift + U: Spawn une unité aléatoire
    if keyboard.just_pressed(KeyCode::KeyU) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell {
                q: selected_hex.x,
                r: selected_hex.y,
            };

            network_client.send_message(ClientMessage::DebugSpawnUnit { cell: cell.clone() });
            info!("Debug: Spawning unit at ({}, {})", cell.q, cell.r);
        } else {
            warn!("Debug: No cell selected to spawn unit");
        }
    }

    // Shift + D: Supprimer l'organisation sur la cellule actuelle
    if keyboard.just_pressed(KeyCode::KeyD) {
        if let Some(current_organization) = current_organization
            && let Some(org) = &current_organization.organization
        {
            network_client.send_message(ClientMessage::DebugDeleteOrganization {
                organization_id: org.id,
            });
            info!(
                "Debug: Deleting organization '{}' (ID: {})",
                org.name, org.id
            );
        } else {
            warn!("Debug: No organization on current cell to delete");
        }
    }

    // Shift + H: Afficher l'aide des raccourcis debug
    if keyboard.just_pressed(KeyCode::KeyH) {
        info!("=== DEBUG KEYBOARD SHORTCUTS ===");
        info!("Shift + M: Create Hamlet on selected cell");
        info!("Shift + V: Create Village on selected cell");
        info!("Shift + T: Create Town on selected cell");
        info!("Shift + C: Create City on selected cell");
        info!("Shift + U: Spawn random unit on selected cell");
        info!("Shift + D: Delete organization on hovered cell");
        info!("Shift + H: Show this help");
        info!("================================");
    }
}
