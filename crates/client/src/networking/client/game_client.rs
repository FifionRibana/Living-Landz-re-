use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::block_on;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::TerrainChunkId;
use shared::grid::GridCell;
use shared::protocol::channels::ReliableGameChannel;
use shared::protocol::messages::HamletFoundErrorMsg;

use crate::networking::client::auth_task::AuthTask;
use crate::networking::client::http_client::{
    HttpBulkClient, HttpGlobalReceiver, HttpGlobalResult, HttpGlobalSender, HttpTerrainReceiver,
    HttpTerrainSender,
};
use crate::state::resources::{
    ActionTracker, ConnectionStatus, CurrentOrganization, GameDataCache, InventoryCache,
    NotificationState, PlayerInfo, TrackedAction, UnitsCache, UnitsDataCache, WorldCache,
};
use crate::states::AppState;
use shared::protocol::components::{LordPosition, MovingUnitId, MovingUnitPosition, OwnedByPlayer};
use shared::protocol::messages::{
    ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
    ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg, ActionStatusMsg,
    ActionTrainUnitMsg, AssignUnitToSlotMsg, BuildingDestroyedMsg, CreateLordMsg,
    DebugCreateOrganizationMsg, DebugDeleteOrganizationMsg, DebugErrorMsg,
    DebugOrganizationCreatedMsg, DebugOrganizationDeletedMsg, DebugSpawnUnitMsg,
    DebugUnitSpawnedMsg, DestroyBuildingMsg, ExplorationPatchMsg, FoundHamletMsg, GameDataMsg,
    HamletFoundedMsg, InventoryDataMsg, InventoryUpdateMsg, LiquidateOrganizationMsg,
    LoginSuccessMsg, LordCreateErrorMsg, LordCreatedMsg, LordDataMsg, MoveUnitToSlotMsg,
    OrganizationAtCellMsg, OrganizationLiquidatedMsg, PlayerOrganizationDataMsg,
    PopulationChangedMsg, RequestInventoryMsg, RequestOrganizationAtCellMsg,
    RoadChunkSdfUpdateMsg, TerrainChunkDataMsg, TerritoryBorderSdfUpdateMsg,
    TerritoryContourUpdateMsg, UnitPositionUpdatedMsg, UnitProfessionChangedMsg,
    UnitSlotUpdatedMsg, UnitWorkStatusUpdateMsg,
};

/// Bevy Message: UI systems write this, lightyear send system reads it.
#[derive(Message, Clone)]
pub struct SendActionMoveUnit {
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

#[derive(Message, Clone)]
pub struct SendActionBuildBuilding {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub building_type: shared::BuildingTypeEnum,
}

#[derive(Message, Clone)]
pub struct SendActionBuildRoad {
    pub start_cell: GridCell,
    pub end_cell: GridCell,
}

#[derive(Message, Clone)]
pub struct SendActionHarvestResource {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub resource_specific_type: shared::ResourceSpecificTypeEnum,
    pub unit_ids: Vec<u64>,
}

#[derive(Message, Clone)]
pub struct SendActionCraftResource {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub recipe_id: String,
    pub quantity: u32,
    pub unit_ids: Vec<u64>,
}

#[derive(Message, Clone)]
pub struct SendActionTrainUnit {
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub target_profession: shared::ProfessionEnum,
}

#[derive(Message, Clone)]
pub struct SendActionExplore {
    pub cell: GridCell,
    pub radius: i32,
}

#[derive(Message, Clone)]
pub struct SendRequestInventory {
    pub unit_id: u64,
}

#[derive(Message, Clone)]
pub struct SendRequestOrganizationAtCell {
    pub cell: GridCell,
}

#[derive(Resource, Default)]
pub struct TerritoryDebugCells(pub Vec<GridCell>);

// ─── Remaining command Bevy Messages (#138) ─────────────────────────

#[derive(Message, Clone)]
pub struct SendCreateLord {
    pub first_name: String,
    pub gender: String,
    pub portrait_layers: String,
}

#[derive(Message, Clone)]
pub struct SendFoundHamlet;

#[derive(Message, Clone)]
pub struct SendDestroyBuilding {
    pub cell: GridCell,
}

#[derive(Message, Clone)]
pub struct SendLiquidateOrganization;

#[derive(Message, Clone)]
pub struct SendMoveUnitToSlot {
    pub unit_id: u64,
    pub cell: GridCell,
    pub from_slot: shared::SlotPosition,
    pub to_slot: shared::SlotPosition,
}

#[derive(Message, Clone)]
pub struct SendAssignUnitToSlot {
    pub unit_id: u64,
    pub cell: GridCell,
    pub slot: shared::SlotPosition,
}

#[derive(Message, Clone)]
pub struct SendDebugCreateOrganization {
    pub name: String,
    pub organization_type: shared::OrganizationType,
    pub cell: GridCell,
    pub parent_organization_id: Option<u64>,
}

#[derive(Message, Clone)]
pub struct SendDebugDeleteOrganization {
    pub organization_id: u64,
}

#[derive(Message, Clone)]
pub struct SendDebugSpawnUnit {
    pub cell: GridCell,
}

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        // HTTP bulk channels
        let (terrain_tx, terrain_rx) = std::sync::mpsc::channel();
        let (global_tx, global_rx) = std::sync::mpsc::channel();

        app.init_resource::<AuthTask>()
            .init_resource::<PendingTerrainChunks>()
            .insert_resource(HttpBulkClient::new())
            .insert_resource(HttpTerrainSender { tx: terrain_tx })
            .insert_resource(HttpTerrainReceiver::new(terrain_rx))
            .insert_resource(HttpGlobalSender { tx: global_tx })
            .insert_resource(HttpGlobalReceiver::new(global_rx))
            .init_resource::<TerritoryDebugCells>()
            .add_systems(
                Update,
                (
                    poll_auth_task,
                    receive_login_success,
                    receive_lord_data,
                    receive_organization_data,
                    receive_game_data,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_connection_events,
                    handle_lord_replication,
                    handle_moving_unit_replication,
                    receive_action_status,
                    receive_action_error,
                    receive_unit_position_updated,
                    receive_action_completed,
                )
                    .run_if(in_state(AppState::InGame)),
            );

