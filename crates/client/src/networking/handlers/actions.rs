use bevy::prelude::*;
use shared::ActionStatusEnum;
use shared::protocol::ServerMessage;

use crate::networking::client::NetworkClient;
use crate::networking::events::ServerEvent;
use crate::state::resources::{ActionTracker, NotificationState, TrackedAction};

/// Handles action-related messages (status updates, completions).
pub fn handle_action_events(
    mut events: MessageReader<ServerEvent>,
    mut action_tracker: Option<ResMut<ActionTracker>>,
    mut network_client: Option<ResMut<NetworkClient>>,
    mut notifications: ResMut<NotificationState>,
) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

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
                let Some(ref mut action_tracker) = action_tracker else {
                    continue;
                };
                info!(
                    "Action {} status update: {:?} for player {} at chunk ({}, {}) cell ({}, {})",
                    action_id, status, player_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                // Capture start_time: use existing if upgrading, else now
                let start_time = action_tracker
                    .get_action(*action_id)
                    .map(|a| a.start_time)
                    .unwrap_or(current_time);

                let tracked_action = TrackedAction {
                    action_id: *action_id,
                    player_id: *player_id,
                    chunk_id: *chunk_id,
                    cell: *cell,
                    action_type: *action_type,
                    status: *status,
                    start_time,
                    completion_time: *completion_time,
                };

                action_tracker.update_action(tracked_action);

                // Push notification
                match status {
                    ActionStatusEnum::Pending => {
                        notifications.push_info(format!("{} en attente...", action_type.to_name()));
                    }
                    ActionStatusEnum::InProgress => {
                        notifications.push_info(format!("{} en cours", action_type.to_name()));
                    }
                    ActionStatusEnum::Failed => {
                        notifications.push_error(format!("{} échouée", action_type.to_name()));
                    }
                    _ => {}
                }
            }

            ServerMessage::ActionCompleted {
                action_id,
                chunk_id,
                cell,
                action_type,
            } => {
                info!(
                    "Action {} completed at chunk ({}, {}) cell ({}, {})",
                    action_id, chunk_id.x, chunk_id.y, cell.q, cell.r
                );

                // Notification de complétion
                notifications.push_success(format!("{} terminée !", action_type.to_name()));

                if let Some(ref mut client) = network_client {
                    info!(
                        "Requesting chunk data refresh for ({}, {})",
                        chunk_id.x, chunk_id.y
                    );
                    client.send_message(shared::protocol::ClientMessage::RequestTerrainChunks {
                        terrain_name: "Gaulyia".to_string(),
                        terrain_chunk_ids: vec![*chunk_id],
                    });
                }
            }

            ServerMessage::ActionError { reason } => {
                warn!("Action rejected by server: {}", reason);
                notifications.push_error(reason.clone());
            }

            _ => {}
        }
    }
}
