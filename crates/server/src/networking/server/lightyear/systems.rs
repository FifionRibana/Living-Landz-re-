use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lightyear::connection::client::PeerMetadata;
use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use shared::protocol::{
    channels::ReliableGameChannel,
    components::{LordPosition, MovingUnitId, MovingUnitPosition, OwnedByPlayer},
    lightyear_messages::{
        ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
        ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg,
        ActionStatusMsg, ActionTrainUnitMsg, GameDataMsg, LoginSuccessMsg, LordDataMsg,
        PlayerOrganizationDataMsg, UnitPositionUpdatedMsg,
    },
};

use super::bridge::{BridgeEvent, LightyearBridge};
// use crate::lightyear_bridge::{LightyearBridge, BridgeEvent};

// ─── Server-only components (not replicated) ────────────────────────

/// Maps a replicated lord entity back to its DB player_id.
/// NOT replicated — server bookkeeping only.
#[derive(Component)]
pub struct ServerPlayerId(pub u64);

/// Server-only: maps a moving unit entity to its DB unit_id.
/// Used to find the entity when the move completes.
#[derive(Component)]
pub struct ServerMovingUnitId(pub u64);

// ─── Resources ──────────────────────────────────────────────────────

/// Tracks Room entities per chunk coordinate.
/// Key: (chunk_x, chunk_y), Value: Room entity.
#[derive(Resource, Default)]
pub struct ChunkRooms {
    pub rooms: HashMap<(i32, i32), Entity>,
}

/// Maps player_id (DB) → lightyear link entity (the sender).
/// Populated when a lightyear client connects with a known client_id.
#[derive(Resource, Default)]
pub struct PlayerLinkMap {
    pub map: HashMap<u64, Entity>,
}

#[derive(Resource, Default)]
pub struct PendingDespawns(pub Vec<u64>);

// ─── Constants ──────────────────────────────────────────────────────

/// How many chunks around the lord the player can see.
/// Must match client's view_radius.
const ROOM_VIEW_RADIUS: i32 = 4;

// ─── Plugin ─────────────────────────────────────────────────────────

pub struct LightyearGamePlugin;

impl Plugin for LightyearGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RoomPlugin)
            .init_resource::<ChunkRooms>()
            .init_resource::<PlayerLinkMap>()
            .init_resource::<PendingDespawns>()
            .add_observer(on_lightyear_link_created)
            .add_observer(on_lightyear_connected)
            .add_systems(FixedUpdate, poll_bridge_events)
            .add_systems(
                Update,
                (
                    receive_move_unit_messages,
                    receive_build_building_messages,
                    receive_build_road_messages,
                    receive_harvest_resource_messages,
                    receive_craft_resource_messages,
                    receive_train_unit_messages,
                    receive_explore_messages,
                ),
            );
    }
}

// ─── Observers ──────────────────────────────────────────────────────

/// When a new lightyear link is created (client UDP packet received),
/// add a ReplicationSender so we can replicate entities to this client.
fn on_lightyear_link_created(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            std::time::Duration::from_millis(100), // 10Hz replication
            SendUpdatesMode::SinceLastAck,
            false,
        ));
    tracing::info!(
        "✓ Lightyear link created (entity {:?}), ReplicationSender added",
        trigger.entity
    );
}

/// When a lightyear client is confirmed Connected, store the player_id → link mapping
/// and trigger game data loading via the tokio bridge.
fn on_lightyear_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut player_links: ResMut<PlayerLinkMap>,
    lords: Query<(&ServerPlayerId, &LordPosition)>,
    mut commands: Commands,
    mut chunk_rooms: ResMut<ChunkRooms>,
    bridge: Res<LightyearBridge>,
) {
    let Ok(remote_id) = query.get(trigger.entity) else {
        return;
    };
    // remote_id.0 is PeerId::Netcode(client_id) where client_id == player_id
    let player_id = remote_id.0.to_bits();
    player_links.map.insert(player_id, trigger.entity);
    tracing::info!(
        "✓ Lightyear client connected: player_id={} → link entity {:?}",
        player_id,
        trigger.entity
    );

    // If lord was already spawned via tungstenite bridge, join rooms now
    for (pid, pos) in lords.iter() {
        if pid.0 == player_id {
            add_sender_to_nearby_rooms(
                &mut commands,
                &mut chunk_rooms,
                trigger.entity,
                pos.chunk_x,
                pos.chunk_y,
            );
            tracing::info!(
                "🏠 Deferred room join: player {} added to rooms around ({},{})",
                player_id, pos.chunk_x, pos.chunk_y
            );
            break;
        }
    }

    // Trigger game data loading via the tokio bridge
    bridge.send_action(super::bridge::ActionRequest::LoadPlayerData { player_id });
}

