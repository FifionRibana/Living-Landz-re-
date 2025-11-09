use bevy::prelude::*;

use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct ResourceTypesTable {
    pool: PgPool,
}

impl ResourceTypesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS resource_types (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL UNIQUE
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        tracing::info!("✓ Resource types Database schema ready");
        Ok(())
    }
}
