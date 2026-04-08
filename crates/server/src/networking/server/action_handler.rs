use std::sync::Arc;

use crate::action_processor::ActionProcessor;
use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;
use crate::units::PortraitGenerator;
use crate::world;
use crate::world::resources::WorldGlobalState;
use shared::grid::GridCell;
use shared::protocol::ColorData;
use shared::{
    ActionStatusEnum, ActionTypeEnum, BuildingTypeEnum, ContourSegmentData, GameState,
    OrganizationType, ProfessionEnum, ResourceSpecificTypeEnum, SlotPosition, TerrainChunkId,
};
use sqlx::Row;

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
                        &bridge_sender,
                    )
                    .await;
                }
                ActionRequest::LoadPlayerData { player_id } => {
                    handle_load_player_data(
                        player_id,
                        &bridge_sender,
                        &db_tables,
                        &dev_config,
                        &game_state,
                        &grid_config,
                    )
                    .await;
                }
                ActionRequest::LoadInventory { player_id, unit_id } => {
                    handle_load_inventory(player_id, unit_id, &bridge_sender, &db_tables).await;
                }
                ActionRequest::LoadOrganizationAtCell { player_id, cell } => {
                    handle_load_organization_at_cell(player_id, cell, &bridge_sender, &db_tables)
                        .await;
                }

                // ── Remaining commands migrated from tungstenite (#138) ──
                ActionRequest::CreateLord {
                    player_id,
                    first_name,
                    gender,
                    portrait_layers,
                } => {
                    handle_create_lord(
                        player_id,
                        &first_name,
                        &gender,
                        &portrait_layers,
                        &bridge_sender,
                        &db_tables,
                    )
                    .await;
                }
                ActionRequest::FoundHamlet { player_id } => {
                    handle_found_hamlet(
                        player_id,
                        &bridge_sender,
                        &db_tables,
                        &grid_config,
                        &world_global_state,
                        &dev_config,
                    )
                    .await;
                }
                ActionRequest::MoveUnitToSlot {
                    player_id,
                    unit_id,
                    cell,
                    from_slot,
                    to_slot,
                } => {
                    handle_move_unit_to_slot(
                        player_id,
                        unit_id,
                        cell,
                        from_slot,
                        to_slot,
                        &bridge_sender,
                        &db_tables,
                    )
                    .await;
                }
                ActionRequest::AssignUnitToSlot {
                    player_id,
                    unit_id,
                    cell,
                    slot,
                } => {
                    handle_assign_unit_to_slot(
                        player_id,
                        unit_id,
                        cell,
                        slot,
                        &bridge_sender,
                        &db_tables,
                    )
                    .await;
                }
                ActionRequest::DebugCreateOrganization {
                    player_id,
                    name,
                    organization_type,
                    cell,
                    parent_organization_id,
                } => {
                    handle_debug_create_organization(
                        player_id,
                        &name,
                        organization_type,
                        cell,
                        parent_organization_id,
                        &bridge_sender,
                        &db_tables,
                        &grid_config,
                        &world_global_state,
                    )
                    .await;
                }
                ActionRequest::DebugDeleteOrganization {
                    player_id,
                    organization_id,
                } => {
                    handle_debug_delete_organization(
                        player_id,
                        organization_id,
                        &bridge_sender,
                        &db_tables,
                    )
                    .await;
                }
                ActionRequest::DebugSpawnUnit { player_id, cell } => {
                    handle_debug_spawn_unit(player_id, cell, &bridge_sender, &db_tables).await;
                }
                ActionRequest::DestroyBuilding { player_id, cell } => {
                    handle_destroy_building(player_id, cell, &bridge_sender, &db_tables).await;
                }
                ActionRequest::LiquidateOrganization { player_id } => {
                    handle_liquidate_organization(player_id, &bridge_sender, &db_tables).await;
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
        player_id,
        building_type,
        building_specific_type,
        chunk_id.x,
        chunk_id.y,
        cell.q,
        cell.r
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
        .add_action_from_rpc(
            &db_tables.actions,
            &action_data,
            ActionTypeEnum::BuildBuilding,
        )
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
        player_id,
        start_cell.q,
        start_cell.r,
        end_cell.q,
        end_cell.r
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
            tracing::info!("Scheduled build road action {} [via lightyear]", action_id);
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
                    tracing::error!("Failed to assign units to action {}: {}", action_id, e);
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
                player_id,
                recipe_id
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

    let duration_ms =
        dev_config.apply_speed((recipe.craft_duration_seconds as u64) * 1000 * (quantity as u64));

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
                    tracing::error!("Failed to assign units to action {}: {}", action_id, e);
                }
            }

            tracing::info!(
                "Craft action {} scheduled: recipe '{}' x{} for player {} (duration: {}s) [via lightyear]",
                action_id,
                recipe.name,
                quantity,
                player_id,
                duration_ms / 1000
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
                unit_id,
                target_profession,
                action_id
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

// ─── LoadInventory ────────────────────────────────────────────────────

async fn handle_load_inventory(
    player_id: u64,
    unit_id: u64,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    use shared::protocol::InventoryItemData;

    match db_tables.resources.load_items_for_unit(unit_id).await {
        Ok(full_items) => {
            // Group by item_id for display
            let mut grouped: std::collections::HashMap<
                i32,
                (String, shared::ItemTypeEnum, f32, f32, i32),
            > = std::collections::HashMap::new();

            for item in &full_items {
                let entry = grouped.entry(item.definition.id).or_insert((
                    item.definition.name.clone(),
                    item.definition.item_type,
                    item.definition.weight_kg,
                    item.instance.quality,
                    0,
                ));
                entry.4 += 1;
            }

            let items: Vec<InventoryItemData> = grouped
                .into_iter()
                .map(
                    |(item_id, (name, item_type, weight, quality, qty))| InventoryItemData {
                        instance_id: 0,
                        item_id,
                        name,
                        item_type,
                        quality,
                        weight_kg: weight,
                        quantity: qty,
                        is_equipped: false,
                        equipment_slot: None,
                    },
                )
                .collect();

            bridge_sender.send(BridgeEvent::SendInventoryData {
                player_id,
                unit_id,
                items,
            });
        }
        Err(e) => {
            tracing::error!("Failed to load inventory for unit {}: {}", unit_id, e);
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to load inventory: {}", e),
            });
        }
    }
}

