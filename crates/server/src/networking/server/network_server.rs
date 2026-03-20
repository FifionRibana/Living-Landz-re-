use std::sync::Arc;

use shared::GameState;
use shared::grid::GridConfig;
use tokio::net::TcpListener;

use crate::action_processor::ActionProcessor;
use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;
use crate::units::NameGenerator;
use crate::world::resources::WorldGlobalState;

use super::super::Sessions;
use super::handlers;

pub struct NetworkServer {
    pub address: String,
    pub port: u16,
}

impl NetworkServer {
    pub fn new(address: String, port: u16) -> Self {
        Self { address, port }
    }

    pub async fn start(
        &self,
        sessions: Sessions,
        db_tables: Arc<DatabaseTables>,
        action_processor: Arc<ActionProcessor>,
        name_generator: Arc<NameGenerator>,
        game_state: Arc<GameState>,
        grid_config: Arc<GridConfig>,
        dev_config: Arc<DevConfig>,
        world_global_state: Arc<WorldGlobalState>,
    ) {
        let addr = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Failed to bind server");

        tracing::info!("🌐 Server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            tracing::info!("Accept listeners");
            let sessions_clone = sessions.clone();
            let db_tables_clone = db_tables.clone();
            let action_processor_clone = action_processor.clone();
            let name_generator_clone = name_generator.clone();
            let game_state_clone = game_state.clone();
            let grid_config_clone = grid_config.clone();
            let dev_config_clone = dev_config.clone();
            let world_global_state_clone = world_global_state.clone();

            tokio::spawn(async move {
                tracing::info!("Handle connections...");
                handlers::handle_connection(
                    stream,
                    addr,
                    sessions_clone,
                    db_tables_clone,
                    action_processor_clone,
                    name_generator_clone,
                    game_state_clone,
                    grid_config_clone,
                    dev_config_clone,
                    world_global_state_clone,
                )
                .await;
            });
        }
    }
}

pub fn initialize_server(
    sessions: Sessions,
    db_tables: Arc<DatabaseTables>,
    action_processor: Arc<ActionProcessor>,
    name_generator: Arc<NameGenerator>,
    game_state: Arc<GameState>,
    grid_config: Arc<GridConfig>,
    dev_config: Arc<DevConfig>,
    world_global_state: Arc<WorldGlobalState>,
) {
    tracing::info!("Starting network server...");

    // Normal server startup
    let server_address =
        std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "9001".to_string())
        .parse()
        .unwrap_or(9001);

    let sessions_clone = sessions.clone();
    let db_tables_clone = db_tables.clone();
    let action_processor_clone = action_processor.clone();
    let name_generator_clone = name_generator.clone();
    let game_state_clone = game_state.clone();
    let grid_config_clone = grid_config.clone();
    let dev_config_clone = dev_config.clone();
    let world_global_state_clone = world_global_state.clone();

    tokio::spawn(async move {
        let server = NetworkServer::new(server_address, server_port);
        server
            .start(
                sessions_clone,
                db_tables_clone,
                action_processor_clone,
                name_generator_clone,
                game_state_clone,
                grid_config_clone,
                dev_config_clone,
                world_global_state_clone,
            )
            .await;
    });

    tracing::info!("✓ Network server spawned");
}