        app.add_message::<SendActionMoveUnit>()
            .add_message::<SendActionBuildBuilding>()
            .add_message::<SendActionBuildRoad>()
            .add_message::<SendActionHarvestResource>()
            .add_message::<SendActionCraftResource>()
            .add_message::<SendActionTrainUnit>()
            .add_message::<SendActionExplore>()
            .add_message::<SendRequestInventory>()
            .add_message::<SendRequestOrganizationAtCell>()
            .add_message::<SendCreateLord>()
            .add_message::<SendFoundHamlet>()
            .add_message::<SendDestroyBuilding>()
            .add_message::<SendLiquidateOrganization>()
            .add_message::<SendMoveUnitToSlot>()
            .add_message::<SendAssignUnitToSlot>()
            .add_message::<SendDebugCreateOrganization>()
            .add_message::<SendDebugDeleteOrganization>()
            .add_message::<SendDebugSpawnUnit>()
            .add_systems(
                Update,
                (
                    send_pending_move_actions,
                    send_pending_build_building_actions,
                    send_pending_build_road_actions,
                    send_pending_harvest_resource_actions,
                    send_pending_craft_resource_actions,
                    send_pending_train_unit_actions,
                    send_pending_explore_actions,
                    send_inventory_requests,
                    send_organization_at_cell_requests,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    send_create_lord,
                    send_found_hamlet,
                    send_move_unit_to_slot,
                    send_assign_unit_to_slot,
                    send_debug_create_organization,
                    send_debug_delete_organization,
                    send_debug_spawn_unit,
                    send_destroy_building,
                    send_liquidate_organization,
                    receive_lord_created,
                    receive_lord_create_error,
                ),
            );

        // Bulk data + event receivers (run in InGame — world resources must exist)
        app.add_systems(
            Update,
            (
                process_pending_terrain_chunks,
                poll_http_terrain_results,
                poll_http_global_results,
                receive_exploration_patch,
                receive_inventory_data,
                receive_inventory_update,
            )
                .run_if(in_state(AppState::InGame)),
        );
        app.add_systems(
            Update,
            (
                receive_unit_profession_changed,
                receive_unit_work_status_update,
                receive_road_chunk_sdf_update,
                receive_territory_contour_update,
                receive_territory_border_sdf_update,
                receive_hamlet_founded,
                receive_hamlet_found_error,
                receive_population_changed,
                receive_organization_at_cell,
                receive_unit_slot_updated,
                receive_debug_messages,
                receive_building_destroyed,
                receive_organization_liquidated,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Poll the auth task. When the HTTP response arrives, decode the ConnectToken
/// and initiate the lightyear connection.
fn poll_auth_task(
    mut commands: Commands,
    mut auth_task: ResMut<AuthTask>,
    mut connection: ResMut<ConnectionStatus>,
) {
    if auth_task.completed {
        return;
    }

    let Some(ref task) = auth_task.task else {
        return;
    };

    if !task.is_finished() {
        return;
    }

    // Task finished — take it out and get the result
    let task = auth_task.task.take().unwrap();
    let result = block_on(task);

    match result {
        Ok(auth_result) => {
            let player_id = auth_result.player_id;
            connection.player_id = Some(player_id);

            let server_addr: std::net::SocketAddr = std::env::var("LIGHTYEAR_SERVER")
                .unwrap_or_else(|_| "127.0.0.1:5000".to_string())
                .parse()
                .unwrap();

            let client = commands
                .spawn((
                    Client::default(),
                    LocalAddr("127.0.0.1:0".parse().unwrap()),
                    PeerAddr(server_addr),
                    Link::new(None),
                    ReplicationReceiver::default(),
                    NetcodeClient::new(
                        Authentication::Token(auth_result.connect_token),
                        NetcodeConfig::default(),
                    )
                    .unwrap(),
                    UdpIo::default(),
                ))
                .id();
            commands.trigger(Connect { entity: client });

            auth_task.completed = true;
            info!(
                "🔌 Lightyear connecting with ConnectToken for player_id={}",
                player_id
            );
        }
        Err(e) => {
            warn!("❌ Auth failed: {}", e);
        }
    }
}

/// Log lightyear connection events
fn handle_connection_events(
    clients: Query<(Entity, &Client, Option<&Connected>), Changed<Client>>,
) {
    for (entity, _client, connected) in clients.iter() {
        if connected.is_some() {
            info!("✓ Lightyear connected (entity {:?})", entity);
        }
    }
}

/// React to lord position changes replicated from server.
/// Later this will update the client's WorldCache and camera.
fn handle_lord_replication(
    lords: Query<(&OwnedByPlayer, &LordPosition), Changed<LordPosition>>,
    mut player_info: ResMut<PlayerInfo>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
) {
    for (owner, pos) in lords.iter() {
        let Some(ref mut lord) = player_info.lord else {
            continue;
        };

        // Only process our own lord
        if lord.player_id != Some(owner.0) {
            continue;
        }

        let new_cell = shared::grid::GridCell {
            q: pos.cell_q,
            r: pos.cell_r,
        };
        let new_chunk = shared::TerrainChunkId {
            x: pos.chunk_x,
            y: pos.chunk_y,
        };

        // Skip if position hasn't actually changed (initial replication = same as login)
        if lord.current_cell == new_cell && lord.current_chunk == new_chunk {
            continue;
        }

        let unit_id = lord.id;
        let old_cell = lord.current_cell;

        info!(
            "📍 Lord {} moved ({},{}) → ({},{}) via lightyear",
            unit_id, old_cell.q, old_cell.r, new_cell.q, new_cell.r
        );

        // 1. Update PlayerInfo lord position
        lord.current_cell = new_cell;
        lord.current_chunk = new_chunk;

        // 2. Update UnitsCache (cell → unit_id mapping for rendering)
        if let Some(ref mut cache) = units_cache {
            cache.remove_unit(unit_id);
            cache.add_unit(new_cell, unit_id);
        }

        // 3. Update UnitsDataCache (full unit data)
        if let Some(ref mut data_cache) = units_data_cache
            && let Some(unit) = data_cache.get_unit_mut(unit_id)
        {
            unit.current_cell = new_cell;
            unit.current_chunk = new_chunk;
        }
    }
}

/// Log when a moving unit appears or changes position via lightyear replication.
fn handle_moving_unit_replication(
    units: Query<(&MovingUnitId, &MovingUnitPosition, &OwnedByPlayer), Changed<MovingUnitPosition>>,
) {
    for (unit_id, pos, owner) in units.iter() {
        info!(
            "🚶 Moving unit {} (owner {}) at chunk ({},{}) cell ({},{})",
            unit_id.0, owner.0, pos.chunk_x, pos.chunk_y, pos.cell_q, pos.cell_r
        );
    }
}

/// Receive ActionStatusMsg from server via lightyear.
fn receive_action_status(
    mut receivers: Query<&mut MessageReceiver<ActionStatusMsg>>,
    mut action_tracker: Option<ResMut<ActionTracker>>,
    mut notifications: ResMut<NotificationState>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "📨 Action {} status: {:?} (via lightyear)",
                msg.action_id, msg.status
            );

            if let Some(ref mut tracker) = action_tracker {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let start_time = tracker
                    .get_action(msg.action_id)
                    .map(|a| a.start_time)
                    .unwrap_or(current_time);

                tracker.update_action(TrackedAction {
                    action_id: msg.action_id,
                    player_id: msg.player_id,
                    chunk_id: msg.chunk_id,
                    cell: msg.cell,
                    action_type: msg.action_type,
                    status: msg.status,
                    start_time,
                    completion_time: msg.completion_time,
                    action_name: msg.action_name.clone(),
                    unit_ids: msg.unit_ids.clone(),
                });

                match msg.status {
                    shared::ActionStatusEnum::Pending => {
                        notifications
                            .push_info(format!("{} en attente...", msg.action_type.to_name()));
                    }
                    shared::ActionStatusEnum::InProgress => {
                        notifications.push_info(format!("{} en cours", msg.action_type.to_name()));
                    }
                    shared::ActionStatusEnum::Failed => {
                        notifications.push_error(format!("{} échouée", msg.action_type.to_name()));
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Receive ActionErrorMsg from server via lightyear.
fn receive_action_error(
    mut receivers: Query<&mut MessageReceiver<ActionErrorMsg>>,
    mut notifications: ResMut<NotificationState>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            warn!("❌ Action error (via lightyear): {}", msg.reason);
            notifications.push_error(msg.reason.clone());
        }
    }
}

/// Receive UnitPositionUpdatedMsg from server via lightyear (non-lord units).
fn receive_unit_position_updated(
    mut receivers: Query<&mut MessageReceiver<UnitPositionUpdatedMsg>>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "📨 Unit {} moved ({},{}) → ({},{}) (via lightyear)",
                msg.unit_id, msg.from_cell.q, msg.from_cell.r, msg.to_cell.q, msg.to_cell.r
            );

            // Update UnitsCache (cell → unit_id mapping for rendering)
            if let Some(ref mut cache) = units_cache {
                cache.remove_unit(msg.unit_id);
                cache.add_unit(msg.to_cell, msg.unit_id);
            }

            // Update UnitsDataCache (full unit data)
            if let Some(ref mut data_cache) = units_data_cache
                && let Some(unit) = data_cache.get_unit_mut(msg.unit_id)
            {
                unit.current_cell = msg.to_cell;
                unit.current_chunk = msg.to_chunk;
            }
        }
    }
}

/// Receive ActionCompletedMsg from server via lightyear.
/// Triggers a chunk data refresh via HTTP so the client sees new buildings/roads/etc.
fn receive_action_completed(
    mut receivers: Query<&mut MessageReceiver<ActionCompletedMsg>>,
    mut notifications: ResMut<NotificationState>,
    http_client: Res<HttpBulkClient>,
    http_sender: Res<HttpTerrainSender>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "📨 Action {} completed at chunk ({},{}) cell ({},{}) (via lightyear)",
                msg.action_id, msg.chunk_id.x, msg.chunk_id.y, msg.cell.q, msg.cell.r
            );