// ─── LoadOrganizationAtCell ──────────────────────────────────────────

async fn handle_load_organization_at_cell(
    player_id: u64,
    cell: GridCell,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    // Query to find which organization owns this cell
    match sqlx::query_as::<_, (i64,)>(
        "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
    )
    .bind(cell.q)
    .bind(cell.r)
    .fetch_optional(&db_tables.pool)
    .await
    {
        Ok(Some((org_id,))) => {
            // Load organization summary
            match db_tables.organizations.load_organization(org_id as u64).await {
                Ok(org_data) => {
                    let summary = shared::OrganizationSummary {
                        id: org_data.id,
                        name: org_data.name,
                        organization_type: org_data.organization_type,
                        leader_unit_id: org_data.leader_unit_id,
                        population: org_data.population,
                        named_unit_count: 0,
                        population_capacity: 0,
                        emblem_url: org_data.emblem_url,
                    };
                    bridge_sender.send(BridgeEvent::SendOrganizationAtCell {
                        player_id,
                        cell,
                        organization: Some(summary),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to load organization: {}", e);
                    bridge_sender.send(BridgeEvent::SendOrganizationAtCell {
                        player_id,
                        cell,
                        organization: None,
                    });
                }
            }
        }
        Ok(None) => {
            bridge_sender.send(BridgeEvent::SendOrganizationAtCell {
                player_id,
                cell,
                organization: None,
            });
        }
        Err(e) => {
            tracing::error!("Database error checking organization: {}", e);
            bridge_sender.send(BridgeEvent::SendOrganizationAtCell {
                player_id,
                cell,
                organization: None,
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
    bridge_sender: &BridgeSender,
) {
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
                    player_id,
                    newly_explored.len(),
                    cell.q,
                    cell.r
                );

                // Compute all seeds for rasterization
                let all_seeds =
                    crate::exploration::compute_all_seeds(n_chunk_x, n_chunk_y, layout, world_seed);

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

                let compressed_patch = shared::protocol::bulk_compress::compress(&patch_data);
                bridge_sender.send(BridgeEvent::SendExplorationPatch {
                    player_id,
                    patch_x: px,
                    patch_y: py,
                    patch_width: pw,
                    patch_height: ph,
                    compressed_data: compressed_patch,
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to mark voronoi exploration: {}", e);
        }
    }
}

// ─── LoadPlayerData (#141 Step 4) ────────────────────────────────────

async fn handle_load_player_data(
    player_id: u64,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    dev_config: &DevConfig,
    game_state: &GameState,
    grid_config: &shared::grid::GridConfig,
) {
    let player_id_i64 = player_id as i64;

    let player = match shared::types::game::methods::get_player_by_id(
        &db_tables.pool,
        player_id_i64,
    )
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            tracing::error!("LoadPlayerData: player {} not found", player_id);
            return;
        }
        Err(e) => {
            tracing::error!("LoadPlayerData: DB error for player {}: {}", player_id, e);
            return;
        }
    };

    let player_data = shared::protocol::PlayerData {
        id: player.id,
        family_name: player.family_name.clone(),
        language_id: player.language_id,
        coat_of_arms_id: player.coat_of_arms_id,
        motto: player.motto.clone(),
        origin_location: player.origin_location.clone(),
    };

    let characters =
        shared::types::game::methods::get_player_characters(&db_tables.pool, player_id_i64)
            .await
            .unwrap_or_default();

    let character_data = characters
        .into_iter()
        .next()
        .map(|c| shared::protocol::CharacterData {
            id: c.id,
            player_id: c.player_id,
            first_name: c.first_name,
            family_name: c.family_name,
            second_name: c.second_name,
            nickname: c.nickname,
            coat_of_arms_id: c.coat_of_arms_id,
            image_id: c.image_id,
            motto: c.motto,
        });

    let lord = db_tables
        .units
        .load_lord_for_player(player_id)
        .await
        .unwrap_or(None);

    crate::exploration::ensure_spawn_explored(lord.clone(), db_tables, player_id_i64, grid_config)
        .await;

    if let Some(ref lord_data) = lord {
        bridge_sender.send(BridgeEvent::SpawnLord {
            player_id,
            chunk: lord_data.current_chunk,
            cell: lord_data.current_cell,
        });
    }

    let organization = if let Some(ref lord_data) = lord {
        match sqlx::query_as::<_, (i64, String, i16, Option<i64>, i32, Option<String>)>(
            "SELECT o.id, o.name, o.organization_type_id, o.leader_unit_id, o.population, o.emblem_url FROM organizations.organizations o WHERE o.leader_unit_id = $1 LIMIT 1",
        ).bind(lord_data.id as i64).fetch_optional(&db_tables.pool).await {
            Ok(Some((id, name, type_id, leader_id, pop, emblem))) => {
                let org_id = id;
                let named_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM units.units u INNER JOIN organizations.territory_cells tc ON u.current_cell_q = tc.cell_q AND u.current_cell_r = tc.cell_r WHERE tc.organization_id = $1 AND u.is_lord = false"
                ).bind(org_id).fetch_one(&db_tables.pool).await.unwrap_or(0);

                let building_rows = sqlx::query(
                    "SELECT building_type_id FROM buildings.buildings_base b INNER JOIN organizations.territory_cells tc ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r WHERE tc.organization_id = $1 AND b.is_built = true"
                ).bind(org_id).fetch_all(&db_tables.pool).await.unwrap_or_default();

                let pop_capacity: i32 = building_rows.iter()
                    .filter_map(|r| {
                        let type_id: i32 = r.get("building_type_id");
                        shared::BuildingTypeEnum::from_id(type_id as i16)
                            .map(|bt| bt.housing_capacity() as i32)
                    })
                    .sum();

                Some(shared::OrganizationSummary {
                    id: id as u64, name,
                    organization_type: shared::OrganizationType::from_id(type_id),
                    leader_unit_id: leader_id.map(|l| l as u64),
                    population: pop,
                    named_unit_count: named_count as i32,
                    population_capacity: pop_capacity,
                    emblem_url: emblem,
                })
            },
            _ => None,
        }
    } else {
        None
    };

    let game_data = crate::game_data::build_game_data_payload(game_state, dev_config);

    bridge_sender.send(BridgeEvent::SendLoginData {
        player_id,
        player: player_data,
        character: character_data,
        lord,
        organization,
        game_data,
    });

    tracing::info!("📦 LoadPlayerData complete for player {}", player_id);
}

// ─── Remaining command handlers (#138) ──────────────────────────────

async fn handle_create_lord(
    player_id: u64,
    first_name: &str,
    gender: &str,
    portrait_layers: &str,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    tracing::info!(
        "Player {} creating lord: {} ({})",
        player_id,
        first_name,
        gender
    );

    // Check no existing lord
    match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(_)) => {
            bridge_sender.send(BridgeEvent::SendLordCreateError {
                player_id,
                reason: "Vous avez déjà un Lord/Lady".to_string(),
            });
            return;
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendLordCreateError {
                player_id,
                reason: format!("Erreur: {}", e),
            });
            return;
        }
        Ok(None) => {}
    }

    let family_name =
        match shared::types::game::methods::get_player_by_id(&db_tables.pool, player_id as i64)
            .await
        {
            Ok(Some(p)) => p.family_name,
            _ => {
                bridge_sender.send(BridgeEvent::SendLordCreateError {
                    player_id,
                    reason: "Joueur introuvable".to_string(),
                });
                return;
            }
        };

    let starting_cell = GridCell { q: 0, r: 0 };
    let starting_chunk = TerrainChunkId { x: 0, y: 0 };
    let avatar_url = format!("lord_{}_{}", gender, player_id);

    match db_tables
        .units
        .create_unit(
            Some(player_id),
            first_name.to_string(),
            family_name.clone(),
            gender.to_string(),
            "lord".to_string(),
            avatar_url,
            starting_cell,
            starting_chunk,
            ProfessionEnum::Unknown,
            true,
            Some(portrait_layers.to_string()),
        )
        .await
    {
        Ok(unit_id) => {
            let _ = shared::types::game::methods::create_character(
                &db_tables.pool,
                player_id as i64,
                first_name,
                &family_name,
                None,
                None,
                None,
            )
            .await;
            match db_tables.units.load_unit(unit_id).await {
                Ok(unit_data) => {
                    bridge_sender.send(BridgeEvent::SendLordCreated {
                        player_id,
                        unit_data,
                    });
                }
                Err(e) => {
                    bridge_sender.send(BridgeEvent::SendLordCreateError {
                        player_id,
                        reason: format!("Lord créé mais erreur au chargement: {}", e),
                    });
                }
            }
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendLordCreateError {
                player_id,
                reason: format!("Erreur lors de la création: {}", e),
            });
        }
    }
}

