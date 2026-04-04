use bevy::prelude::*;

use crate::{
    grid::resources::SelectedHexes,
    networking::client::game_client::{
        SendDebugCreateOrganization, SendDebugDeleteOrganization, SendDebugSpawnUnit,
    },
    state::resources::CurrentOrganization,
};
use shared::{
    OrganizationType,
    grid::{GridCell, GridConfig},
};

/// Debug keyboard shortcuts system
pub fn handle_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_hexes: Res<SelectedHexes>,
    current_organization: Option<Res<CurrentOrganization>>,
    _grid_config: Res<GridConfig>,
    mut create_org_events: MessageWriter<SendDebugCreateOrganization>,
    mut delete_org_events: MessageWriter<SendDebugDeleteOrganization>,
    mut spawn_unit_events: MessageWriter<SendDebugSpawnUnit>,
) {
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if !shift_pressed {
        return;
    }

    // Shift + M: Create Hamlet
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell { q: selected_hex.x, r: selected_hex.y };
            create_org_events.write(SendDebugCreateOrganization {
                name: format!("Hamlet_{}", selected_hex.x),
                organization_type: OrganizationType::Hamlet,
                cell,
                parent_organization_id: None,
            });
            info!("Debug: Creating Hamlet at ({}, {})", cell.q, cell.r);
        }
    }

    // Shift + V: Create Village
    if keyboard.just_pressed(KeyCode::KeyV) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell { q: selected_hex.x, r: selected_hex.y };
            create_org_events.write(SendDebugCreateOrganization {
                name: format!("Village_{}", selected_hex.x),
                organization_type: OrganizationType::Village,
                cell,
                parent_organization_id: None,
            });
            info!("Debug: Creating Village at ({}, {})", cell.q, cell.r);
        }
    }

    // Shift + T: Create Town
    if keyboard.just_pressed(KeyCode::KeyT) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell { q: selected_hex.x, r: selected_hex.y };
            create_org_events.write(SendDebugCreateOrganization {
                name: format!("Town_{}", selected_hex.x),
                organization_type: OrganizationType::Town,
                cell,
                parent_organization_id: None,
            });
            info!("Debug: Creating Town at ({}, {})", cell.q, cell.r);
        }
    }

    // Shift + C: Create City
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell { q: selected_hex.x, r: selected_hex.y };
            create_org_events.write(SendDebugCreateOrganization {
                name: format!("City_{}", selected_hex.x),
                organization_type: OrganizationType::City,
                cell,
                parent_organization_id: None,
            });
            info!("Debug: Creating City at ({}, {})", cell.q, cell.r);
        }
    }

    // Shift + U: Spawn unit
    if keyboard.just_pressed(KeyCode::KeyU) {
        if let Some(selected_hex) = selected_hexes.ids.iter().next() {
            let cell = GridCell { q: selected_hex.x, r: selected_hex.y };
            spawn_unit_events.write(SendDebugSpawnUnit { cell });
            info!("Debug: Spawning unit at ({}, {})", cell.q, cell.r);
        }
    }

    // Shift + D: Delete organization
    if keyboard.just_pressed(KeyCode::KeyD) {
        if let Some(current_organization) = current_organization
            && let Some(org) = &current_organization.organization
        {
            delete_org_events.write(SendDebugDeleteOrganization {
                organization_id: org.id,
            });
            info!("Debug: Deleting organization '{}' (ID: {})", org.name, org.id);
        }
    }

    // Shift + H: Help
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