            notifications.push_success(format!("{} terminée !", msg.action_type.to_name()));

            // Request chunk data refresh via HTTP so the client sees the result
            let sender = http_sender.clone();
            let client = http_client.client.clone();
            let base_url = http_client.base_url.clone();
            let chunk_id = msg.chunk_id;

            IoTaskPool::get()
                .spawn(async_compat::Compat::new(async move {
                    let body = serde_json::json!({
                        "terrain_name": "Gaulyia",
                        "chunk_ids": [[chunk_id.x, chunk_id.y]],
                    });
                    match client.post(format!("{}/api/terrain/chunks", base_url))
                        .json(&body)
                        .send()
                        .await
                    {
                        Ok(response) => match response.bytes().await {
                            Ok(bytes) => {
                                match crate::networking::client::http_client::parse_terrain_response_pub(&bytes) {
                                    Ok(parsed) => { let _ = sender.tx.send(parsed); }
                                    Err(e) => bevy::log::error!("HTTP terrain parse error: {}", e),
                                }
                            }
                            Err(e) => bevy::log::error!("HTTP terrain response error: {}", e),
                        },
                        Err(e) => bevy::log::error!("HTTP terrain request error: {}", e),
                    }
                })).detach();
        }
    }
}

// ─── Post-login data receivers ───────────────────────────────────────

/// Receive LoginSuccessMsg from server via lightyear.
fn receive_login_success(
    mut receivers: Query<&mut MessageReceiver<LoginSuccessMsg>>,
    mut connection: ResMut<ConnectionStatus>,
    mut player_info: ResMut<PlayerInfo>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Login successful via lightyear, player ID: {}",
                msg.player.id
            );
            connection.logged_in = true;
            connection.player_id = Some(msg.player.id as u64);
            player_info.temp_player_name = Some(msg.player.family_name.clone());

            if let Some(ref char_data) = msg.character {
                let character_name = if let Some(ref nickname) = char_data.nickname {
                    format!(
                        "{} \"{}\" {}",
                        char_data.first_name, nickname, char_data.family_name
                    )
                } else {
                    format!("{} {}", char_data.first_name, char_data.family_name)
                };
                player_info.temp_character_name = Some(character_name);
            }
        }
    }
}

/// Receive LordDataMsg from server via lightyear.
/// Transitions to InGame if lord exists, or CharacterCreation if not.
/// When entering InGame, spawns HTTP fetches for global bulk data (ocean, lake, terrain-global, exploration).
fn receive_lord_data(
    mut receivers: Query<&mut MessageReceiver<LordDataMsg>>,
    mut player_info: ResMut<PlayerInfo>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut inventory_events: MessageWriter<SendRequestInventory>,
    http_client: Res<HttpBulkClient>,
    global_sender: Res<HttpGlobalSender>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            if let Some(lord_data) = msg.lord {
                info!(
                    "✓ Lord loaded via lightyear: {} at ({},{})",
                    lord_data.full_name(),
                    lord_data.current_cell.q,
                    lord_data.current_cell.r
                );

                // Request inventory for the lord via lightyear
                inventory_events.write(SendRequestInventory {
                    unit_id: lord_data.id,
                });

                player_info.set_lord(lord_data);
                next_app_state.set(AppState::InGame);

                // Spawn HTTP fetches for global bulk data in parallel
                let client = http_client.clone();
                let sender = global_sender.clone();
                IoTaskPool::get()
                    .spawn(async_compat::Compat::new(async move {
                        let (ocean, lake, terrain_global, exploration) = futures::future::join4(
                            client.fetch_ocean("Gaulyia"),
                            client.fetch_lake("Gaulyia"),
                            client.fetch_terrain_global("Gaulyia"),
                            client.fetch_exploration("Gaulyia"),
                        )
                        .await;

                        if let Ok(data) = ocean {
                            let _ = sender.tx.send(HttpGlobalResult::Ocean(data));
                        } else if let Err(e) = ocean {
                            bevy::log::error!("HTTP ocean fetch failed: {}", e);
                        }
                        if let Ok(data) = lake {
                            let _ = sender.tx.send(HttpGlobalResult::Lake(data));
                        } else if let Err(e) = lake {
                            bevy::log::error!("HTTP lake fetch failed: {}", e);
                        }
                        if let Ok(data) = terrain_global {
                            let _ = sender.tx.send(HttpGlobalResult::TerrainGlobal(data));
                        } else if let Err(e) = terrain_global {
                            bevy::log::error!("HTTP terrain-global fetch failed: {}", e);
                        }
                        if let Ok(data) = exploration {
                            let _ = sender.tx.send(HttpGlobalResult::Exploration(data));
                        } else if let Err(e) = exploration {
                            bevy::log::error!("HTTP exploration fetch failed: {}", e);
                        }
                    }))
                    .detach();
            } else {
                info!("No lord — entering character creation (via lightyear)");
                next_app_state.set(AppState::CharacterCreation);
            }
        }
    }
}