// ─── Bridge polling ─────────────────────────────────────────────────

/// Main system: poll bridge events from tokio and apply to ECS.
pub fn poll_bridge_events(
    mut commands: Commands,
    bridge: Res<LightyearBridge>,
    mut lords: Query<(Entity, &ServerPlayerId, &mut LordPosition)>,
    moving_units: Query<(Entity, &ServerMovingUnitId)>,
    mut chunk_rooms: ResMut<ChunkRooms>,
    mut player_links: ResMut<PlayerLinkMap>,
    mut msg_sender: ServerMultiMessageSender,
    server_entity: Option<Single<&Server>>,
    mut pending_despawns: ResMut<PendingDespawns>,
) {
    // Extract the Server entity once before the loop
    let srv = server_entity.map(|s| s.into_inner());

    // Process any despawns that were deferred from the previous tick
    pending_despawns.0.retain(|&unit_id| {
        for (entity, id) in moving_units.iter() {
            if id.0 == unit_id {
                commands.entity(entity).despawn();
                tracing::info!("🚶 Moving unit {} despawned (deferred)", unit_id);
                return false; // Remove from pending
            }
        }
        true // Keep in pending — entity still not spawned (shouldn't happen)
    });

    for event in bridge.drain() {
        match event {
            BridgeEvent::SpawnLord {
                player_id,
                chunk,
                cell,
            } => {
                handle_spawn_lord(
                    &mut commands,
                    &lords,
                    &mut chunk_rooms,
                    &player_links,
                    player_id,
                    chunk,
                    cell,
                );
            }

            BridgeEvent::UpdateLordPosition {
                player_id,
                to_chunk,
                to_cell,
            } => {
                handle_update_lord_position(
                    &mut commands,
                    &mut lords,
                    &mut chunk_rooms,
                    &player_links,
                    player_id,
                    to_chunk,
                    to_cell,
                );
            }

            BridgeEvent::DespawnLord { player_id } => {
                handle_despawn_lord(
                    &mut commands,
                    &lords,
                    &chunk_rooms,
                    &mut player_links,
                    player_id,
                );
            }

            BridgeEvent::SendActionStatus {
                player_id,
                action_id,
                chunk_id,
                cell,
                status,
                action_type,
                completion_time,
                action_name,
                unit_ids,
            } => {
                let Some(srv) = srv else {
                    continue;
                };
                let target = NetworkTarget::Single(PeerId::Netcode(player_id));
                let msg = ActionStatusMsg {
                    action_id,
                    player_id,
                    chunk_id,
                    cell,
                    status,
                    action_type,
                    completion_time,
                    action_name,
                    unit_ids,
                };
                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target) {
                    tracing::error!(
                        "Failed to send ActionStatusMsg to player {}: {:?}",
                        player_id,
                        e
                    );
                }
            }

            BridgeEvent::SendActionError { player_id, reason } => {
                let Some(srv) = srv else {
                    continue;
                };
                let target = NetworkTarget::Single(PeerId::Netcode(player_id));
                let msg = ActionErrorMsg {
                    reason: reason.clone(),
                };
                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target) {
                    tracing::error!(
                        "Failed to send ActionErrorMsg to player {}: {:?}",
                        player_id,
                        e
                    );
                }
            }

            BridgeEvent::SpawnMovingUnit {
                player_id,
                unit_id,
                chunk,
                cell,
            } => {
                // Don't spawn if already exists
                if moving_units.iter().any(|(_, id)| id.0 == unit_id) {
                    tracing::warn!("Moving unit {} already exists, skipping", unit_id);
                    continue;
                }

                let entity = commands
                    .spawn((
                        Name::new(format!("MovingUnit_{}", unit_id)),
                        ServerMovingUnitId(unit_id),
                        MovingUnitPosition {
                            chunk_x: chunk.x,
                            chunk_y: chunk.y,
                            cell_q: cell.q,
                            cell_r: cell.r,
                        },
                        MovingUnitId(unit_id),
                        OwnedByPlayer(player_id),
                        Replicate::to_clients(NetworkTarget::All),
                        NetworkVisibility,
                    ))
                    .id();

                // Add to chunk room
                let room =
                    get_or_create_chunk_room(&mut commands, &mut chunk_rooms, chunk.x, chunk.y);
                commands.trigger(RoomEvent {
                    target: RoomTarget::AddEntity(entity),
                    room,
                });

                tracing::info!(
                    "🚶 Moving unit {} spawned (entity {:?}) at chunk ({},{}) cell ({},{})",
                    unit_id,
                    entity,
                    chunk.x,
                    chunk.y,
                    cell.q,
                    cell.r
                );
            }

            BridgeEvent::DespawnMovingUnit { unit_id } => {
                let mut found = false;
                for (entity, id) in moving_units.iter() {
                    if id.0 == unit_id {
                        commands.entity(entity).despawn();
                        tracing::info!("🚶 Moving unit {} despawned", unit_id);
                        found = true;
                        break;
                    }
                }
                if !found {
                    // Entity not yet in ECS (spawned this same tick) — defer to next tick
                    tracing::info!(
                        "🚶 Moving unit {} despawn deferred (entity not yet spawned)",
                        unit_id
                    );
                    pending_despawns.0.push(unit_id);
                }
            }

            BridgeEvent::SendUnitPositionUpdated {
                player_id,
                unit_id,
                from_cell,
                from_chunk,
                to_cell,
                to_chunk,
            } => {
                let Some(srv) = srv else {
                    continue;
                };
                let target = NetworkTarget::Single(PeerId::Netcode(player_id));
                let msg = UnitPositionUpdatedMsg {
                    unit_id,
                    from_cell,
                    from_chunk,
                    to_cell,
                    to_chunk,
                };
                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target) {
                    tracing::error!(
                        "Failed to send UnitPositionUpdatedMsg to player {}: {:?}",
                        player_id, e
                    );
                }
            }

            BridgeEvent::BroadcastActionCompleted {
                action_id,
                chunk_id,
                cell,
                action_type,
            } => {
                let Some(srv) = srv else {
                    continue;
                };
                let target = NetworkTarget::All;
                let msg = ActionCompletedMsg {
                    action_id,
                    chunk_id,
                    cell,
                    action_type,
                };
                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target) {
                    tracing::error!(
                        "Failed to broadcast ActionCompletedMsg for action {}: {:?}",
                        action_id, e
                    );
                }
            }

            BridgeEvent::SendLoginData {
                player_id,
                player,
                character,
                lord,
                organization,
                game_data,
            } => {
                let Some(srv) = srv else {
                    continue;
                };
                let target = NetworkTarget::Single(PeerId::Netcode(player_id));

                // Send in order: LoginSuccess → LordData → Organization → GameData
                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(
                    &LoginSuccessMsg {
                        player: player.clone(),
                        character: character.clone(),
                    },
                    srv,
                    &target,
                ) {
                    tracing::error!(
                        "Failed to send LoginSuccessMsg to player {}: {:?}",
                        player_id, e
                    );
                }

                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(
                    &LordDataMsg { lord: lord.clone() },
                    srv,
                    &target,
                ) {
                    tracing::error!(
                        "Failed to send LordDataMsg to player {}: {:?}",
                        player_id, e
                    );
                }

                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(
                    &PlayerOrganizationDataMsg {
                        organization: organization.clone(),
                    },
                    srv,
                    &target,
                ) {
                    tracing::error!(
                        "Failed to send PlayerOrganizationDataMsg to player {}: {:?}",
                        player_id, e
                    );
                }

                if let Err(e) = msg_sender.send::<_, ReliableGameChannel>(
                    &GameDataMsg {
                        payload: game_data.clone(),
                    },
                    srv,
                    &target,
                ) {
                    tracing::error!(
                        "Failed to send GameDataMsg to player {}: {:?}",
                        player_id, e
                    );
                }

                tracing::info!(
                    "📦 Sent login data to player {} via lightyear",
                    player_id
                );
            }
        }
    }
}

