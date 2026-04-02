use std::sync::Arc;

use shared::grid::GridCell;
use shared::{
    ActionStatusEnum, ActionTypeEnum, BuildingTypeEnum, GameState, ProfessionEnum,
    ResourceSpecificTypeEnum, TerrainChunkId,
};

use crate::action_processor::ActionProcessor;
use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;
use crate::networking::Sessions;
use crate::world::resources::WorldGlobalState;

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
    game_state: Arc<GameState>,
    grid_config: Arc<shared::grid::GridConfig>,
    world_global_state: Arc<WorldGlobalState>,
    sessions: Sessions,
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
                ActionRequest::BuildBuilding {
                    player_id,
                    chunk_id,
                    cell,
                    building_type,
                } => {
                    handle_build_building(
                        player_id,
                        chunk_id,
                        cell,
                        building_type,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                        &game_state,
                        &grid_config,
                    )
                    .await;
                }
                ActionRequest::BuildRoad {
                    player_id,
                    start_cell,
                    end_cell,
                } => {
                    handle_build_road(
                        player_id,
                        start_cell,
                        end_cell,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                    )
                    .await;
                }
                ActionRequest::HarvestResource {
                    player_id,
                    chunk_id,
                    cell,
                    resource_specific_type,
                    unit_ids,
                } => {
                    handle_harvest_resource(
                        player_id,
                        chunk_id,
                        cell,
                        resource_specific_type,
                        unit_ids,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                        &game_state,
                    )
                    .await;
                }
                ActionRequest::CraftResource {
                    player_id,
                    chunk_id,
                    cell,
                    recipe_id,
                    quantity,
                    unit_ids,
                } => {
                    handle_craft_resource(
                        player_id,
                        chunk_id,
                        cell,
                        recipe_id,
                        quantity,
                        unit_ids,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                        &game_state,
                    )
                    .await;
                }
                ActionRequest::TrainUnit {
                    player_id,
                    unit_id,
                    chunk_id,
                    cell,
                    target_profession,
                } => {
                    handle_train_unit(
                        player_id,
                        unit_id,
                        chunk_id,
                        cell,
                        target_profession,
                        &bridge_sender,
                        &db_tables,
                        &action_processor,
                        &dev_config,
                    )
                    .await;
                }
                ActionRequest::Explore {
                    player_id,
                    cell,
                    radius,
                } => {
                    handle_explore(
                        player_id,
                        cell,
                        radius,
                        &db_tables,
                        &grid_config,
                        &world_global_state,
                        &sessions,
                    )
                    .await;
                }
            }
        }

        tracing::warn!("🎮 Action RPC handler stopped — channel closed");
    });
}

// ─── Ownership check helper ──────────────────────────────────────────

/// Check if a player controls a unit (directly or via organization).
async fn player_controls_unit(
    db_tables: &DatabaseTables,
    unit_id: u64,
    player_id: u64,
    unit_player_id: Option<u64>,
) -> bool {
    if unit_player_id == Some(player_id) {
        return true;
    }
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
}