/// Duration for founding a hamlet (surveying + camp setup), in seconds.
const FOUND_HAMLET_DURATION_SECS: u64 = 30;

async fn handle_found_hamlet(
    player_id: u64,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    grid_config: &shared::grid::GridConfig,
    world_global_state: &WorldGlobalState,
    dev_config: &DevConfig,
) {
    tracing::info!("Player {} requesting to found a hamlet", player_id);

    let lord = match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(lord)) => lord,
        Ok(None) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id,
                reason: "Vous n'avez pas de Lord/Lady".to_string(),
            });
            return;
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id,
                reason: format!("Erreur: {}", e),
            });
            return;
        }
    };

    let cell = lord.current_cell;
    let chunk = cell.to_chunk_id(&grid_config.layout);

    // Check cell not already in a territory
    match sqlx::query_scalar::<_, i64>(
        "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
    ).bind(cell.q).bind(cell.r).fetch_optional(&db_tables.pool).await {
        Ok(Some(_)) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id, reason: "Cette cellule appartient déjà à un territoire".to_string(),
            });
            return;
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id, reason: format!("Erreur DB: {}", e),
            });
            return;
        }
        Ok(None) => {}
    }

    // Check player doesn't already have an org
    match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM organizations.organizations WHERE leader_unit_id = $1",
    )
    .bind(lord.id as i64)
    .fetch_optional(&db_tables.pool)
    .await
    {
        Ok(Some(id)) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id,
                reason: format!("Vous avez déjà une organisation (ID: {})", id),
            });
            return;
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id,
                reason: format!("Erreur DB: {}", e),
            });
            return;
        }
        Ok(None) => {}
    }

    // Validation passed — send Pending status to client
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let duration_ms = dev_config.apply_speed(FOUND_HAMLET_DURATION_SECS * 1000);
    let duration_secs = duration_ms / 1000;
    let completion_time = now + duration_secs;

    // Use a synthetic action_id (negative to avoid collision with real actions)
    let action_id = now; // unique enough for display purposes

    bridge_sender.send(BridgeEvent::SendActionStatus {
        player_id,
        action_id,
        chunk_id: chunk,
        cell,
        status: ActionStatusEnum::InProgress,
        action_type: ActionTypeEnum::FoundHamlet,
        completion_time,
        action_name: Some("Fondation du hameau".to_string()),
        unit_ids: vec![lord.id],
    });

    tracing::info!(
        "⏳ FoundHamlet action started for player {} — {}ms duration (dev speed applied)",
        player_id, duration_ms
    );

    // Wait for the duration, then execute the founding
    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;

    tracing::info!("✓ FoundHamlet duration elapsed for player {}, executing...", player_id);

    let family_name =
        match shared::types::game::methods::get_player_by_id(&db_tables.pool, player_id as i64)
            .await
        {
            Ok(Some(p)) => p.family_name,
            _ => "Inconnu".to_string(),
        };

    let hamlet_name = format!("Hameau de {}", family_name);

    let request = shared::CreateOrganizationRequest {
        name: hamlet_name.clone(),
        organization_type: OrganizationType::Hamlet,
        headquarters_cell: Some(cell),
        parent_organization_id: None,
        founder_unit_id: lord.id,
    };

    let org_id = match db_tables.organizations.create_organization(request).await {
        Ok(id) => id,
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendHamletFoundError {
                player_id,
                reason: format!("Échec de la création: {}", e),
            });
            return;
        }
    };

    // Auto-place base camp on lord's cell
    let camp_type = BuildingTypeEnum::Campement;
    let camp_id: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM buildings.buildings_base",
    )
    .fetch_one(&db_tables.pool)
    .await
    .unwrap_or(1);

    let chunk = cell.to_chunk_id(&grid_config.layout);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let camp_data = shared::BuildingData {
        base_data: shared::BuildingBaseData {
            id: camp_id as u64,
            specific_type: shared::BuildingSpecificTypeEnum::Unknown,
            category: shared::BuildingCategoryEnum::Residential,
            building_type_id: camp_type.to_id(),
            cell,
            chunk,
            quality: 1.0,
            durability: 1.0,
            damage: 0.0,
            created_at: now,
        },
        specific_data: shared::BuildingSpecific::Unknown(),
    };

    if let Err(e) = db_tables.buildings.create_building(&camp_data).await {
        tracing::warn!("Failed to create base camp: {}", e);
    } else {
        let _ = db_tables.buildings.mark_building_as_built(camp_id as u64).await;
        tracing::info!("⛺ Base camp placed at ({},{}) for org {}", cell.q, cell.r, org_id);
    }

    // Initialize population to base camp housing capacity
    let initial_pop = camp_type.housing_capacity() as i32;
    let _ = sqlx::query("UPDATE organizations.organizations SET population = $1 WHERE id = $2")
        .bind(initial_pop)
        .bind(org_id as i64)
        .execute(&db_tables.pool)
        .await;

    // Claim territory via Voronoi zones
    let mut claimed_cells = Vec::new();
    match db_tables.voronoi_zones.get_zone_at_cell(cell).await {
        Ok(Some(zone_id)) => {
            match db_tables.voronoi_zones.is_zone_available(zone_id).await {
                Ok(true) => {
                    if let Ok(zone_cells) = db_tables.voronoi_zones.compute_zone_cells(zone_id).await {
                        for zone_cell in &zone_cells {
                            if db_tables
                                .organizations
                                .add_territory_cell(org_id, zone_cell)
                                .await
                                .is_ok()
                            {
                                claimed_cells.push(*zone_cell);
                            }
                        }
                        let _ = sqlx::query("UPDATE organizations.organizations SET voronoi_zone_id = $1 WHERE id = $2")
                            .bind(zone_id).bind(org_id as i64).execute(&db_tables.pool).await;
                    } else {
                        crate::world::territory::claim_cell_and_neighbors(
                            db_tables,
                            org_id,
                            &cell,
                            &mut claimed_cells,
                        )
                        .await;
                    }
                }
                _ => {
                    crate::world::territory::claim_cell_and_neighbors(
                        db_tables,
                        org_id,
                        &cell,
                        &mut claimed_cells,
                    )
                    .await;
                }
            }
        }
        _ => {
            crate::world::territory::claim_cell_and_neighbors(
                db_tables,
                org_id,
                &cell,
                &mut claimed_cells,
            )
            .await;
        }
    }

    // Generate territory contours
    if let Ok(territory_cells) = db_tables.organizations.load_territory_cells(org_id).await
        && !territory_cells.is_empty()
    {
        tracing::info!(
            "Building contour for org {} from {} territory cells. Sample: {:?}",
            org_id,
            territory_cells.len(),
            &territory_cells[..territory_cells.len().min(5)]
        );

        use hexx::Hex;
        let territory_hex: std::collections::HashSet<Hex> =
            territory_cells.iter().map(|c| c.to_hex()).collect();
        let contour_points = &world::territory::build_contour(
            &grid_config.layout,
            &territory_hex,
            0.0,
            org_id as u64,
        );
        let contour_chunks = crate::utils::chunks::split_contour_into_chunks(contour_points);

        tracing::info!(
            "Contour generated: {} points, split into {} chunks",
            contour_points.len(),
            contour_chunks.len()
        );
        for (chunk_id, contour_segments) in &contour_chunks {
            let _ = db_tables
                .territory_contours
                .store_contour(org_id, chunk_id.x, chunk_id.y, contour_segments)
                .await;
            let (border_color, fill_color) = world::territory::generate_org_colors(org_id);
            bridge_sender.send(BridgeEvent::BroadcastTerritoryContourUpdate {
                chunk_id: *chunk_id,
                contours: vec![shared::protocol::TerritoryContourChunkData {
                    organization_id: org_id,
                    chunk_id: *chunk_id,
                    segments: contour_segments
                        .iter()
                        .map(ContourSegmentData::from_contour_segment)
                        .collect(),
                    border_color: ColorData::from_array(border_color),
                    fill_color: ColorData::from_array(fill_color),
                }],
            });
        }
    }

    bridge_sender.send(BridgeEvent::SendHamletFounded {
        player_id,
        organization_id: org_id,
        name: hamlet_name,
        headquarters: cell,
        territory_cells: claimed_cells,
    });

    // Send population stats so client shows correct capacity
    let pop_capacity = calculate_housing_capacity_for_org(db_tables, org_id).await;
    let initial_pop = BuildingTypeEnum::Campement.housing_capacity() as i32;
    bridge_sender.send(BridgeEvent::SendPopulationChanged {
        player_id,
        organization_id: org_id,
        new_population: initial_pop,
        named_unit_count: 0,
        population_capacity: pop_capacity as i32,
        immigrant: None,
    });

    // Mark action as completed
    bridge_sender.send(BridgeEvent::SendActionStatus {
        player_id,
        action_id,
        chunk_id: chunk,
        cell,
        status: ActionStatusEnum::Completed,
        action_type: ActionTypeEnum::FoundHamlet,
        completion_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        action_name: Some("Fondation du hameau".to_string()),
        unit_ids: vec![lord.id],
    });

    bridge_sender.send(BridgeEvent::BroadcastActionCompleted {
        action_id,
        chunk_id: chunk,
        cell,
        action_type: ActionTypeEnum::FoundHamlet,
    });
}

