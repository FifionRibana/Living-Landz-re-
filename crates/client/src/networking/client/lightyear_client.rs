use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::TerrainChunkId;
use shared::grid::GridCell;
use shared::protocol::channels::ReliableGameChannel;

use crate::networking::client::NetworkClient;
use crate::state::resources::{
    ActionTracker, ConnectionStatus, NotificationState, PlayerInfo, TrackedAction, UnitsCache,
    UnitsDataCache,
};
use crate::states::AppState;
use shared::protocol::components::{LordPosition, MovingUnitId, MovingUnitPosition, OwnedByPlayer};
use shared::protocol::lightyear_messages::{
    ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
    ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg,
    ActionStatusMsg, ActionTrainUnitMsg, UnitPositionUpdatedMsg,
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

pub struct LightyearClientPlugin;

impl Plugin for LightyearClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            connect_to_server.run_if(in_state(AppState::InGame).and(run_once)),
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
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// Connect to lightyear server once we enter InGame state.
/// Uses the player_id from tungstenite login as the netcode client_id,
/// so the server can map PeerId::Netcode(client_id) back to the DB player_id.
fn connect_to_server(mut commands: Commands, connection: Res<ConnectionStatus>) {
    // player_id is set during tungstenite login (LoginSuccess handler)
    let Some(player_id) = connection.player_id else {
        warn!("Cannot connect to lightyear: no player_id yet (login not complete?)");
        return;
    };

    let server_addr: std::net::SocketAddr = std::env::var("LIGHTYEAR_SERVER")
        .unwrap_or_else(|_| "127.0.0.1:5000".to_string())
        .parse()
        .unwrap();

    let auth = Authentication::Manual {
        server_addr,
        client_id: player_id, // player_id from tungstenite login
        private_key: shared::protocol::netcode_config::private_key(),
        protocol_id: shared::protocol::netcode_config::PROTOCOL_ID,
    };

    let client = commands
        .spawn((
            Client::default(),
            LocalAddr("127.0.0.1:0".parse().unwrap()),
            PeerAddr(server_addr),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .id();
    commands.trigger(Connect { entity: client });

    info!(
        "🔌 Connecting to lightyear server at {} with player_id={}",
        server_addr, player_id
    );
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
/// Triggers a chunk data refresh so the client sees new buildings/roads/etc.
fn receive_action_completed(
    mut receivers: Query<&mut MessageReceiver<ActionCompletedMsg>>,
    mut notifications: ResMut<NotificationState>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            info!(
                "📨 Action {} completed at chunk ({},{}) cell ({},{}) (via lightyear)",
                msg.action_id, msg.chunk_id.x, msg.chunk_id.y, msg.cell.q, msg.cell.r
            );

            notifications.push_success(format!("{} terminée !", msg.action_type.to_name()));

            // Request chunk data refresh so the client sees the result
            if let Some(ref mut client) = network_client {
                info!(
                    "Requesting chunk data refresh for ({},{})",
                    msg.chunk_id.x, msg.chunk_id.y
                );
                client.send_message(shared::protocol::ClientMessage::RequestTerrainChunks {
                    terrain_name: "Gaulyia".to_string(),
                    terrain_chunk_ids: vec![msg.chunk_id],
                });
            }
        }
    }
}

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
            info!("📤 Sent ActionCraftResource via lightyear: recipe '{}'", event.recipe_id);
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
            info!("📤 Sent ActionTrainUnit via lightyear for unit {}", event.unit_id);
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
