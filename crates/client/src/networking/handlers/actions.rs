use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::client::NetworkClient;
use crate::networking::events::ServerEvent;
use crate::state::resources::{ActionTracker, TrackedAction};

/// Handles action-related messages (status updates, completions).
pub fn handle_action_events(
    mut events: MessageReader<ServerEvent>,
    mut action_tracker: Option<ResMut<ActionTracker>>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::ActionStatusUpdate {
                action_id,
                player_id,
                chunk_id,
                cell,
                status,
                action_type,
                completion_time,
            } => {
                let Some(ref mut action_tracker) = action_tracker else { continue };
                info!(
                    "Action {} status update: {:?} for player {} at chunk ({}, {}) cell ({}, {})",
                    action_id, status, player_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                let tracked_action = TrackedAction {
                    action_id: *action_id,
                    player_id: *player_id,
                    chunk_id: *chunk_id,
                    cell: *cell,
                    action_type: *action_type,
                    status: *status,
                    completion_time: *completion_time,
                };

                action_tracker.update_action(tracked_action);
            }

            ServerMessage::ActionCompleted {
                action_id,
                chunk_id,
                cell,
                action_type: _,
            } => {
                info!(
                    "Action {} completed at chunk ({}, {}) cell ({}, {})",
                    action_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                if let Some(ref mut client) = network_client {
                    info!(
                        "Requesting chunk data refresh for ({}, {})",
                        chunk_id.x, chunk_id.y
                    );
                    client.send_message(
                        shared::protocol::ClientMessage::RequestTerrainChunks {
                            terrain_name: "Gaulyia".to_string(),
                            terrain_chunk_ids: vec![*chunk_id],
                        },
                    );
                }
            }

            _ => {}
        }
    }
}