async fn handle_move_unit_to_slot(
    player_id: u64,
    unit_id: u64,
    cell: GridCell,
    _from_slot: SlotPosition,
    to_slot: SlotPosition,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    let slot_type_str = match to_slot.slot_type {
        shared::SlotType::Interior => "interior",
        shared::SlotType::Exterior => "exterior",
    };
    match db_tables
        .units
        .update_slot_position(
            unit_id,
            Some(slot_type_str.to_string()),
            Some(to_slot.index as i32),
        )
        .await
    {
        Ok(_) => {
            bridge_sender.send(BridgeEvent::SendUnitSlotUpdated {
                player_id,
                unit_id,
                cell,
                slot_position: Some(to_slot),
            });
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to update slot position: {}", e),
            });
        }
    }
}

async fn handle_assign_unit_to_slot(
    player_id: u64,
    unit_id: u64,
    cell: GridCell,
    slot: SlotPosition,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    let slot_type_str = match slot.slot_type {
        shared::SlotType::Interior => "interior",
        shared::SlotType::Exterior => "exterior",
    };
    match db_tables
        .units
        .update_slot_position(
            unit_id,
            Some(slot_type_str.to_string()),
            Some(slot.index as i32),
        )
        .await
    {
        Ok(_) => {
            bridge_sender.send(BridgeEvent::SendUnitSlotUpdated {
                player_id,
                unit_id,
                cell,
                slot_position: Some(slot),
            });
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: format!("Failed to assign slot position: {}", e),
            });
        }
    }
}

