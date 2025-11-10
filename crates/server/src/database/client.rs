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
    pub buildings: tables::BuildingsTable,
    pub cells: tables::CellsTable,
    pub terrains: tables::TerrainsTable,
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
        let mut game_state = GameState::default();

        let database_url = format!(
            "{}://{}:{}@{}/{}",
            self.protocol, credentials.username, credentials.password, self.address, self.name
        );
        tracing::info!("Connecting to database at {}", database_url);

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        // === TYPES ===
        let building_categories_db = tables::types::BuildingCategoriesTable::new(pool.clone());
        building_categories_db
            .init_schema()
            .await
            .expect("Failed to init building categories database table");

        // let building_categories = building_categories_db
        //     .fill()
        //     .await
        //     .expect("Failed to fill building categories database table");

        // game_state.building_categories.extend(
        //     building_categories
        //         .iter()
        //         .map(|building_category| (building_category.id, building_category.clone())),
        // );

        let building_types_db = tables::types::BuildingTypesTable::new(pool.clone());
        building_types_db
            .init_schema()
            .await
            .expect("Failed to init building types database table");

        let building_types = building_types_db
            .fill()
            .await
            .expect("Failed to fill building types database table");

        game_state.building_types.extend(
            building_types
                .iter()
                .map(|building_type| (building_type.id, building_type.clone())),
        );

        let resource_types_db = tables::types::ResourceTypesTable::new(pool.clone());
        resource_types_db
            .init_schema()
            .await
            .expect("Failed to init resource types database table");

        // Data tables
        let buildings_db = tables::BuildingsTable::new(pool.clone());
        buildings_db
            .init_schema()
            .await
            .expect("Failed to init buildings database table");

        let terrain_db = tables::TerrainsTable::new(pool.clone());
        terrain_db
            .init_schema()
            .await
            .expect("Failed to init terrains database table");

        let cell_db = tables::CellsTable::new(pool.clone());
        cell_db
            .init_schema()
            .await
            .expect("Failed to init cells database table");

        tracing::info!("✓ Database connected");

        (
            DatabaseTables {
                buildings: buildings_db,
                cells: cell_db,
                terrains: terrain_db,
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
