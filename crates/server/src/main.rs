use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::protocol::components::{LordPosition, OwnedByPlayer};

mod action_processor;
mod auth;
mod database;
mod dev;
mod exploration;
mod networking;
mod population;
mod road;
mod units;
mod utils;
mod world;

// 🔧 LIGHTYEAR: Bridge resource — gives Bevy systems access to the tokio world
#[derive(Resource)]
pub struct AsyncBridge {
    pub runtime: tokio::runtime::Handle,
    pub db_tables: Arc<database::client::DatabaseTables>,
    pub game_state: Arc<shared::GameState>,
    pub sessions: networking::Sessions,
    pub grid_config: Arc<shared::grid::GridConfig>,
}

// 🔧 LIGHTYEAR: No more #[tokio::main] — we own the runtime manually
fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    // Build tokio runtime manually
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // === Run all async init inside the runtime ===
    let init = rt.block_on(async {
        let dev_config = dev::DevConfig::from_env();
        if dev_config.dev_mode {
            tracing::warn!(
                "⚡ DEV MODE ACTIVE — speed: {}x, bypass resources: {}",
                dev_config.speed_factor,
                dev_config.bypass_resources
            );
        }
        let dev_config_arc = Arc::new(dev_config);

        let args: Vec<String> = std::env::args().collect();

        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let seed_dir = manifest_dir.join("../../tools/game_seed");

        if !seed_dir.exists() {
            tracing::warn!(
                "Seed directory not found at {}, skipping game-seed",
                seed_dir.display()
            );
        } else {
            let status = std::process::Command::new("uv")
                .args(["run", "game-seed"])
                .current_dir(&seed_dir)
                .status()
                .expect("Failed to run game-seed");

            if !status.success() {
                tracing::error!("Game seed failed with status {}", status);
                std::process::exit(1);
            }
        }

        let (db_tables, game_state) = database::client::initialize_database().await;

        let grid_config = world::systems::setup_grid_config();
        utils::chunks::fix_chunk_assignments(&db_tables.pool, &grid_config.layout).await;
        utils::portraits::fix_avatar_urls(&db_tables.pool).await;

        let mut map_name = "test_island".to_string();

        if let Some(arg) = args.iter().find(|arg| arg.starts_with("--map=")) {
            map_name = arg.trim_start_matches("--map=").to_string();
        }
        tracing::info!("Using map: {}", map_name);

        // Handle CLI commands (exit after)
        if args.contains(&"--clear".to_string()) {
            tracing::info!("=== Starting World Cleaning ===");
            world::systems::clear_world(&map_name, &db_tables).await;
            tracing::info!("=== Cleaning Complete - Exiting ===");
            std::process::exit(0);
        } else if args.contains(&"--save-png".to_string()) {
            tracing::info!("=== Starting Map png saving ===");
            world::systems::save_world_to_png(&map_name).await;
            tracing::info!("=== Saving Complete - Exiting ===");
            std::process::exit(0);
        } else if args.contains(&"--generate-world".to_string()) {
            tracing::info!("=== Starting World Generation ===");
            world::systems::generate_world(&map_name, &db_tables, &game_state).await;
            tracing::info!("=== Generation Complete - Exiting ===");
            std::process::exit(0);
        } else if args.contains(&"--generate-globals".to_string()) {
            tracing::info!("=== Loading World Globals ===");
            world::systems::generate_world_globals(&map_name, &db_tables).await;
            std::process::exit(0);
        } else if args.contains(&"--regen-territory".to_string()) {
            tracing::info!("=== Starting Territory Contours Regeneration ===");
            world::systems::regenerate_territory_contours(&db_tables).await;
            tracing::info!("=== Regeneration Complete - Exiting ===");
            std::process::exit(0);
        }

        // Load world globals
        tracing::info!("=== Loading World Globals ===");
        let world_global_state =
            world::systems::load_or_generate_world_globals(&map_name, &db_tables).await;
        let world_global_state_arc = Arc::new(world_global_state);
        tracing::info!("✓ World globals loaded");

        let sessions = networking::Sessions::default();
        let db_tables_arc = Arc::new(db_tables);
        let game_state_arc = Arc::new(game_state);
        let grid_config_arc = Arc::new(grid_config);

        let name_generator = match units::NameGenerator::load_from_files() {
            Ok(generator) => Arc::new(generator),
            Err(e) => {
                tracing::error!("Failed to load name generator: {}", e);
                std::process::exit(1);
            }
        };

        let action_processor = Arc::new(action_processor::ActionProcessor::new(
            db_tables_arc.clone(),
            sessions.clone(),
            game_state_arc.clone(),
            grid_config_arc.clone(),
            dev_config_arc.clone(),
        ));

        if let Err(e) = action_processor.load_active_actions().await {
            tracing::error!("Failed to load active actions: {}", e);
        }

        // === Start tungstenite server in background (existing system, still alive) ===
        networking::server::initialize_server(
            sessions.clone(),
            db_tables_arc.clone(),
            action_processor.clone(),
            name_generator.clone(),
            game_state_arc.clone(),
            grid_config_arc.clone(),
            dev_config_arc.clone(),
            world_global_state_arc.clone(),
        );

        // Start action processor in background
        action_processor::start_action_processor(action_processor.clone());

        // Start population system in background
        let population_system = Arc::new(population::PopulationSystem::new(
            db_tables_arc.clone(),
            sessions.clone(),
            name_generator.clone(),
        ));
        population::start_population_tick(population_system);

        // Return what Bevy needs
        (db_tables_arc, game_state_arc, sessions, grid_config_arc)
    });

    let (db_tables_arc, game_state_arc, sessions, grid_config_arc) = init;

    // 🔧 LIGHTYEAR: Keep tokio runtime alive by leaking it into a 'static handle
    // The runtime must outlive the Bevy App
    let rt_handle = rt.handle().clone();
    std::mem::forget(rt); // Leak the runtime so background tasks keep running

    // === Build Bevy App with Lightyear ===
    App::new()
        .add_plugins(MinimalPlugins)
        // 🔧 LIGHTYEAR: Server plugins at 20Hz tick rate
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_millis(50),
        })
        // 🔧 LIGHTYEAR: Shared protocol (components, messages, channels)
        .add_plugins(shared::protocol::plugin::ProtocolPlugin)
        // Bridge to async world
        .insert_resource(AsyncBridge {
            runtime: rt_handle,
            db_tables: db_tables_arc,
            game_state: game_state_arc,
            sessions: sessions.clone(),
            grid_config: grid_config_arc,
        })
        .insert_resource(sessions)
        // 🔧 LIGHTYEAR: Server setup + replication systems
        .add_systems(Startup, setup_lightyear_server)
        .add_observer(handle_new_lightyear_client)
        .add_systems(FixedUpdate, sync_lord_positions)
        .run();
}

// 🔧 LIGHTYEAR: Spawn server entity with WebTransport + WebSocket listeners
use lightyear::websocket::server::ServerConfig as WsServerConfig;

fn setup_lightyear_server(mut commands: Commands) {
    let port: u16 = std::env::var("LIGHTYEAR_PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse()
        .unwrap_or(5000);

    let server_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let server = commands
        .spawn((
            NetcodeServer::new(server::NetcodeConfig::default()),
            LocalAddr(server_addr),
            ServerUdpIo::default(),
        ))
        .id();
    commands.trigger(Start { entity: server });

    tracing::info!("🚀 Lightyear server started — UDP:{}", port);
}

/// When a client connects, add ReplicationSender on the link entity
fn handle_new_lightyear_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            std::time::Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
        ));
    tracing::info!("✓ Lightyear client link created, ReplicationSender added");
}

// 🔧 LIGHTYEAR: Placeholder — will sync lord positions as replicated entities
fn sync_lord_positions(
    _bridge: Res<AsyncBridge>,
    _commands: Commands,
    // Query for connected clients, spawn/update replicated Lord entities
) {
    // Phase 1: on client connect → load lord from DB → spawn replicated entity
    // Phase 2: on lord move → update LordPosition component → lightyear replicates
}