// ─── Event handlers ─────────────────────────────────────────────────

fn handle_spawn_lord(
    commands: &mut Commands,
    lords: &Query<(Entity, &ServerPlayerId, &mut LordPosition)>,
    chunk_rooms: &mut ChunkRooms,
    player_links: &PlayerLinkMap,
    player_id: u64,
    chunk: shared::TerrainChunkId,
    cell: shared::grid::GridCell,
) {
    // Don't spawn twice
    if lords.iter().any(|(_, pid, _)| pid.0 == player_id) {
        tracing::warn!(
            "Lord already exists for player {}, skipping spawn",
            player_id
        );
        return;
    }

    // Spawn the replicated lord entity
    let lord_entity = commands
        .spawn((
            Name::new(format!("Lord_P{}", player_id)),
            ServerPlayerId(player_id),
            LordPosition {
                chunk_x: chunk.x,
                chunk_y: chunk.y,
                cell_q: cell.q,
                cell_r: cell.r,
            },
            OwnedByPlayer(player_id),
            // Replicate to all clients + enable room-based visibility
            Replicate::to_clients(NetworkTarget::All),
            NetworkVisibility,
        ))
        .id();

    // Add lord entity to its chunk room
    let room = get_or_create_chunk_room(commands, chunk_rooms, chunk.x, chunk.y);
    commands.trigger(RoomEvent {
        target: RoomTarget::AddEntity(lord_entity),
        room,
    });

    // Add the player's link (sender) to surrounding chunk rooms
    if let Some(&link_entity) = player_links.map.get(&player_id) {
        add_sender_to_nearby_rooms(commands, chunk_rooms, link_entity, chunk.x, chunk.y);
    } else {
        // Lightyear client may not be connected yet — that's OK.
        // The rooms will be joined when the client connects and we receive a Connected event.
        tracing::debug!(
            "Player {} lord spawned but lightyear link not yet connected — rooms deferred",
            player_id
        );
    }

    tracing::info!(
        "🏰 Lord spawned for player {} (entity {:?}) at chunk ({},{}) cell ({},{})",
        player_id,
        lord_entity,
        chunk.x,
        chunk.y,
        cell.q,
        cell.r
    );
}

