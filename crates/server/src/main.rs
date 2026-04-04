use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use std::fs;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod action_processor;
mod auth;
mod database;
mod dev;
mod exploration;
mod http;
mod networking;
mod population;
mod road;
mod units;
mod utils;
mod world;

// Convenience aliases for the new lightyear module
use networking::server::lightyear::bridge;
use networking::server::lightyear::systems::LightyearGamePlugin;

/// Gives Bevy systems access to the tokio runtime and shared server state.
/// Kept for future use — any Bevy system that needs async DB access can use this.
#[derive(Resource)]
pub struct AsyncBridge {
    pub runtime: tokio::runtime::Handle,
    pub db_tables: Arc<database::client::DatabaseTables>,
    pub game_state: Arc<shared::GameState>,
    pub grid_config: Arc<shared::grid::GridConfig>,
}

fn main() {
// Set up dual logging: console + file (truncated on each launch)
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("server.log")
        .expect("Failed to open server.log");

    tracing_subscriber::registry()
        .with(EnvFilter::new(
            "info,wgpu=error,naga=warn,lightyear=warn,sqlx=warn,hyper=warn",
        ))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(
            fmt::layer()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false),
        )
        .init();

    dotenv::dotenv().ok();

    // Build tokio runtime manually — Bevy owns the main thread,
    // tokio runs tungstenite + DB + action processing in background tasks.
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // === All async initialization inside the runtime ===
    let init = rt.block_on(async {
        // ── Dev config ──
        let dev_config = dev::DevConfig::from_env();
        if dev_config.dev_mode {
            tracing::warn!(
                "⚡ DEV MODE ACTIVE — speed: {}x, bypass resources: {}",
                dev_config.speed_factor,
                dev_config.bypass_resources
            );
        }
        let dev_config_arc = Arc::new(dev_config);

        // ── CLI args ──
        let args: Vec<String> = std::env::args().collect();

        // ── Game seed ──
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

        // ── Database ──
        let (db_tables, game_state) = database::client::initialize_database().await;

        let grid_config = world::systems::setup_grid_config();
        utils::chunks::fix_chunk_assignments(&db_tables.pool, &grid_config.layout).await;
        utils::portraits::fix_avatar_urls(&db_tables.pool).await;

        // ── Map selection ──
        let mut map_name = "test_island".to_string();
        if let Some(arg) = args.iter().find(|arg| arg.starts_with("--map=")) {
            map_name = arg.trim_start_matches("--map=").to_string();
        }
        tracing::info!("Using map: {}", map_name);

        // ── CLI commands (exit after) ──
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

        // ── World globals ──
        tracing::info!("=== Loading World Globals ===");
        let world_global_state =
            world::systems::load_or_generate_world_globals(&map_name, &db_tables).await;
        let world_global_state_arc = Arc::new(world_global_state);
        tracing::info!("✓ World globals loaded");

        // ── Shared state ──
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

        // ── Lightyear bridge: tokio ↔ Bevy ECS ──
        let (lightyear_bridge_res, bridge_sender, action_request_receiver) =
            bridge::create_bridge();
        let bridge_sender_arc = Arc::new(bridge_sender);

        // ── Action processor (receives bridge_sender to push position updates) ──
        let action_processor = Arc::new(action_processor::ActionProcessor::new(
            db_tables_arc.clone(),
            game_state_arc.clone(),
            grid_config_arc.clone(),
            dev_config_arc.clone(),
            bridge_sender_arc.clone(),
        ));

        if let Err(e) = action_processor.load_active_actions().await {
            tracing::error!("Failed to load active actions: {}", e);
        }

        // ── HTTP auth server (shares tokio runtime) ──
        let game_server_addr: std::net::SocketAddr = format!(
            "127.0.0.1:{}",
            std::env::var("LIGHTYEAR_PORT").unwrap_or_else(|_| "5000".to_string())
        )
        .parse()
        .unwrap();
        tokio::spawn(http::start_http_server(
            db_tables_arc.clone(),
            game_server_addr,
        ));

        // ── Background processors ──
        action_processor::start_action_processor(action_processor.clone());

        // Start action RPC handler (Bevy→tokio bridge for lightyear action messages)
        networking::server::lightyear::action_handler::start_action_rpc_handler(
            action_request_receiver,
            bridge_sender_arc.clone(),
            db_tables_arc.clone(),
            action_processor.clone(),
            dev_config_arc.clone(),
            game_state_arc.clone(),
            grid_config_arc.clone(),
            world_global_state_arc.clone(),
        );

        let population_system = Arc::new(population::PopulationSystem::new(
            db_tables_arc.clone(),
            bridge_sender_arc.clone(),
            name_generator.clone(),
        ));
        population::start_population_tick(population_system);

        // ── Return everything Bevy needs ──
        (
            db_tables_arc,
            game_state_arc,
            grid_config_arc,
            lightyear_bridge_res,
        )
    });

    let (db_tables_arc, game_state_arc, grid_config_arc, lightyear_bridge_res) = init;

    // Keep tokio runtime alive — background tasks (action processor,
    // population ticks) must survive beyond this point.
    let rt_handle = rt.handle().clone();
    std::mem::forget(rt);

    // === Build Bevy App with Lightyear ===
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_millis(50),
        })
        .add_plugins(shared::protocol::plugin::ProtocolPlugin)
        .add_plugins(LightyearGamePlugin)
        .insert_resource(lightyear_bridge_res)
        .insert_resource(AsyncBridge {
            runtime: rt_handle,
            db_tables: db_tables_arc,
            game_state: game_state_arc,
            grid_config: grid_config_arc,
        })
        .add_systems(Startup, setup_lightyear_server)
        .run();
}

/// Spawn the lightyear server entity with UDP transport.
fn setup_lightyear_server(mut commands: Commands) {
    let port: u16 = std::env::var("LIGHTYEAR_PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse()
        .unwrap_or(5000);

    let server_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let netcode_cfg = server::NetcodeConfig {
        protocol_id: shared::protocol::netcode_config::PROTOCOL_ID,
        private_key: shared::protocol::netcode_config::private_key(),
        ..Default::default()
    };

    let server = commands
        .spawn((
            NetcodeServer::new(netcode_cfg),
            LocalAddr(server_addr),
            ServerUdpIo::default(),
        ))
        .id();
    commands.trigger(Start { entity: server });

    tracing::info!("🚀 Lightyear server started — UDP:{}", port);
}
