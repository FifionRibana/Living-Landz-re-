use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;
use crate::rendering::territory::{
    TerritoryBorderCellsDebug, TerritoryBorderSdfCache, TerritoryContourCache,
};
use crate::state::resources::CurrentOrganization;

/// Handles territory-related messages (contours, border SDF, border cells, organization at cell).
pub fn handle_territory_events(
    mut events: MessageReader<ServerEvent>,
    mut territory_border_cache: ResMut<TerritoryBorderSdfCache>,
    mut territory_contour_cache: ResMut<TerritoryContourCache>,
    mut current_organization: Option<ResMut<CurrentOrganization>>,
    mut commands: Commands,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::TerritoryContourUpdate { chunk_id, contours } => {
                info!(
                    "✓ Received {} territory contours for chunk ({},{})",
                    contours.len(),
                    chunk_id.x,
                    chunk_id.y
                );

                for contour_data in contours {
                    territory_contour_cache.add_contour(
                        *chunk_id,
                        contour_data.organization_id,
                        contour_data
                            .segments
                            .iter()
                            .map(|s| s.to_contour_segment())
                            .collect(),
                        Color::linear_rgba(
                            contour_data.border_color.r,
                            contour_data.border_color.g,
                            contour_data.border_color.b,
                            contour_data.border_color.a,
                        ),
                        Color::linear_rgba(
                            contour_data.fill_color.r,
                            contour_data.fill_color.g,
                            contour_data.fill_color.b,
                            contour_data.fill_color.a,
                        ),
                    );
                }
            }

            ServerMessage::TerritoryBorderSdfUpdate {
                chunk_id,
                border_sdf_data_list,
            } => {
                info!(
                    "✓ Received territory border SDF for chunk ({},{}) [DEPRECATED]",
                    chunk_id.x, chunk_id.y
                );
                territory_border_cache
                    .chunks
                    .insert((chunk_id.x, chunk_id.y), border_sdf_data_list.clone());
            }

            ServerMessage::TerritoryBorderCells {
                organization_id,
                border_cells,
            } => {
                info!(
                    "✓ Received {} border cells for organization {}",
                    border_cells.len(),
                    organization_id
                );
                commands.insert_resource(TerritoryBorderCellsDebug {
                    organization_id: *organization_id,
                    border_cells: border_cells.clone(),
                });
            }

            ServerMessage::OrganizationAtCell { cell, organization } => {
                let Some(ref mut current_organization) = current_organization else {
                    continue;
                };
                current_organization.update(*cell, organization.clone());
            }

            _ => {}
        }
    }
}
