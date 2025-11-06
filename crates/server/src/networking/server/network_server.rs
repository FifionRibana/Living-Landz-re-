use std::sync::Arc;

use tokio::net::TcpListener;

use crate::database::client::DatabaseTables;

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

    pub async fn start(&self, sessions: Sessions, db_tables: Arc<DatabaseTables>) {
        let addr = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Failed to bind server");

        tracing::info!("🌐 Server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            tracing::info!("Accept listeners");
            let sessions_clone = sessions.clone();
            let db_tables_clone = db_tables.clone();

            tokio::spawn(async move {
                tracing::info!("Handle connections...");
                handlers::handle_connection(stream, addr, sessions_clone, db_tables_clone).await;
            });
        }
    }
}

pub fn initialize_server(sessions: Sessions, db_tables: Arc<DatabaseTables>) {
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

    tokio::spawn(async move {
        let server = NetworkServer::new(server_address, server_port);
        server.start(sessions_clone, db_tables_clone).await;
    });

    tracing::info!("✓ Network server spawned");
}