async fn handle_debug_create_organization(
    player_id: u64,
    name: &str,
    organization_type: OrganizationType,
    cell: GridCell,
    parent_organization_id: Option<u64>,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
    grid_config: &shared::grid::GridConfig,
    world_global_state: &WorldGlobalState,
) {
    tracing::info!(
        "DEBUG: Creating org '{}' type {:?} at {:?}",
        name,
        organization_type,
        cell
    );

    // Create leader unit
    let (first, last) = (format!("Leader_{}", name), "Debug".to_string());
    let (variant_id, avatar_url) =
        PortraitGenerator::generate_variant_and_url("male", ProfessionEnum::Merchant);

    let founder_unit_id = match db_tables
        .units
        .create_unit(
            None,
            first,
            last,
            "male".to_string(),
            variant_id,
            avatar_url,
            cell,
            TerrainChunkId { x: 0, y: 0 },
            ProfessionEnum::Merchant,
            false,
            None,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendDebugError {
                player_id,
                reason: format!("Failed to create leader unit: {}", e),
            });
            return;
        }
    };

    let request = shared::CreateOrganizationRequest {
        name: name.to_string(),
        organization_type,
        headquarters_cell: Some(cell),
        parent_organization_id,
        founder_unit_id,
    };

    match db_tables.organizations.create_organization(request).await {
        Ok(org_id) => {
            // Claim Voronoi zone territory
            let mut claimed_cells = Vec::new();
            if let Ok(Some(zone_id)) = db_tables.voronoi_zones.get_zone_at_cell(cell).await {
                if let Ok(true) = db_tables.voronoi_zones.is_zone_available(zone_id).await {
                    if let Ok(zone_cells) = db_tables.voronoi_zones.compute_zone_cells(zone_id).await {
                        for zc in &zone_cells {
                            if db_tables
                                .organizations
                                .add_territory_cell(org_id, zc)
                                .await
                                .is_ok()
                            {
                                claimed_cells.push(*zc);
                            }
                        }
                        let _ = sqlx::query("UPDATE organizations.organizations SET voronoi_zone_id = $1 WHERE id = $2")
                            .bind(zone_id).bind(org_id as i64).execute(&db_tables.pool).await;
                    }
                }
            }
            if claimed_cells.is_empty() {
                crate::world::territory::claim_cell_and_neighbors(
                    db_tables,
                    org_id,
                    &cell,
                    &mut claimed_cells,
                )
                .await;
            }

            // Generate contours
            if let Ok(territory_cells) = db_tables.organizations.load_territory_cells(org_id).await
            {
                if !territory_cells.is_empty() {
                    use hexx::Hex;
                    let territory_hex: std::collections::HashSet<Hex> =
                        territory_cells.iter().map(|c| c.to_hex()).collect();
                    let contour_points = &world::territory::build_contour(
                        &grid_config.layout,
                        &territory_hex,
                        0.0,
                        org_id as u64,
                    );
                    let contour_chunks =
                        crate::utils::chunks::split_contour_into_chunks(contour_points);
                    for (chunk_id, segs) in &contour_chunks {
                        let _ = db_tables
                            .territory_contours
                            .store_contour(org_id, chunk_id.x, chunk_id.y, segs)
                            .await;
                        let (bc, fc) = world::territory::generate_org_colors(org_id);
                        bridge_sender.send(BridgeEvent::BroadcastTerritoryContourUpdate {
                            chunk_id: *chunk_id,
                            contours: vec![shared::protocol::TerritoryContourChunkData {
                                organization_id: org_id,
                                chunk_id: *chunk_id,
                                segments: segs
                                    .iter()
                                    .map(ContourSegmentData::from_contour_segment)
                                    .collect(),
                                border_color: ColorData::from_array(bc),
                                fill_color: ColorData::from_array(fc),
                            }],
                        });
                    }
                }
            }

            bridge_sender.send(BridgeEvent::SendDebugOrganizationCreated {
                player_id,
                organization_id: org_id,
                name: name.to_string(),
            });
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendDebugError {
                player_id,
                reason: format!("Failed to create org: {}", e),
            });
        }
    }
}