// ─── MoveUnit ────────────────────────────────────────────────────────

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

    // 2. Validate ownership
    if !player_controls_unit(db_tables, unit_id, player_id, unit_data.player_id).await {
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

// ─── BuildBuilding ───────────────────────────────────────────────────

async fn handle_build_building(
    player_id: u64,
    chunk_id: TerrainChunkId,
    cell: GridCell,
    building_type: BuildingTypeEnum,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
    game_state: &GameState,
    grid_config: &shared::grid::GridConfig,
) {
    use shared::{
        ActionBaseData, ActionData, ActionSpecificTypeEnum, BuildBuildingAction, SpecificAction,
    };

    // Recompute chunk_id from cell (same as tungstenite handler)
    let chunk_id = cell.to_chunk_id(&grid_config.layout);
    let building_specific_type = building_type.to_specific_type();

    tracing::info!(
        "Player {} requested to build {:?} ({:?}) at chunk ({},{}) cell ({},{}) [via lightyear]",
        player_id, building_type, building_specific_type,
        chunk_id.x, chunk_id.y, cell.q, cell.r
    );

    // 1. Find the lord
    let lord_unit_id = match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(lord)) => lord.id,
        Ok(None) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Aucun seigneur trouvé".to_string(),
            });
            return;
        }
        Err(e) => {
            tracing::error!("Failed to load lord: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Erreur serveur".to_string(),
            });
            return;
        }
    };

    // 2. Check construction costs
    if !dev_config.skip_resource_check() {
        let bt_id = building_type.to_id() as i32;
        let costs = game_state.building_costs(bt_id);

        if !costs.is_empty() {
            let inventory = match db_tables
                .resources
                .load_inventory_summary(lord_unit_id)
                .await
            {
                Ok(inv) => inv,
                Err(e) => {
                    tracing::error!("Failed to load inventory: {}", e);
                    bridge_sender.send(BridgeEvent::SendActionError {
                        player_id,
                        reason: "Erreur de chargement de l'inventaire".to_string(),
                    });
                    return;
                }
            };

            let mut missing = Vec::new();
            for cost in costs {
                let have = inventory.get(&cost.item_id).copied().unwrap_or(0);
                if have < cost.quantity {
                    let item_name = game_state.item_name(cost.item_id, 1);
                    missing.push(format!(
                        "{} (besoin: {}, possédé: {})",
                        item_name, cost.quantity, have
                    ));
                }
            }

            if !missing.is_empty() {
                bridge_sender.send(BridgeEvent::SendActionError {
                    player_id,
                    reason: format!(
                        "Matériaux de construction manquants : {}",
                        missing.join(", ")
                    ),
                });
                return;
            }
        }
    }

    // 3. Schedule the action
    let specific_data = SpecificAction::BuildBuilding(BuildBuildingAction {
        player_id,
        chunk_id,
        cell,
        building_type,
        building_specific_type,
    });

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let bt_id = building_type.to_id() as i32;
    let duration_ms =
        dev_config.apply_speed((game_state.building_duration_seconds(bt_id) as u64) * 1000);

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell,
            action_type: ActionTypeEnum::BuildBuilding,
            action_specific_type: ActionSpecificTypeEnum::BuildBuilding,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    match action_processor
        .add_action_from_rpc(&db_tables.actions, &action_data, ActionTypeEnum::BuildBuilding)
        .await
    {
        Ok(action_id) => {
            tracing::info!(
                "Scheduled build building action {} [via lightyear]",
                action_id
            );
            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::BuildBuilding,
                completion_time: start_time + (duration_ms / 1000),
                action_name: None,
                unit_ids: vec![],
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule build building action: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to schedule action: {}", e),
            });
        }
    }
}

// ─── BuildRoad ───────────────────────────────────────────────────────

async fn handle_build_road(
    player_id: u64,
    start_cell: GridCell,
    end_cell: GridCell,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
) {
    use shared::{
        ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, BuildRoadAction,
        SpecificAction, SpecificActionData,
    };

    tracing::info!(
        "Player {} requested to build road from ({},{}) to ({},{}) [via lightyear]",
        player_id, start_cell.q, start_cell.r, end_cell.q, end_cell.r
    );

    let specific_data = SpecificAction::BuildRoad(BuildRoadAction {
        player_id,
        start_cell,
        end_cell,
    });

    // Compute chunk from start_cell
    use crate::database::tables::RoadSegmentsTable;
    let chunk_id = RoadSegmentsTable::cell_to_chunk_id(&start_cell);

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let duration_ms = dev_config.apply_speed(specific_data.duration_ms(&ActionContext {
        player_id,
        grid_cell: start_cell,
    }));

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell: start_cell,
            action_type: ActionTypeEnum::BuildRoad,
            action_specific_type: ActionSpecificTypeEnum::BuildRoad,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    match action_processor
        .add_action_from_rpc(&db_tables.actions, &action_data, ActionTypeEnum::BuildRoad)
        .await
    {
        Ok(action_id) => {
            tracing::info!(
                "Scheduled build road action {} [via lightyear]",
                action_id
            );
            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell: start_cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::BuildRoad,
                completion_time: start_time + (duration_ms / 1000),
                action_name: None,
                unit_ids: vec![],
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule build road action: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to schedule action: {}", e),
            });
        }
    }
}

// ─── HarvestResource ─────────────────────────────────────────────────

