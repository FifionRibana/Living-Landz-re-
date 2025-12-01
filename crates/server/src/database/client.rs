use shared::GameState;

use super::tables;

pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
}

pub struct DatabaseClient {
    pub protocol: String,
    pub address: String,
    pub name: String,
}

pub struct DatabaseTables {
    pub pool: sqlx::PgPool,
    pub actions: tables::ScheduledActionsTable,
    pub buildings: tables::BuildingsTable,
    pub cells: tables::CellsTable,
    pub terrains: tables::TerrainsTable,
    pub ocean_data: tables::OceanDataTable,
}

impl DatabaseClient {
    pub async fn new(protocol: &str, address: &str, name: &str) -> Self {
        Self {
            protocol: protocol.to_string(),
            address: address.to_string(),
            name: name.to_string(),
        }
    }

    pub async fn connect(&self, credentials: &DatabaseCredentials) -> (DatabaseTables, GameState) {
        let database_url = format!(
            "{}://{}:{}@{}/{}",
            self.protocol, credentials.username, credentials.password, self.address, self.name
        );
        tracing::info!("Connecting to database at {}", database_url);

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        tracing::info!("✓ Database connected");

        // === TYPES ===
        let building_types_db = tables::types::BuildingTypesTable::new(pool.clone());
        building_types_db
            .initialize_buildings()
            .await
            .expect("Failed to init building types database table");

        let resource_types_db = tables::types::ResourceTypesTable::new(pool.clone());
        resource_types_db
            .initialize_resources()
            .await
            .expect("Failed to init resource types database table");

        tracing::info!("✓ Database types initialized ");

        // ===== GAME STATE =====
        let mut game_state = GameState::new(pool.clone());
        game_state.initialize_caches().await.expect("Failed to initialize game cache");
        tracing::info!("✓ Game state cache initialized");

        (
            DatabaseTables {
                pool: pool.clone(),
                actions: tables::ScheduledActionsTable::new(pool.clone()),
                buildings: tables::BuildingsTable::new(pool.clone()),
                cells: tables::CellsTable::new(pool.clone()),
                terrains: tables::TerrainsTable::new(pool.clone()),
                ocean_data: tables::OceanDataTable::new(pool.clone()),
            },
            game_state,
        )
    }
}

pub async fn initialize_database() -> (DatabaseTables, GameState) {
    tracing::info!("Setting up database client...");

    let protocol = std::env::var("DB_PROTOCOL").unwrap_or_else(|_| "postgres".to_string());
    let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = std::env::var("DB_PORT")
        .unwrap_or_else(|_| "5432".to_string())
        .parse()
        .unwrap_or(5432);
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "living_landz".to_string());
    let user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());

    // let db_url = format!(
    //     "{}://{}:{}@{}:{}/{}",
    //     protocol, user, password, host, port, db_name
    // );

    tracing::info!(
        "Connecting to database at {}://{}:{}/{}",
        protocol,
        host,
        port,
        db_name
    );

    let (db_tables, game_state): (DatabaseTables, GameState) = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let db_credentials = DatabaseCredentials {
                username: user,
                password,
            };

            let db_client =
                DatabaseClient::new(&protocol, format!("{}:{}", host, port).as_str(), &db_name)
                    .await;

            let (db, gs) = db_client.connect(&db_credentials).await;

            (db, gs)
        })
    });

    (db_tables, game_state)
}