async fn handle_debug_delete_organization(
    player_id: u64,
    organization_id: u64,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    tracing::info!("DEBUG: Deleting organization {}", organization_id);
    match sqlx::query("DELETE FROM organizations.organizations WHERE id = $1")
        .bind(organization_id as i64)
        .execute(&db_tables.pool)
        .await
    {
        Ok(_) => {
            bridge_sender.send(BridgeEvent::SendDebugOrganizationDeleted {
                player_id,
                organization_id,
            });
        }
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendDebugError {
                player_id,
                reason: format!("Failed to delete org: {}", e),
            });
        }
    }
}

async fn handle_debug_spawn_unit(
    player_id: u64,
    cell: GridCell,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    tracing::info!("DEBUG: Spawning unit at {:?}", cell);
    let (variant_id, avatar_url) =
        PortraitGenerator::generate_variant_and_url("male", ProfessionEnum::Settler);
    match db_tables
        .units
        .create_unit(
            Some(player_id),
            "Debug".to_string(),
            "Unit".to_string(),
            "male".to_string(),
            variant_id,
            avatar_url,
            cell,
            TerrainChunkId { x: 0, y: 0 },
            ProfessionEnum::Settler,
            false,
            None,
        )
        .await
    {
        Ok(unit_id) => match db_tables.units.load_unit(unit_id).await {
            Ok(unit_data) => {
                bridge_sender.send(BridgeEvent::SendDebugUnitSpawned {
                    player_id,
                    unit_data,
                });
            }
            Err(e) => {
                bridge_sender.send(BridgeEvent::SendDebugError {
                    player_id,
                    reason: format!("Unit created but failed to load: {}", e),
                });
            }
        },
        Err(e) => {
            bridge_sender.send(BridgeEvent::SendDebugError {
                player_id,
                reason: format!("Failed to spawn unit: {}", e),
            });
        }
    }
}