/// Receive PlayerOrganizationDataMsg from server via lightyear.
fn receive_organization_data(
    mut receivers: Query<&mut MessageReceiver<PlayerOrganizationDataMsg>>,
    mut player_info: ResMut<PlayerInfo>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            if let Some(ref org) = msg.organization {
                info!(
                    "✓ Organization loaded via lightyear: {} (ID: {})",
                    org.name, org.id
                );
            }
            player_info.organization = msg.organization;
        }
    }
}

/// Receive GameDataMsg from server via lightyear.
fn receive_game_data(
    mut receivers: Query<&mut MessageReceiver<GameDataMsg>>,
    mut game_data_cache: ResMut<GameDataCache>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Game data loaded via lightyear: {} items, {} recipes",
                msg.payload.items.len(),
                msg.payload.recipes.len()
            );
            game_data_cache.load_from_payload(msg.payload);
        }
    }
}

// ─── Action send systems ─────────────────────────────────────────────

fn send_pending_move_actions(
    mut events: MessageReader<SendActionMoveUnit>,
    mut senders: Query<&mut MessageSender<ActionMoveUnitMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionMoveUnitMsg {
                unit_id: event.unit_id,
                chunk_id: event.chunk_id,
                cell: event.cell,
            });
            info!(
                "📤 Sent ActionMoveUnit via lightyear for unit {}",
                event.unit_id
            );
            break;
        }
    }
}

fn send_pending_build_building_actions(
    mut events: MessageReader<SendActionBuildBuilding>,
    mut senders: Query<&mut MessageSender<ActionBuildBuildingMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionBuildBuildingMsg {
                chunk_id: event.chunk_id,
                cell: event.cell,
                building_type: event.building_type,
            });
            info!("📤 Sent ActionBuildBuilding via lightyear");
            break;
        }
    }
}

fn send_pending_build_road_actions(
    mut events: MessageReader<SendActionBuildRoad>,
    mut senders: Query<&mut MessageSender<ActionBuildRoadMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionBuildRoadMsg {
                start_cell: event.start_cell,
                end_cell: event.end_cell,
            });
            info!("📤 Sent ActionBuildRoad via lightyear");
            break;
        }
    }
}

fn send_pending_harvest_resource_actions(
    mut events: MessageReader<SendActionHarvestResource>,
    mut senders: Query<&mut MessageSender<ActionHarvestResourceMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionHarvestResourceMsg {
                chunk_id: event.chunk_id,
                cell: event.cell,
                resource_specific_type: event.resource_specific_type,
                unit_ids: event.unit_ids.clone(),
            });
            info!("📤 Sent ActionHarvestResource via lightyear");
            break;
        }
    }
}

fn send_pending_craft_resource_actions(
    mut events: MessageReader<SendActionCraftResource>,
    mut senders: Query<&mut MessageSender<ActionCraftResourceMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionCraftResourceMsg {
                chunk_id: event.chunk_id,
                cell: event.cell,
                recipe_id: event.recipe_id.clone(),
                quantity: event.quantity,
                unit_ids: event.unit_ids.clone(),
            });
            info!(
                "📤 Sent ActionCraftResource via lightyear: recipe '{}'",
                event.recipe_id
            );
            break;
        }
    }
}

fn send_pending_train_unit_actions(
    mut events: MessageReader<SendActionTrainUnit>,
    mut senders: Query<&mut MessageSender<ActionTrainUnitMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionTrainUnitMsg {
                unit_id: event.unit_id,
                chunk_id: event.chunk_id,
                cell: event.cell,
                target_profession: event.target_profession,
            });
            info!(
                "📤 Sent ActionTrainUnit via lightyear for unit {}",
                event.unit_id
            );
            break;
        }
    }
}

fn send_pending_explore_actions(
    mut events: MessageReader<SendActionExplore>,
    mut senders: Query<&mut MessageSender<ActionExploreMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(ActionExploreMsg {
                cell: event.cell,
                radius: event.radius,
            });
            info!("📤 Sent ActionExplore via lightyear");
            break;
        }
    }
}

// ─── Bulk data ──────────────────────────────────────────────────────

/// Max terrain chunks to decompress + insert per frame to avoid frame drops.
const MAX_TERRAIN_CHUNKS_PER_FRAME: usize = 4;

/// Pending terrain chunks waiting to be processed (buffered from HTTP receiver).
#[derive(Resource, Default)]
pub struct PendingTerrainChunks(pub Vec<TerrainChunkDataMsg>);

