use std::sync::Arc;

use axum::Router;

use shared::grid::GridConfig;
use shared::GameState;

use crate::database::client::DatabaseTables;
use crate::world::resources::WorldGlobalState;

pub async fn start_http_server(
    db_tables: Arc<DatabaseTables>,
    game_server_addr: std::net::SocketAddr,
    world_global_state: Arc<WorldGlobalState>,
    game_state: Arc<GameState>,
    grid_config: Arc<GridConfig>,
) {
    let port: u16 = std::env::var("AUTH_HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let auth_state = super::auth::AuthState {
        db_tables: db_tables.clone(),
        game_server_addr,
    };

    let bulk_state = super::bulk::BulkState {
        db_tables,
        world_global_state,
        game_state,
        grid_config,
    };

    // Auth routes (own state)
    let auth_router = Router::new()
        .route("/api/auth/login", axum::routing::post(super::auth::login))
        .route("/api/auth/register", axum::routing::post(super::auth::register))
        .with_state(auth_state);

    // Bulk data routes (own state)
    let bulk_router = Router::new()
        .route("/api/terrain/chunks", axum::routing::post(super::bulk::terrain_chunks))
        .route("/api/world/{name}/ocean", axum::routing::get(super::bulk::ocean_data))
        .route("/api/world/{name}/lake", axum::routing::get(super::bulk::lake_data))
        .route("/api/world/{name}/terrain-global", axum::routing::get(super::bulk::terrain_global_data))
        .route("/api/world/{name}/exploration", axum::routing::get(super::bulk::exploration_map))
        .with_state(bulk_state);

    let app = auth_router.merge(bulk_router);

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    tracing::info!("🔐 Auth + Bulk HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