// ─── Destroy building / Liquidate organization (#216) ───────────────

async fn calculate_housing_capacity_for_org(db_tables: &DatabaseTables, org_id: u64) -> u32 {
    let rows = sqlx::query(
        r#"SELECT b.building_type_id FROM buildings.buildings_base b
           INNER JOIN organizations.territory_cells tc
               ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
           WHERE tc.organization_id = $1 AND b.is_built = true"#,
    )
    .bind(org_id as i64)
    .fetch_all(&db_tables.pool)
    .await
    .unwrap_or_default();

    rows.iter()
        .filter_map(|r| {
            let id: i32 = r.get("building_type_id");
            BuildingTypeEnum::from_id(id as i16).map(|bt| bt.housing_capacity())
        })
        .sum()
}

async fn handle_destroy_building(
    player_id: u64,
    cell: GridCell,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    // 1. Get building ID at cell
    let building_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM buildings.buildings_base WHERE cell_q = $1 AND cell_r = $2",
    )
    .bind(cell.q)
    .bind(cell.r)
    .fetch_optional(&db_tables.pool)
    .await
    {
        Ok(Some(id)) => id as u64,
        _ => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Aucun bâtiment sur cette cellule".to_string(),
            });
            return;
        }
    };

    // 2. Verify player owns this territory
    let lord = match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(lord)) => lord,
        _ => return,
    };

    let org_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM organizations.organizations WHERE leader_unit_id = $1",
    )
    .bind(lord.id as i64)
    .fetch_optional(&db_tables.pool)
    .await
    {
        Ok(Some(id)) => id as u64,
        _ => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Pas d'organisation".to_string(),
            });
            return;
        }
    };

    let in_territory = sqlx::query_scalar::<_, i64>(
        "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2",
    )
    .bind(cell.q)
    .bind(cell.r)
    .fetch_optional(&db_tables.pool)
    .await
    .ok()
    .flatten();

    if in_territory != Some(org_id as i64) {
        bridge_sender.send(BridgeEvent::SendActionError {
            player_id,
            reason: "Bâtiment hors de votre territoire".to_string(),
        });
        return;
    }

    // 3. Free units on this cell
    let _ = sqlx::query(
        "UPDATE units.units SET slot_type = NULL, slot_index = NULL \
         WHERE current_cell_q = $1 AND current_cell_r = $2 AND is_lord = false",
    )
    .bind(cell.q)
    .bind(cell.r)
    .execute(&db_tables.pool)
    .await;

    // 4. Delete building
    if let Err(e) = db_tables.buildings.delete_building(building_id).await {
        bridge_sender.send(BridgeEvent::SendActionError {
            player_id,
            reason: format!("Échec: {}", e),
        });
        return;
    }

    // 5. Recalculate capacity and clamp population
    let capacity = calculate_housing_capacity_for_org(db_tables, org_id).await;
    let current_pop: i32 = sqlx::query_scalar(
        "SELECT population FROM organizations.organizations WHERE id = $1",
    )
    .bind(org_id as i64)
    .fetch_one(&db_tables.pool)
    .await
    .unwrap_or(0);

    let new_pop = current_pop.min(capacity as i32);
    if new_pop != current_pop {
        let _ = sqlx::query(
            "UPDATE organizations.organizations SET population = $1 WHERE id = $2",
        )
        .bind(new_pop)
        .bind(org_id as i64)
        .execute(&db_tables.pool)
        .await;
    }

    tracing::info!(
        "🗑️ Building {} destroyed at ({},{}) by player {}",
        building_id, cell.q, cell.r, player_id
    );

    bridge_sender.send(BridgeEvent::SendBuildingDestroyed {
        player_id,
        cell,
        building_id,
        new_population: new_pop,
        population_capacity: capacity as i32,
    });
}