/// Process up to N pending terrain chunks per frame.
/// Drops stale chunks (camera moved away) BEFORE decompressing to save CPU/memory.
fn process_pending_terrain_chunks(
    mut pending: ResMut<PendingTerrainChunks>,
    mut cache: Option<ResMut<WorldCache>>,
    mut units_cache: Option<ResMut<UnitsCache>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut commands: Commands,
    terrain_query: Query<(Entity, &crate::rendering::terrain::components::Terrain)>,
    camera: Query<&Transform, With<crate::camera::MainCamera>>,
    streaming_config: Res<crate::state::resources::StreamingConfig>,
    mut contour_cache: ResMut<crate::rendering::territory::TerritoryContourCache>,
) {
    let Some(ref mut cache) = cache else { return };
    let Some(ref mut units_cache) = units_cache else {
        return;
    };
    let Some(ref mut units_data_cache) = units_data_cache else {
        return;
    };

    if pending.0.is_empty() {
        return;
    }

    // Compute current camera chunk to detect stale chunks
    let camera_chunk = camera.single().ok().map(|transform| {
        let pos = transform.translation.truncate();
        TerrainChunkId {
            x: pos.x.div_euclid(shared::constants::CHUNK_SIZE.x).ceil() as i32,
            y: pos.y.div_euclid(shared::constants::CHUNK_SIZE.y).ceil() as i32,
        }
    });

    let unload_dist = streaming_config.unload_distance;

    // First pass: drop stale chunks from the pending queue (cheap — no decompression)
    if let Some(ref cam) = camera_chunk {
        let before = pending.0.len();
        pending.0.retain(|msg| {
            let dx = (msg.chunk_id.x - cam.x).abs();
            let dy = (msg.chunk_id.y - cam.y).abs();
            dx <= unload_dist && dy <= unload_dist
        });
        let dropped = before - pending.0.len();
        if dropped > 0 {
            info!(
                "🗑️ Dropped {} stale pending chunks (camera moved away)",
                dropped
            );
        }
    }

    let to_process = pending.0.len().min(MAX_TERRAIN_CHUNKS_PER_FRAME);
    let batch: Vec<_> = pending.0.drain(..to_process).collect();

    for msg in batch {
        // Double-check staleness for this specific chunk (camera may have moved during processing)
        if let Some(ref cam) = camera_chunk {
            let dx = (msg.chunk_id.x - cam.x).abs();
            let dy = (msg.chunk_id.y - cam.y).abs();
            if dx > unload_dist || dy > unload_dist {
                continue;
            }
        }

        // Decompress + deserialize
        let decompressed = match shared::protocol::bulk_compress::decompress(&msg.compressed_data) {
            Ok(d) => d,
            Err(e) => {
                warn!(
                    "Failed to decompress terrain chunk ({},{}): {}",
                    msg.chunk_id.x, msg.chunk_id.y, e
                );
                continue;
            }
        };

        let (payload, _): (
            (
                shared::TerrainChunkData,
                Vec<shared::BiomeChunkData>,
                Vec<shared::grid::CellData>,
                Vec<shared::BuildingData>,
                Vec<shared::UnitData>,
                Option<shared::RoadChunkSdfData>,
                Vec<shared::protocol::TerritoryContourChunkData>,
            ),
            _,
        ) = match bincode::decode_from_slice(&decompressed, bincode::config::standard()) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Failed to decode terrain chunk ({},{}): {}",
                    msg.chunk_id.x, msg.chunk_id.y, e
                );
                continue;
            }
        };

        let (
            mut terrain_chunk_data,
            biome_chunk_data,
            cell_data,
            building_data,
            unit_data,
            road_sdf,
            contours,
        ) = payload;

        let terrain_already_loaded =
            cache.is_terrain_loaded(&terrain_chunk_data.name, &terrain_chunk_data.id);

        if !terrain_already_loaded {
            // Apply road SDF to the terrain chunk before inserting into cache
            if let Some(road_sdf_data) = road_sdf {
                terrain_chunk_data.road_sdf_data = Some(road_sdf_data);
            }

            let is_update = cache.insert_terrain(&terrain_chunk_data);
            if is_update {
                let terrain_name = &terrain_chunk_data.name;
                let terrain_id = terrain_chunk_data.id;
                for (entity, terrain) in terrain_query.iter() {
                    if &terrain.name == terrain_name && terrain.id == terrain_id {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }

            for chunk_data in &biome_chunk_data {
                cache.insert_biome(chunk_data);
            }
            cache.insert_cells(&cell_data);
        }

        // Always update buildings/units/contours — they can change after terrain is loaded
        // (e.g. new building constructed, unit moved in)
        cache.insert_buildings(&building_data);

        for unit in &unit_data {
            let cell = unit.current_cell;
            units_cache.add_unit(cell, unit.id);

            if let Some(slot_pos) = crate::networking::handlers::db_to_slot_position(
                unit.slot_type.clone(),
                unit.slot_index,
            ) {
                units_cache.set_unit_slot(cell, slot_pos, unit.id);
            }
            units_data_cache.insert_unit(unit.clone());
        }

        // Insert territory contours into cache
        for contour_data in &contours {
            contour_cache.add_contour(
                msg.chunk_id,
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

    if !pending.0.is_empty() {
        info!(
            "🔄 {} terrain chunks still pending for next frame",
            pending.0.len()
        );
    }
}

fn receive_exploration_patch(
    mut receivers: Query<&mut MessageReceiver<ExplorationPatchMsg>>,
    mut cache: Option<ResMut<WorldCache>>,
) {
    let Some(ref mut cache) = cache else { return };
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            let data = match shared::protocol::bulk_compress::decompress(&msg.compressed_data) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to decompress exploration patch: {}", e);
                    continue;
                }
            };
            cache.apply_exploration_patch(
                msg.patch_x,
                msg.patch_y,
                msg.patch_width,
                msg.patch_height,
                &data,
            );
        }
    }
}

// ─── HTTP bulk poll systems ─────────────────────────────────────────

/// Poll terrain chunk results from HTTP async tasks and push into PendingTerrainChunks.
fn poll_http_terrain_results(
    http_receiver: Res<HttpTerrainReceiver>,
    mut pending: ResMut<PendingTerrainChunks>,
) {
    while let Some(chunks) = http_receiver.try_recv() {
        for (chunk_id, compressed_data) in chunks {
            pending.0.push(TerrainChunkDataMsg {
                chunk_id,
                compressed_data,
            });
        }
    }
}

