use super::TerrainDatabase;

pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
}

pub struct DatabaseClient {
    pub protocol: String,
    pub address: String,
    pub name: String,
}

impl DatabaseClient {
    pub async fn new(protocol: &str, address: &str, name: &str) -> Self {
        Self {
            protocol: protocol.to_string(),
            address: address.to_string(),
            name: name.to_string(),
        }
    }

    pub async fn connect(
        &self,
        credentials: &DatabaseCredentials,
    ) -> TerrainDatabase {
        let database_url = format!(
            "{}://{}:{}@{}/{}",
            self.protocol, credentials.username, credentials.password, self.address, self.name
        );
        tracing::info!("Connecting to database at {}", database_url);

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let terrain_db = TerrainDatabase::new(pool.clone());
        terrain_db
            .init_schema()
            .await
            .expect("Failed to init terrain database schema");

        tracing::info!("✓ Database connected");
        terrain_db
    }
}

pub async fn initialize_database() -> TerrainDatabase {
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

    let terrain_db: TerrainDatabase =
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let db_credentials = DatabaseCredentials {
                    username: user,
                    password,
                };

                let db_client = DatabaseClient::new(
                    &protocol,
                    format!("{}:{}", host, port).as_str(),
                    &db_name,
                )
                .await;

                let db = db_client.connect(&db_credentials).await;

                db
            })
        });

    tracing::info!("✓ Database client ready");
    terrain_db
}