async fn handle_liquidate_organization(
    player_id: u64,
    bridge_sender: &BridgeSender,
    db_tables: &DatabaseTables,
) {
    let lord = match db_tables.units.load_lord_for_player(player_id).await {
        Ok(Some(lord)) => lord,
        _ => return,
    };

    let org_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM organizations.organizations WHERE leader_unit_id = $1",
    )
    .bind(lord.id as i64)
    .fetch_optional(&db_tables.pool)
    .await
    {
        Ok(Some(id)) => id as u64,
        _ => {
            bridge_sender.send(BridgeEvent::SendActionError {
                player_id,
                reason: "Pas d'organisation à dissoudre".to_string(),
            });
            return;
        }
    };

    tracing::info!(
        "🏚️ Player {} liquidating organization {}",
        player_id, org_id
    );

    // Collect affected chunks BEFORE deleting territory
    let affected_chunks: Vec<TerrainChunkId> = sqlx::query(
        "SELECT DISTINCT b.chunk_x, b.chunk_y FROM buildings.buildings_base b \
         INNER JOIN organizations.territory_cells tc \
             ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r \
         WHERE tc.organization_id = $1",
    )
    .bind(org_id as i64)
    .fetch_all(&db_tables.pool)
    .await
    .unwrap_or_default()
    .iter()
    .map(|r| TerrainChunkId {
        x: r.get("chunk_x"),
        y: r.get("chunk_y"),
    })
    .collect();

    // Order matters: delete buildings first (needs territory_cells), then clean up

    // 1. Delete player-built buildings in territory (keep trees)
    let _ = sqlx::query(
        r#"DELETE FROM buildings.buildings_base
           WHERE id IN (
               SELECT b.id FROM buildings.buildings_base b
               INNER JOIN organizations.territory_cells tc
                   ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
               WHERE tc.organization_id = $1 AND b.category_id != 1
           )"#,
    )
    .bind(org_id as i64)
    .execute(&db_tables.pool)
    .await;

    // 2. Delete territory contours
    let _ = sqlx::query(
        "DELETE FROM organizations.territory_contours WHERE organization_id = $1",
    )
    .bind(org_id as i64)
    .execute(&db_tables.pool)
    .await;

    // 3. Delete territory cells
    let _ = sqlx::query(
        "DELETE FROM organizations.territory_cells WHERE organization_id = $1",
    )
    .bind(org_id as i64)
    .execute(&db_tables.pool)
    .await;

    // 4. Delete non-lord member units
    let _ = sqlx::query(
        r#"DELETE FROM units.units WHERE id IN (
            SELECT unit_id FROM organizations.members WHERE organization_id = $1
        ) AND is_lord = false"#,
    )
    .bind(org_id as i64)
    .execute(&db_tables.pool)
    .await;

    // 5. Delete members
    let _ = sqlx::query(
        "DELETE FROM organizations.members WHERE organization_id = $1",
    )
    .bind(org_id as i64)
    .execute(&db_tables.pool)
    .await;

    // 6. Reset lord slot
    let _ = sqlx::query(
        "UPDATE units.units SET slot_type = NULL, slot_index = NULL WHERE id = $1",
    )
    .bind(lord.id as i64)
    .execute(&db_tables.pool)
    .await;

    // 7. Delete organization
    let _ = sqlx::query("DELETE FROM organizations.organizations WHERE id = $1")
        .bind(org_id as i64)
        .execute(&db_tables.pool)
        .await;

    tracing::info!(
        "✓ Organization {} liquidated ({} chunks affected)",
        org_id, affected_chunks.len()
    );

    bridge_sender.send(BridgeEvent::SendOrganizationLiquidated {
        player_id,
        organization_id: org_id,
        affected_chunks,
    });
}