/// Poll global data results from HTTP async tasks and insert into WorldCache.
fn poll_http_global_results(
    http_receiver: Res<HttpGlobalReceiver>,
    mut cache: Option<ResMut<WorldCache>>,
) {
    let Some(ref mut cache) = cache else { return };

    while let Some(result) = http_receiver.try_recv() {
        match result {
            HttpGlobalResult::Ocean(compressed_data) => {
                let decompressed =
                    match shared::protocol::bulk_compress::decompress(&compressed_data) {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Failed to decompress HTTP ocean data: {}", e);
                            continue;
                        }
                    };
                let (ocean_data, _): (shared::OceanData, _) =
                    match bincode::decode_from_slice(&decompressed, bincode::config::standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Failed to decode HTTP ocean data: {}", e);
                            continue;
                        }
                    };
                info!("✓ Received ocean data via HTTP: {}", ocean_data.name);
                cache.insert_ocean(ocean_data);
            }
            HttpGlobalResult::Lake(compressed_data) => {
                let decompressed =
                    match shared::protocol::bulk_compress::decompress(&compressed_data) {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Failed to decompress HTTP lake data: {}", e);
                            continue;
                        }
                    };
                let (lake_data, _): (shared::LakeData, _) =
                    match bincode::decode_from_slice(&decompressed, bincode::config::standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Failed to decode HTTP lake data: {}", e);
                            continue;
                        }
                    };
                info!("✓ Received lake data via HTTP: {}", lake_data.name);
                cache.insert_lake(lake_data);
            }
            HttpGlobalResult::TerrainGlobal(compressed_data) => {
                let decompressed =
                    match shared::protocol::bulk_compress::decompress(&compressed_data) {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Failed to decompress HTTP terrain global data: {}", e);
                            continue;
                        }
                    };
                let (data, _): (shared::TerrainGlobalData, _) =
                    match bincode::decode_from_slice(&decompressed, bincode::config::standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Failed to decode HTTP terrain global data: {}", e);
                            continue;
                        }
                    };
                info!(
                    "✓ Received terrain global data via HTTP: biome {}x{}, heightmap {}x{}",
                    data.biome_width,
                    data.biome_height,
                    data.heightmap_width,
                    data.heightmap_height
                );
                cache.insert_terrain_global(data);
            }
            HttpGlobalResult::Exploration(exploration) => {
                let data =
                    match shared::protocol::bulk_compress::decompress(&exploration.compressed_data)
                    {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Failed to decompress HTTP exploration map: {}", e);
                            continue;
                        }
                    };
                info!(
                    "✓ Received exploration map via HTTP: {}×{} ({} bytes)",
                    exploration.width,
                    exploration.height,
                    data.len()
                );
                cache.set_exploration_map(
                    exploration.width,
                    exploration.height,
                    data,
                    exploration.n_chunk_x,
                    exploration.n_chunk_y,
                );
            }
        }
    }
}

// ─── Step 2: Request send systems ────────────────────────────────────

fn send_inventory_requests(
    mut events: MessageReader<SendRequestInventory>,
    mut senders: Query<&mut MessageSender<RequestInventoryMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(RequestInventoryMsg {
                unit_id: event.unit_id,
            });
            break;
        }
    }
}

fn send_organization_at_cell_requests(
    mut events: MessageReader<SendRequestOrganizationAtCell>,
    mut senders: Query<&mut MessageSender<RequestOrganizationAtCellMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(RequestOrganizationAtCellMsg { cell: event.cell });
            break;
        }
    }
}

// ─── Remaining command senders (#138) ────────────────────────────────

fn send_create_lord(
    mut events: MessageReader<SendCreateLord>,
    mut senders: Query<&mut MessageSender<CreateLordMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(CreateLordMsg {
                first_name: event.first_name.clone(),
                gender: event.gender.clone(),
                portrait_layers: event.portrait_layers.clone(),
            });
            break;
        }
    }
}

fn send_found_hamlet(
    mut events: MessageReader<SendFoundHamlet>,
    mut senders: Query<&mut MessageSender<FoundHamletMsg>>,
) {
    for _event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(FoundHamletMsg);
            break;
        }
    }
}

fn send_move_unit_to_slot(
    mut events: MessageReader<SendMoveUnitToSlot>,
    mut senders: Query<&mut MessageSender<MoveUnitToSlotMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(MoveUnitToSlotMsg {
                unit_id: event.unit_id,
                cell: event.cell,
                from_slot: event.from_slot,
                to_slot: event.to_slot,
            });
            break;
        }
    }
}

fn send_assign_unit_to_slot(
    mut events: MessageReader<SendAssignUnitToSlot>,
    mut senders: Query<&mut MessageSender<AssignUnitToSlotMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(AssignUnitToSlotMsg {
                unit_id: event.unit_id,
                cell: event.cell,
                slot: event.slot,
            });
            break;
        }
    }
}

fn send_debug_create_organization(
    mut events: MessageReader<SendDebugCreateOrganization>,
    mut senders: Query<&mut MessageSender<DebugCreateOrganizationMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(DebugCreateOrganizationMsg {
                name: event.name.clone(),
                organization_type: event.organization_type,
                cell: event.cell,
                parent_organization_id: event.parent_organization_id,
            });
            break;
        }
    }
}

fn send_debug_delete_organization(
    mut events: MessageReader<SendDebugDeleteOrganization>,
    mut senders: Query<&mut MessageSender<DebugDeleteOrganizationMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(DebugDeleteOrganizationMsg {
                organization_id: event.organization_id,
            });
            break;
        }
    }
}

fn send_debug_spawn_unit(
    mut events: MessageReader<SendDebugSpawnUnit>,
    mut senders: Query<&mut MessageSender<DebugSpawnUnitMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(DebugSpawnUnitMsg { cell: event.cell });
            break;
        }
    }
}

// ─── Lord lifecycle receivers (#138) ─────────────────────────────────

fn receive_lord_created(
    mut receivers: Query<&mut MessageReceiver<LordCreatedMsg>>,
    mut player_info: ResMut<PlayerInfo>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut inventory_events: MessageWriter<SendRequestInventory>,
    http_client: Res<HttpBulkClient>,
    global_sender: Res<HttpGlobalSender>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Lord created via lightyear: {} (ID: {})",
                msg.unit_data.full_name(),
                msg.unit_data.id
            );
            let unit_id = msg.unit_data.id;
            player_info.set_lord(msg.unit_data);
            inventory_events.write(SendRequestInventory { unit_id });
            next_app_state.set(AppState::InGame);

            // Spawn HTTP fetches for global bulk data
            let client = http_client.clone();
            let sender = global_sender.clone();
            IoTaskPool::get()
                .spawn(async_compat::Compat::new(async move {
                    let (ocean, lake, terrain_global, exploration) = futures::future::join4(
                        client.fetch_ocean("Gaulyia"),
                        client.fetch_lake("Gaulyia"),
                        client.fetch_terrain_global("Gaulyia"),
                        client.fetch_exploration("Gaulyia"),
                    )
                    .await;
                    if let Ok(data) = ocean {
                        let _ = sender.tx.send(HttpGlobalResult::Ocean(data));
                    }
                    if let Ok(data) = lake {
                        let _ = sender.tx.send(HttpGlobalResult::Lake(data));
                    }
                    if let Ok(data) = terrain_global {
                        let _ = sender.tx.send(HttpGlobalResult::TerrainGlobal(data));
                    }
                    if let Ok(data) = exploration {
                        let _ = sender.tx.send(HttpGlobalResult::Exploration(data));
                    }
                }))
                .detach();
        }
    }
}

fn receive_lord_create_error(mut receivers: Query<&mut MessageReceiver<LordCreateErrorMsg>>) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            warn!("Failed to create lord: {}", msg.reason);
        }
    }
}

// ─── Step 2: Event receiver systems ──────────────────────────────────