async fn handle_harvest_resource(
    player_id: u64,
    chunk_id: TerrainChunkId,
    cell: GridCell,
    resource_specific_type: ResourceSpecificTypeEnum,
    unit_ids: Vec<u64>,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
    game_state: &GameState,
) {
    use shared::{
        ActionBaseData, ActionData, ActionSpecificTypeEnum as AST, HarvestResourceAction,
        SpecificAction,
    };

    // 1. Check harvest yields exist for this resource type
    let yields = game_state.harvest_yields_for(resource_specific_type.to_id());
    if yields.is_empty() {
        bridge_sender.send(BridgeEvent::SendActionError {
            player_id,
            reason: format!(
                "Aucun rendement de récolte défini pour ce type de ressource ({:?})",
                resource_specific_type
            ),
        });
        return;
    }

    // 2. Find the lord
    match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(_)) => {} // Lord exists
        Ok(None) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Aucun seigneur trouvé".to_string(),
            });
            return;
        }
        Err(e) => {
            tracing::error!("Failed to load lord: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Erreur serveur".to_string(),
            });
            return;
        }
    }

    // 3. Check production line capacity
    let building_type = db_tables
        .buildings
        .get_building_type_at_cell(&cell)
        .await
        .unwrap_or(None);

    if let Some(bt) = building_type {
        let max_lines = bt.production_lines() as usize;
        let active_count = action_processor
            .active_production_count_on_cell(&cell)
            .await;

        if active_count >= max_lines {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!(
                    "Toutes les lignes de production sont occupées ({}/{})",
                    active_count, max_lines
                ),
            });
            return;
        }
    }

    // 4. Validate units aren't already busy
    if !unit_ids.is_empty() {
        let busy = db_tables
            .units
            .get_busy_units(&unit_ids)
            .await
            .unwrap_or_default();
        if !busy.is_empty() {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Certaines unités sont déjà occupées : {:?}", busy),
            });
            return;
        }
    }

    // 5. Schedule the action
    let specific_data = SpecificAction::HarvestResource(HarvestResourceAction {
        player_id,
        chunk_id,
        cell,
        resource_specific_type,
    });

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let duration_ms = dev_config.apply_speed((yields[0].duration_seconds as u64) * 1000);

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell,
            action_type: ActionTypeEnum::HarvestResource,
            action_specific_type: AST::HarvestResource,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    match action_processor
        .add_action_from_rpc(
            &db_tables.actions,
            &action_data,
            ActionTypeEnum::HarvestResource,
        )
        .await
    {
        Ok(action_id) => {
            // Assign units to this action
            if !unit_ids.is_empty() {
                if let Err(e) = db_tables
                    .units
                    .set_units_working_on(&unit_ids, action_id)
                    .await
                {
                    tracing::error!(
                        "Failed to assign units to action {}: {}",
                        action_id, e
                    );
                }
            }

            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::HarvestResource,
                completion_time: start_time + (duration_ms / 1000),
                action_name: None,
                unit_ids: unit_ids.clone(),
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule harvest action: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Erreur : {}", e),
            });
        }
    }
}

// ─── CraftResource ───────────────────────────────────────────────────

