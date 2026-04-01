use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::state::resources::{ConnectionStatus, PlayerInfo, UnitsCache, UnitsDataCache};
use crate::states::AppState;
use shared::protocol::components::{LordPosition, OwnedByPlayer};

pub struct LightyearClientPlugin;

impl Plugin for LightyearClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            connect_to_server.run_if(
                in_state(AppState::InGame)
                    .and(run_once)
            ),
        )
        .add_systems(
            Update,
            (handle_connection_events, handle_lord_replication)
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
        private_key: Key::default(),
        protocol_id: 0,
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

        let new_cell = shared::grid::GridCell { q: pos.cell_q, r: pos.cell_r };
        let new_chunk = shared::TerrainChunkId { x: pos.chunk_x, y: pos.chunk_y };

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
        if let Some(ref mut data_cache) = units_data_cache {
            if let Some(unit) = data_cache.get_unit_mut(unit_id) {
                unit.current_cell = new_cell;
                unit.current_chunk = new_chunk;
            }
        }
    }
}