fn handle_update_lord_position(
    commands: &mut Commands,
    lords: &mut Query<(Entity, &ServerPlayerId, &mut LordPosition)>,
    chunk_rooms: &mut ChunkRooms,
    player_links: &PlayerLinkMap,
    player_id: u64,
    to_chunk: shared::TerrainChunkId,
    to_cell: shared::grid::GridCell,
) {
    let mut found = false;

    for (lord_entity, pid, mut pos) in lords.iter_mut() {
        if pid.0 != player_id {
            continue;
        }

        let old_chunk_x = pos.chunk_x;
        let old_chunk_y = pos.chunk_y;

        // Update the position — lightyear detects the change and replicates
        pos.chunk_x = to_chunk.x;
        pos.chunk_y = to_chunk.y;
        pos.cell_q = to_cell.q;
        pos.cell_r = to_cell.r;

        // If chunk changed, move entity and sender between rooms
        if old_chunk_x != to_chunk.x || old_chunk_y != to_chunk.y {
            // Remove lord from old room, add to new room
            if let Some(&old_room) = chunk_rooms.rooms.get(&(old_chunk_x, old_chunk_y)) {
                commands.trigger(RoomEvent {
                    target: RoomTarget::RemoveEntity(lord_entity),
                    room: old_room,
                });
            }
            let new_room = get_or_create_chunk_room(commands, chunk_rooms, to_chunk.x, to_chunk.y);
            commands.trigger(RoomEvent {
                target: RoomTarget::AddEntity(lord_entity),
                room: new_room,
            });

            // Update sender rooms (remove from old radius, add in new radius)
            if let Some(&link_entity) = player_links.map.get(&player_id) {
                remove_sender_from_all_rooms(commands, chunk_rooms, link_entity);
                add_sender_to_nearby_rooms(
                    commands,
                    chunk_rooms,
                    link_entity,
                    to_chunk.x,
                    to_chunk.y,
                );
            }
        }

        tracing::info!(
            "📍 Lord {} moved to chunk ({},{}) cell ({},{})",
            player_id,
            to_chunk.x,
            to_chunk.y,
            to_cell.q,
            to_cell.r
        );
        found = true;
        break;
    }

    if !found {
        tracing::warn!(
            "No lord entity found for player {} — position update lost",
            player_id
        );
    }
}

fn handle_despawn_lord(
    commands: &mut Commands,
    lords: &Query<(Entity, &ServerPlayerId, &mut LordPosition)>,
    chunk_rooms: &ChunkRooms,
    player_links: &mut PlayerLinkMap,
    player_id: u64,
) {
    for (entity, pid, _) in lords.iter() {
        if pid.0 == player_id {
            commands.entity(entity).despawn();
            tracing::info!("🏰 Lord despawned for player {}", player_id);
            break;
        }
    }

    // Remove sender from all rooms AND clean up the mapping
    if let Some(link_entity) = player_links.map.remove(&player_id) {
        remove_sender_from_all_rooms(commands, chunk_rooms, link_entity);
        tracing::info!(
            "🧹 Cleaned up PlayerLinkMap for player {} (link {:?})",
            player_id,
            link_entity
        );
    }
}

// ─── Room helpers ───────────────────────────────────────────────────