fn receive_inventory_data(
    mut receivers: Query<&mut MessageReceiver<InventoryDataMsg>>,
    mut cache: ResMut<InventoryCache>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            cache.set_inventory(msg.unit_id, msg.items);
            info!(
                "✓ Received inventory for unit {} via lightyear",
                msg.unit_id
            );
        }
    }
}

fn receive_inventory_update(
    mut receivers: Query<&mut MessageReceiver<InventoryUpdateMsg>>,
    mut cache: ResMut<InventoryCache>,
    game_data: Res<GameDataCache>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            let item_info = game_data.get_item(msg.item_id);
            let item_name = game_data.item_name(msg.item_id, 1);
            let item_type = item_info
                .map(|i| {
                    shared::ItemTypeEnum::from_id(i.item_type_id)
                        .unwrap_or(shared::ItemTypeEnum::Unknown)
                })
                .unwrap_or(shared::ItemTypeEnum::Unknown);
            let weight = item_info.map(|i| i.weight_kg).unwrap_or(0.0);
            cache.apply_update_with_info(
                msg.unit_id,
                msg.item_id,
                msg.new_total,
                &item_name,
                item_type,
                weight,
            );
            info!(
                "✓ Inventory update via lightyear: unit {} item {} delta={} total={}",
                msg.unit_id, item_name, msg.quantity_delta, msg.new_total
            );
        }
    }
}

fn receive_unit_profession_changed(
    mut receivers: Query<&mut MessageReceiver<UnitProfessionChangedMsg>>,
    mut units_data_cache: Option<ResMut<UnitsDataCache>>,
    mut commands: Commands,
    unit_query: Query<(Entity, &crate::ui::components::SlotUnitSprite)>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Unit {} profession changed to {:?} via lightyear",
                msg.unit_id, msg.new_profession
            );
            if let Some(ref mut data_cache) = units_data_cache {
                if let Some(unit) = data_cache.get_unit_mut(msg.unit_id) {
                    unit.profession = msg.new_profession;
                    if let Some(url) = &msg.new_avatar_url {
                        unit.avatar_url = Some(url.clone());
                    }
                }
            }
            // Despawn unit sprite to trigger re-render with new portrait
            for (entity, sprite) in unit_query.iter() {
                if sprite.unit_id == msg.unit_id {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
    }
}

fn receive_unit_work_status_update(
    mut receivers: Query<&mut MessageReceiver<UnitWorkStatusUpdateMsg>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "Unit {} work status: {:?} via lightyear",
                msg.unit_id, msg.working_on_action_id
            );
        }
    }
}

fn receive_road_chunk_sdf_update(
    mut receivers: Query<&mut MessageReceiver<RoadChunkSdfUpdateMsg>>,
    mut cache: Option<ResMut<WorldCache>>,
    mut commands: Commands,
    terrain_query: Query<(Entity, &crate::rendering::terrain::components::Terrain)>,
) {
    let Some(ref mut cache) = cache else { return };
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Road SDF update for chunk ({},{}) via lightyear",
                msg.chunk_id.x, msg.chunk_id.y
            );
            let storage_key = format!("{}_{}_{}", msg.terrain_name, msg.chunk_id.x, msg.chunk_id.y);
            let terrain_chunk_opt = cache
                .loaded_terrains()
                .find(|t| t.get_storage_key() == storage_key)
                .cloned();
            if let Some(mut updated_terrain) = terrain_chunk_opt {
                updated_terrain.road_sdf_data = Some(msg.road_sdf_data);
                cache.insert_terrain(&updated_terrain);
                let terrain_id = updated_terrain.id;
                for (entity, terrain) in terrain_query.iter() {
                    if terrain.name == msg.terrain_name && terrain.id == terrain_id {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }
        }
    }
}

fn receive_territory_contour_update(
    mut receivers: Query<&mut MessageReceiver<TerritoryContourUpdateMsg>>,
    mut contour_cache: ResMut<crate::rendering::territory::TerritoryContourCache>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Territory contour update for chunk ({},{}) via lightyear",
                msg.chunk_id.x, msg.chunk_id.y
            );
            for contour_data in &msg.contours {
                contour_cache.add_contour(
                    msg.chunk_id,
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
    }
}

fn receive_territory_border_sdf_update(
    mut receivers: Query<&mut MessageReceiver<TerritoryBorderSdfUpdateMsg>>,
    mut border_cache: ResMut<crate::rendering::territory::TerritoryBorderSdfCache>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "[DEPRECATED] Territory border SDF update for chunk ({},{}) via lightyear",
                msg.chunk_id.x, msg.chunk_id.y
            );
            border_cache
                .chunks
                .insert((msg.chunk_id.x, msg.chunk_id.y), msg.border_sdf_data_list);
        }
    }
}

fn receive_hamlet_founded(
    mut receivers: Query<&mut MessageReceiver<HamletFoundedMsg>>,
    mut player_info: ResMut<PlayerInfo>,
    mut debug_cells: ResMut<TerritoryDebugCells>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Hamlet '{}' founded (org ID: {}) via lightyear",
                msg.name, msg.organization_id
            );
            let lord_unit_id = player_info.lord.as_ref().map(|l| l.id);
            let camp_cap = shared::BuildingTypeEnum::Campement.housing_capacity() as i32;
            player_info.organization = Some(shared::OrganizationSummary {
                id: msg.organization_id,
                name: msg.name.clone(),
                organization_type: shared::OrganizationType::Hamlet,
                leader_unit_id: lord_unit_id,
                population: camp_cap,
                named_unit_count: 0,
                population_capacity: camp_cap,
                emblem_url: None,
            });
            debug_cells.0 = msg.territory_cells.clone();
        }
    }
}

fn receive_population_changed(
    mut receivers: Query<&mut MessageReceiver<PopulationChangedMsg>>,
    mut player_info: ResMut<PlayerInfo>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "Population changed: org {} pop {} / {} ({} named) via lightyear",
                msg.organization_id,
                msg.new_population,
                msg.population_capacity,
                msg.named_unit_count
            );
            if let Some(ref mut org) = player_info.organization {
                if org.id == msg.organization_id {
                    org.population = msg.new_population;
                    org.named_unit_count = msg.named_unit_count;
                    org.population_capacity = msg.population_capacity;
                }
            }
        }
    }
}

fn receive_organization_at_cell(
    mut receivers: Query<&mut MessageReceiver<OrganizationAtCellMsg>>,
    mut current_organization: Option<ResMut<CurrentOrganization>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            if let Some(ref mut current) = current_organization {
                current.update(msg.cell, msg.organization.clone());
            }
        }
    }
}