async fn handle_craft_resource(
    player_id: u64,
    chunk_id: TerrainChunkId,
    cell: GridCell,
    recipe_id: String,
    quantity: u32,
    unit_ids: Vec<u64>,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
    game_state: &GameState,
) {
    use shared::{
        ActionBaseData, ActionData, ActionSpecificTypeEnum as AST, CraftResourceAction,
        SpecificAction,
    };

    // 1. Find the recipe (by numeric ID or slug)
    let recipe = match game_state.find_recipe(&recipe_id) {
        Some(r) => r,
        None => {
            tracing::warn!(
                "Player {} requested unknown recipe '{}'",
                player_id, recipe_id
            );
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Recette inconnue : {}", recipe_id),
            });
            return;
        }
    };

    // 2. Find the lord
    let lord_unit_id = match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(lord)) => lord.id,
        Ok(None) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Aucun seigneur trouvé".to_string(),
            });
            return;
        }
        Err(e) => {
            tracing::error!("Failed to load lord for player {}: {}", player_id, e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Erreur serveur".to_string(),
            });
            return;
        }
    };

    // 3. Check ingredients
    if !dev_config.skip_resource_check() && !recipe.ingredients.is_empty() {
        let inventory = match db_tables
            .resources
            .load_inventory_summary(lord_unit_id)
            .await
        {
            Ok(inv) => inv,
            Err(e) => {
                tracing::error!("Failed to load inventory: {}", e);
                bridge_sender.send(BridgeEvent::SendActionError {
                    player_id,
                    reason: "Erreur de chargement de l'inventaire".to_string(),
                });
                return;
            }
        };

        let mut missing = Vec::new();
        for ingredient in &recipe.ingredients {
            let needed = ingredient.quantity * quantity as i32;
            let have = inventory.get(&ingredient.item_id).copied().unwrap_or(0);
            if have < needed {
                let item_name = game_state.item_name(ingredient.item_id, 1).clone();
                missing.push(format!(
                    "{} (besoin: {}, possédé: {})",
                    item_name, needed, have
                ));
            }
        }

        if !missing.is_empty() {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Ressources manquantes : {}", missing.join(", ")),
            });
            return;
        }
    }

    // 4. Check production line capacity
    let building_type = db_tables
        .buildings
        .get_building_type_at_cell(&cell)
        .await
        .unwrap_or(None);

    if let Some(bt) = building_type {
        let max_lines = bt.production_lines() as usize;
        let active_count = action_processor
            .active_production_count_on_cell(&cell)
            .await;

        if active_count >= max_lines {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!(
                    "Toutes les lignes de production sont occupées ({}/{})",
                    active_count, max_lines
                ),
            });
            return;
        }
    }

    // 5. Validate units aren't already busy
    if !unit_ids.is_empty() {
        let busy = db_tables
            .units
            .get_busy_units(&unit_ids)
            .await
            .unwrap_or_default();
        if !busy.is_empty() {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Certaines unités sont déjà occupées : {:?}", busy),
            });
            return;
        }
    }

    // 6. Schedule the action
    let specific_data = SpecificAction::CraftResource(CraftResourceAction {
        player_id,
        recipe_id: recipe.id.to_string(),
        chunk_id,
        cell,
        quantity,
    });

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let duration_ms = dev_config
        .apply_speed((recipe.craft_duration_seconds as u64) * 1000 * (quantity as u64));

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell,
            action_type: ActionTypeEnum::CraftResource,
            action_specific_type: AST::CraftResource,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    match action_processor
        .add_action_from_rpc(
            &db_tables.actions,
            &action_data,
            ActionTypeEnum::CraftResource,
        )
        .await
    {
        Ok(action_id) => {
            // Assign units to this action
            if !unit_ids.is_empty() {
                if let Err(e) = db_tables
                    .units
                    .set_units_working_on(&unit_ids, action_id)
                    .await
                {
                    tracing::error!(
                        "Failed to assign units to action {}: {}",
                        action_id, e
                    );
                }
            }

            tracing::info!(
                "Craft action {} scheduled: recipe '{}' x{} for player {} (duration: {}s) [via lightyear]",
                action_id, recipe.name, quantity, player_id, duration_ms / 1000
            );

            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::CraftResource,
                completion_time: start_time + (duration_ms / 1000),
                action_name: Some(recipe.name.clone()),
                unit_ids: unit_ids.clone(),
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule craft action: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Erreur lors de la planification : {}", e),
            });
        }
    }
}

// ─── TrainUnit ───────────────────────────────────────────────────────

