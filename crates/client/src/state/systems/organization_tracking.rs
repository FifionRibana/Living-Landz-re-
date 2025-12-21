use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hexx::Hex;

use crate::camera::MainCamera;
use crate::networking::client::NetworkClient;
use crate::state::resources::CurrentOrganization;
use shared::grid::{GridCell, GridConfig};

/// System that tracks the hovered cell and requests organization info when it changes
pub fn track_hovered_cell_organization(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_config: Res<GridConfig>,
    mut current_organization: ResMut<CurrentOrganization>,
    mut network_client: ResMut<NetworkClient>,
    // Check if cursor is over UI to avoid queries during UI interaction
    ui_interaction_query: Query<&Interaction, (With<Node>, With<Pickable>)>,
) -> Result {
    let window = windows.single()?;
    let (camera, camera_transform) = cameras.single()?;

    // Check if cursor is over any UI element
    let is_over_ui = ui_interaction_query
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Hovered | Interaction::Pressed));

    // Don't query organization if hovering over UI
    if is_over_ui {
        return Ok(());
    }

    if let Some(position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
    {
        let hex_position: Hex = grid_config.layout.world_pos_to_hex(position);
        let current_cell = GridCell {
            q: hex_position.x,
            r: hex_position.y,
        };

        // Only send request if we've moved to a different cell
        let should_request = match &current_organization.last_queried_cell {
            Some(last_cell) => last_cell.q != current_cell.q || last_cell.r != current_cell.r,
            None => true,
        };

        if should_request {
            network_client.send_message(shared::protocol::ClientMessage::RequestOrganizationAtCell {
                cell: current_cell.clone(),
            });
            current_organization.last_queried_cell = Some(current_cell);
        }
    }

    Ok(())
}
