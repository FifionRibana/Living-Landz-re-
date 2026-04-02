use std::sync::Arc;

use axum::Router;

use crate::database::client::DatabaseTables;

pub async fn start_http_server(
    db_tables: Arc<DatabaseTables>,
    game_server_addr: std::net::SocketAddr,
) {
    let port: u16 = std::env::var("AUTH_HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let state = super::auth::AuthState {
        db_tables,
        game_server_addr,
    };

    let app = Router::new()
        .route(
            "/api/auth/login",
            axum::routing::post(super::auth::login),
        )
        .route(
            "/api/auth/register",
            axum::routing::post(super::auth::register),
        )
        .with_state(state);

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    tracing::info!("🔐 Auth HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
