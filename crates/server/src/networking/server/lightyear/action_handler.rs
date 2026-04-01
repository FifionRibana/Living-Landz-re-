use std::sync::Arc;

use shared::grid::GridCell;
use shared::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId};

use crate::action_processor::ActionProcessor;
use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;

use super::bridge::{ActionRequest, ActionRequestReceiver, BridgeEvent, BridgeSender};

/// Start the action RPC handler as a tokio task.
/// This loops forever, processing ActionRequests from Bevy and pushing
/// responses back through the bridge.
pub fn start_action_rpc_handler(
    mut receiver: ActionRequestReceiver,
    bridge_sender: Arc<BridgeSender>,
    db_tables: Arc<DatabaseTables>,
    action_processor: Arc<ActionProcessor>,
    dev_config: Arc<DevConfig>,
) {
    tokio::spawn(async move {
        tracing::info!("🎮 Action RPC handler started");

        while let Some(request) = receiver.rx.recv().await {
            match request {
                ActionRequest::MoveUnit {
                    player_id,
                    unit_id,
                    chunk_id,
                    cell,
                } => {
                    handle_move_unit(
                        player_id,
                        unit_id,
                        chunk_id,
                        cell,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                    )
                    .await;
                }
            }
        }

        tracing::warn!("🎮 Action RPC handler stopped — channel closed");
    });
}

/// Process a MoveUnit action request.
/// Same logic as the tungstenite handler, extracted into a standalone function.
async fn handle_move_unit(
    player_id: u64,
    unit_id: u64,
    chunk_id: TerrainChunkId,
    cell: GridCell,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
) {
    use shared::{
        ActionBaseData, ActionData, ActionSpecificTypeEnum, MoveUnitAction, SpecificAction,
    };

    // 1. Load unit from DB
    let unit_data = match db_tables.units.load_unit(unit_id).await {
        Ok(u) => u,
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Unité introuvable: {}", e),
            });
            return;
        }
    };

    // 2. Validate ownership (same logic as player_controls_unit in handlers.rs)
    let owns = if unit_data.player_id == Some(player_id) {
        true
    } else {
        // Check via organization membership: unit belongs to an org led by the player's lord
        let result = sqlx::query(
            r#"
            SELECT 1 FROM organizations.members om
            JOIN organizations.organizations o ON o.id = om.organization_id
            JOIN units.units lord ON lord.id = o.leader_unit_id
            WHERE om.unit_id = $1 AND lord.player_id = $2 AND lord.is_lord = true
            LIMIT 1
            "#,
        )
        .bind(unit_id as i64)
        .bind(player_id as i64)
        .fetch_optional(&db_tables.pool)
        .await;

        matches!(result, Ok(Some(_)))
    };

    if !owns {
        bridge_sender.send(BridgeEvent::SendActionError {
            player_id,
            reason: "Cette unité ne vous appartient pas".to_string(),
        });
        return;
    }

    // 3. Compute distance
    let from_hex = unit_data.current_cell.to_hex();
    let to_hex = cell.to_hex();
    let hex_distance = from_hex.unsigned_distance_to(to_hex) as u64;

    if hex_distance == 0 {
        bridge_sender.send(BridgeEvent::SendActionError {
            player_id,
            reason: "L'unité est déjà sur cette cellule".to_string(),
        });
        return;
    }

    // 4. Compute duration
    let duration_ms = dev_config.apply_speed(hex_distance * 2000);

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let specific_data = SpecificAction::MoveUnit(MoveUnitAction {
        player_id,
        unit_id,
        chunk_id,
        cell,
    });

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell,
            action_type: ActionTypeEnum::MoveUnit,
            action_specific_type: ActionSpecificTypeEnum::MoveUnit,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    // 5. Create action in DB + add to processor cache
    match action_processor
        .add_action_from_rpc(&db_tables.actions, &action_data, ActionTypeEnum::MoveUnit)
        .await
    {
        Ok(action_id) => {
            tracing::info!(
                "Unit {} moving {} hexes from ({},{}) to ({},{}) — {}ms (action {}) [via lightyear]",
                unit_id,
                hex_distance,
                unit_data.current_cell.q,
                unit_data.current_cell.r,
                cell.q,
                cell.r,
                duration_ms,
                action_id
            );

            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::MoveUnit,
                completion_time: start_time + (duration_ms / 1000),
                action_name: None,
                unit_ids: vec![],
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule move action via lightyear: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Échec de la planification: {}", e),
            });
        }
    }
}