/// Get an existing Room for this chunk, or create one.
fn get_or_create_chunk_room(
    commands: &mut Commands,
    chunk_rooms: &mut ChunkRooms,
    chunk_x: i32,
    chunk_y: i32,
) -> Entity {
    *chunk_rooms
        .rooms
        .entry((chunk_x, chunk_y))
        .or_insert_with(|| {
            let room = commands
                .spawn((
                    Room::default(),
                    Name::new(format!("Room_{}_{}", chunk_x, chunk_y)),
                ))
                .id();
            tracing::trace!("Created chunk room ({},{}) → {:?}", chunk_x, chunk_y, room);
            room
        })
}

/// Add a sender (client link entity) to all chunk rooms within ROOM_VIEW_RADIUS.
fn add_sender_to_nearby_rooms(
    commands: &mut Commands,
    chunk_rooms: &mut ChunkRooms,
    link_entity: Entity,
    center_x: i32,
    center_y: i32,
) {
    for dx in -ROOM_VIEW_RADIUS..=ROOM_VIEW_RADIUS {
        for dy in -ROOM_VIEW_RADIUS..=ROOM_VIEW_RADIUS {
            let room =
                get_or_create_chunk_room(commands, chunk_rooms, center_x + dx, center_y + dy);
            commands.trigger(RoomEvent {
                target: RoomTarget::AddSender(link_entity),
                room,
            });
        }
    }
}

/// Remove a sender from ALL rooms it's in.
fn remove_sender_from_all_rooms(
    commands: &mut Commands,
    chunk_rooms: &ChunkRooms,
    link_entity: Entity,
) {
    for &room in chunk_rooms.rooms.values() {
        commands.trigger(RoomEvent {
            target: RoomTarget::RemoveSender(link_entity),
            room,
        });
    }
}

// ─── Receive lightyear action messages ────────────────────────────────

fn receive_move_unit_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionMoveUnitMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionMoveUnit from player {}: unit {} → ({},{})",
                player_id, msg.unit_id, msg.cell.q, msg.cell.r
            );
            bridge.send_action(super::bridge::ActionRequest::MoveUnit {
                player_id,
                unit_id: msg.unit_id,
                chunk_id: msg.chunk_id,
                cell: msg.cell,
            });
        }
    }
}

fn receive_build_building_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionBuildBuildingMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionBuildBuilding from player {}: {:?} at ({},{})",
                player_id, msg.building_type, msg.cell.q, msg.cell.r
            );
            bridge.send_action(super::bridge::ActionRequest::BuildBuilding {
                player_id,
                chunk_id: msg.chunk_id,
                cell: msg.cell,
                building_type: msg.building_type,
            });
        }
    }
}

fn receive_build_road_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionBuildRoadMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionBuildRoad from player {}: ({},{}) → ({},{})",
                player_id, msg.start_cell.q, msg.start_cell.r, msg.end_cell.q, msg.end_cell.r
            );
            bridge.send_action(super::bridge::ActionRequest::BuildRoad {
                player_id,
                start_cell: msg.start_cell,
                end_cell: msg.end_cell,
            });
        }
    }
}

fn receive_harvest_resource_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionHarvestResourceMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionHarvestResource from player {}: {:?} at ({},{})",
                player_id, msg.resource_specific_type, msg.cell.q, msg.cell.r
            );
            bridge.send_action(super::bridge::ActionRequest::HarvestResource {
                player_id,
                chunk_id: msg.chunk_id,
                cell: msg.cell,
                resource_specific_type: msg.resource_specific_type,
                unit_ids: msg.unit_ids,
            });
        }
    }
}

fn receive_craft_resource_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionCraftResourceMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionCraftResource from player {}: recipe '{}' x{} at ({},{})",
                player_id, msg.recipe_id, msg.quantity, msg.cell.q, msg.cell.r
            );
            bridge.send_action(super::bridge::ActionRequest::CraftResource {
                player_id,
                chunk_id: msg.chunk_id,
                cell: msg.cell,
                recipe_id: msg.recipe_id,
                quantity: msg.quantity,
                unit_ids: msg.unit_ids,
            });
        }
    }
}

fn receive_train_unit_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionTrainUnitMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionTrainUnit from player {}: unit {} → {:?}",
                player_id, msg.unit_id, msg.target_profession
            );
            bridge.send_action(super::bridge::ActionRequest::TrainUnit {
                player_id,
                unit_id: msg.unit_id,
                chunk_id: msg.chunk_id,
                cell: msg.cell,
                target_profession: msg.target_profession,
            });
        }
    }
}

fn receive_explore_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionExploreMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();
        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionExplore from player {}: ({},{}) radius {}",
                player_id, msg.cell.q, msg.cell.r, msg.radius
            );
            bridge.send_action(super::bridge::ActionRequest::Explore {
                player_id,
                cell: msg.cell,
                radius: msg.radius,
            });
        }
    }
}
