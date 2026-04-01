use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use lightyear::netcode::Key;

use crate::states::AppState;
use shared::protocol::components::{LordPosition, OwnedByPlayer};

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
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Connect to lightyear server once we enter InGame state
fn connect_to_server(mut commands: Commands) {
    let server_addr: std::net::SocketAddr = std::env::var("LIGHTYEAR_SERVER")
        .unwrap_or_else(|_| "127.0.0.1:5000".to_string())
        .parse()
        .unwrap();

    let client_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();

    let auth = Authentication::Manual {
        server_addr,
        client_id: 0,
        private_key: Key::default(),
        protocol_id: 0,
    };

    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(client_addr),
            PeerAddr(server_addr),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .id();
    commands.trigger(Connect { entity: client });

    info!("🔌 Connecting to lightyear server at {}", server_addr);
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

/// React to lord position changes replicated from server
fn handle_lord_replication(
    lords: Query<(&OwnedByPlayer, &LordPosition), Changed<LordPosition>>,
) {
    for (owner, pos) in lords.iter() {
        info!(
            "📍 Lord {} replicated at chunk ({},{}) cell ({},{})",
            owner.0, pos.chunk_x, pos.chunk_y, pos.cell_q, pos.cell_r
        );
    }
}