async fn handle_train_unit(
    player_id: u64,
    unit_id: u64,
    chunk_id: TerrainChunkId,
    cell: GridCell,
    target_profession: ProfessionEnum,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    dev_config: &DevConfig,
) {
    use shared::{
        ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum as AST, SpecificAction,
        SpecificActionData, TrainUnitAction,
    };

    // 1. Check production line capacity
    let building_type = db_tables
        .buildings
        .get_building_type_at_cell(&cell)
        .await
        .unwrap_or(None);

    if let Some(bt) = building_type {
        let max_lines = bt.production_lines() as usize;
        let active_count = action_processor
            .active_production_count_on_cell(&cell)
            .await;

        if active_count >= max_lines {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!(
                    "Toutes les lignes de production sont occupées ({}/{})",
                    active_count, max_lines
                ),
            });
            return;
        }
    }

    // 2. Schedule the action
    let specific_data = SpecificAction::TrainUnit(TrainUnitAction {
        player_id,
        unit_id,
        chunk_id,
        cell,
        target_profession,
    });

    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let duration_ms = dev_config.apply_speed(specific_data.duration_ms(&ActionContext {
        player_id,
        grid_cell: cell,
    }));

    let action_data = ActionData {
        base_data: ActionBaseData {
            player_id,
            chunk: chunk_id,
            cell,
            action_type: ActionTypeEnum::TrainUnit,
            action_specific_type: AST::TrainUnit,
            start_time,
            duration_ms,
            completion_time: start_time + (duration_ms / 1000),
            status: ActionStatusEnum::Pending,
        },
        specific_data,
    };

    match action_processor
        .add_action_from_rpc(&db_tables.actions, &action_data, ActionTypeEnum::TrainUnit)
        .await
    {
        Ok(action_id) => {
            tracing::info!(
                "Training unit {} to {:?} (action {}) [via lightyear]",
                unit_id, target_profession, action_id
            );
            bridge_sender.send(BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status: ActionStatusEnum::Pending,
                action_type: ActionTypeEnum::TrainUnit,
                completion_time: start_time + (duration_ms / 1000),
                action_name: None,
                unit_ids: vec![],
            });
        }
        Err(e) => {
            tracing::error!("Failed to schedule training: {}", e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to schedule training: {}", e),
            });
        }
    }
}

// ─── Explore ─────────────────────────────────────────────────────────

async fn handle_explore(
    player_id: u64,
    cell: GridCell,
    radius: i32,
    db_tables: &DatabaseTables,
    grid_config: &shared::grid::GridConfig,
    world_global_state: &WorldGlobalState,
    sessions: &Sessions,
) {
    use shared::protocol::ServerMessage;

    // player_id is u64 but exploration DB uses i64
    let player_id_i64 = player_id as i64;

    let n_chunk_x = world_global_state.n_chunk_x;
    let n_chunk_y = world_global_state.n_chunk_y;
    let chunk_w = shared::constants::CHUNK_SIZE.x;
    let chunk_h = shared::constants::CHUNK_SIZE.y;
    let layout = &grid_config.layout;
    let world_seed = crate::exploration::EXPLORATION_WORLD_SEED;

    // Compute zone IDs in radius — pure deterministic, no DB
    let zone_ids = crate::exploration::zones_in_radius(&cell, radius, layout, world_seed);

    match db_tables
        .exploration_voronoi
        .mark_explored(&zone_ids, player_id_i64)
        .await
    {
        Ok(newly_explored) => {
            if !newly_explored.is_empty() {
                tracing::info!(
                    "Player {} explored {} new voronoi zones around ({},{}) [via lightyear]",
                    player_id, newly_explored.len(), cell.q, cell.r
                );

                // Compute all seeds for rasterization
                let all_seeds = crate::exploration::compute_all_seeds(
                    n_chunk_x, n_chunk_y, layout, world_seed,
                );

                // Get seeds for newly explored zones to compute patch bounds
                let new_seeds: Vec<_> = all_seeds
                    .iter()
                    .filter(|s| newly_explored.contains(&s.zone_id))
                    .cloned()
                    .collect();

                let explored_set = db_tables
                    .exploration_voronoi
                    .load_explored_set()
                    .await
                    .unwrap_or_default();

                let (px, py, pw, ph) = crate::exploration::compute_patch_bounds(
                    &new_seeds, n_chunk_x, n_chunk_y, chunk_w, chunk_h, 5,
                );

                let patch_data = crate::exploration::rasterize_patch(
                    &all_seeds,
                    &explored_set,
                    n_chunk_x,
                    n_chunk_y,
                    chunk_w,
                    chunk_h,
                    px,
                    py,
                    pw,
                    ph,
                );

                let update = ServerMessage::ExplorationPatch {
                    patch_x: px,
                    patch_y: py,
                    patch_width: pw,
                    patch_height: ph,
                    patch_data,
                };
                sessions.broadcast(update).await;
            }
        }
        Err(e) => {
            tracing::error!("Failed to mark voronoi exploration: {}", e);
        }
    }
}