fn receive_unit_slot_updated(
    mut receivers: Query<&mut MessageReceiver<UnitSlotUpdatedMsg>>,
    mut units_cache: Option<ResMut<UnitsCache>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Unit {} slot updated at ({},{}) via lightyear",
                msg.unit_id, msg.cell.q, msg.cell.r
            );
            if let Some(ref mut cache) = units_cache {
                // Remove unit from its old slot first
                if let Some(old_slot) = cache.get_unit_slot(&msg.cell, msg.unit_id) {
                    cache.remove_unit_from_slot(msg.cell, old_slot);
                }
                // Then set the new slot
                if let Some(slot_pos) = msg.slot_position {
                    cache.set_unit_slot(msg.cell, slot_pos, msg.unit_id);
                }
            }
        }
    }
}

fn receive_hamlet_found_error(mut receivers: Query<&mut MessageReceiver<HamletFoundErrorMsg>>) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            warn!("❌ Hamlet founding failed: {}", msg.reason);
        }
    }
}

fn receive_debug_messages(
    mut debug_created: Query<&mut MessageReceiver<DebugOrganizationCreatedMsg>>,
    mut debug_deleted: Query<&mut MessageReceiver<DebugOrganizationDeletedMsg>>,
    mut debug_error: Query<&mut MessageReceiver<DebugErrorMsg>>,
    mut debug_spawned: Query<&mut MessageReceiver<DebugUnitSpawnedMsg>>,
) {
    for mut receiver in debug_created.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Debug: Organization '{}' created (ID: {}) via lightyear",
                msg.name, msg.organization_id
            );
        }
    }
    for mut receiver in debug_deleted.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Debug: Organization {} deleted via lightyear",
                msg.organization_id
            );
        }
    }
    for mut receiver in debug_error.iter_mut() {
        for msg in receiver.receive() {
            warn!("Debug error via lightyear: {}", msg.reason);
        }
    }
    for mut receiver in debug_spawned.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "✓ Debug: Unit spawned: {} via lightyear",
                msg.unit_data.full_name()
            );
        }
    }
}

// ─── Destroy building ──────────────────────────────────────────────

fn send_destroy_building(
    mut events: MessageReader<SendDestroyBuilding>,
    mut senders: Query<&mut MessageSender<DestroyBuildingMsg>>,
) {
    for event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(DestroyBuildingMsg { cell: event.cell });
            break;
        }
    }
}

fn receive_building_destroyed(
    mut receivers: Query<&mut MessageReceiver<BuildingDestroyedMsg>>,
    mut world_cache: Option<ResMut<WorldCache>>,
    mut player_info: ResMut<PlayerInfo>,
    mut commands: Commands,
    buildings: Query<(Entity, &crate::rendering::terrain::components::Building)>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "Building {} destroyed at ({},{}) via lightyear",
                msg.building_id, msg.cell.q, msg.cell.r
            );
            if let Some(ref mut cache) = world_cache {
                cache.remove_building(&msg.cell);
            }
            for (entity, building) in buildings.iter() {
                if building.id == msg.building_id as i64 {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(ref mut org) = player_info.organization {
                org.population = msg.new_population;
                org.population_capacity = msg.population_capacity;
            }
        }
    }
}

// ─── Liquidate organization ────────────────────────────────────────

fn send_liquidate_organization(
    mut events: MessageReader<SendLiquidateOrganization>,
    mut senders: Query<&mut MessageSender<LiquidateOrganizationMsg>>,
) {
    for _event in events.read() {
        if let Some(mut sender) = senders.iter_mut().next() {
            sender.send::<ReliableGameChannel>(LiquidateOrganizationMsg);
            break;
        }
    }
}

fn receive_organization_liquidated(
    mut receivers: Query<&mut MessageReceiver<OrganizationLiquidatedMsg>>,
    mut player_info: ResMut<PlayerInfo>,
    mut commands: Commands,
    contour_entities: Query<(Entity, &crate::rendering::territory::TerritoryContourEntity)>,
    building_entities: Query<(Entity, &crate::rendering::terrain::components::Building)>,
    mut contour_cache: ResMut<crate::rendering::territory::TerritoryContourCache>,
    mut world_cache: Option<ResMut<crate::state::resources::WorldCache>>,
    http_client: Res<HttpBulkClient>,
    http_sender: Res<HttpTerrainSender>,
    mut next_game_view: ResMut<NextState<crate::states::GameView>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "Organization {} liquidated via lightyear ({} chunks affected)",
                msg.organization_id, msg.affected_chunks.len()
            );

            player_info.organization = None;

            // Despawn all territory contour entities for this org
            for (entity, contour) in contour_entities.iter() {
                if contour.organization_id == msg.organization_id {
                    commands.entity(entity).despawn();
                }
            }

            // Clear contour cache for this org
            contour_cache.remove_organization(msg.organization_id);

            // Clear buildings from WorldCache for affected chunks and despawn sprites
            if let Some(ref mut cache) = world_cache {
                // Remove all non-tree buildings from cache
                let cells_to_remove: Vec<_> = cache
                    .loaded_buildings()
                    .filter(|b| b.base_data.category != shared::BuildingCategoryEnum::Natural)
                    .map(|b| b.base_data.cell)
                    .collect();
                for cell in &cells_to_remove {
                    cache.remove_building(cell);
                }
            }

            // Despawn all non-tree building sprites
            for (entity, _building) in building_entities.iter() {
                commands.entity(entity).despawn();
            }

            // Re-fetch affected chunks to get clean state (trees remain, buildings gone)
            for chunk_id in &msg.affected_chunks {
                let sender = http_sender.clone();
                let client = http_client.client.clone();
                let base_url = http_client.base_url.clone();
                let cid = *chunk_id;

                IoTaskPool::get()
                    .spawn(async_compat::Compat::new(async move {
                        let body = serde_json::json!({
                            "terrain_name": "Gaulyia",
                            "chunk_ids": [[cid.x, cid.y]],
                        });
                        match client
                            .post(format!("{}/api/terrain/chunks", base_url))
                            .json(&body)
                            .send()
                            .await
                        {
                            Ok(response) => match response.bytes().await {
                                Ok(bytes) => {
                                    match crate::networking::client::http_client::parse_terrain_response_pub(&bytes) {
                                        Ok(parsed) => { let _ = sender.tx.send(parsed); }
                                        Err(e) => bevy::log::error!("HTTP terrain parse error: {}", e),
                                    }
                                }
                                Err(e) => bevy::log::error!("HTTP terrain response error: {}", e),
                            },
                            Err(e) => bevy::log::error!("HTTP terrain request error: {}", e),
                        }
                    }))
                    .detach();
            }

            // Switch back to map view (closes management panel)
            next_game_view.set(crate::states::GameView::Map);
        }
    }
}
