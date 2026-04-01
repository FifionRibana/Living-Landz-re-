use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lightyear::connection::client::PeerMetadata;
use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use shared::protocol::{
    channels::ReliableGameChannel,
    components::{LordPosition, OwnedByPlayer},
    lightyear_messages::{ActionErrorMsg, ActionMoveUnitMsg, ActionStatusMsg},
};

use super::bridge::{BridgeEvent, LightyearBridge};
// use crate::lightyear_bridge::{LightyearBridge, BridgeEvent};

// ─── Server-only components (not replicated) ────────────────────────

/// Maps a replicated lord entity back to its DB player_id.
/// NOT replicated — server bookkeeping only.
#[derive(Component)]
pub struct ServerPlayerId(pub u64);

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
            .add_observer(on_lightyear_link_created)
            .add_observer(on_lightyear_connected)
            .add_systems(FixedUpdate, poll_bridge_events)
            .add_systems(Update, receive_action_messages);
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

/// When a lightyear client is confirmed Connected, store the player_id → link mapping.
/// The client connects with client_id = player_id (set after tungstenite login).
fn on_lightyear_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut player_links: ResMut<PlayerLinkMap>,
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
}

// ─── Bridge polling ─────────────────────────────────────────────────

/// Main system: poll bridge events from tokio and apply to ECS.
pub fn poll_bridge_events(
    mut commands: Commands,
    bridge: Res<LightyearBridge>,
    mut lords: Query<(Entity, &ServerPlayerId, &mut LordPosition)>,
    mut chunk_rooms: ResMut<ChunkRooms>,
    mut player_links: ResMut<PlayerLinkMap>,
    mut msg_sender: ServerMultiMessageSender,
    server_entity: Option<Single<&Server>>,
) {
    // Extract the Server entity once before the loop
    let srv = server_entity.map(|s| s.into_inner());

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
                let Some(srv) = srv else { continue; };
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
                if let Err(e) =
                    msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target)
                {
                    tracing::error!(
                        "Failed to send ActionStatusMsg to player {}: {:?}",
                        player_id,
                        e
                    );
                }
            }

            BridgeEvent::SendActionError { player_id, reason } => {
                let Some(srv) = srv else { continue; };
                let target = NetworkTarget::Single(PeerId::Netcode(player_id));
                let msg = ActionErrorMsg {
                    reason: reason.clone(),
                };
                if let Err(e) =
                    msg_sender.send::<_, ReliableGameChannel>(&msg, srv, &target)
                {
                    tracing::error!(
                        "Failed to send ActionErrorMsg to player {}: {:?}",
                        player_id,
                        e
                    );
                }
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

// Receive ActionMoveUnit messages from lightyear clients,
/// resolve the player_id from the link entity, and push to the tokio handler.
fn receive_action_messages(
    mut receivers: Query<(Entity, &mut MessageReceiver<ActionMoveUnitMsg>, &RemoteId)>,
    bridge: Res<LightyearBridge>,
) {
    for (_entity, mut receiver, remote_id) in receivers.iter_mut() {
        let player_id = remote_id.0.to_bits();

        for msg in receiver.receive() {
            tracing::info!(
                "📨 Received ActionMoveUnit from player {}: unit {} → ({},{})",
                player_id,
                msg.unit_id,
                msg.cell.q,
                msg.cell.r
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
