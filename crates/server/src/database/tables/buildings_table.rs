use bevy::prelude::*;

use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct BuildingsTable {
    pool: PgPool,
}

impl BuildingsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buildings (
                id BIGINT PRIMARY KEY,
                building_type_id INT NOT NULL REFERENCES building_types(id),

                quality FLOAT NOT NULL DEFAULT 1.0,
                durability FLOAT NOT NULL DEFAULT 1.0,
                damage FLOAT NOT NULL DEFAULT 0.0
            );"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_buildings_type ON buildings(building_type_id)")
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Buildings Database schema ready");
        Ok(())
    }
}
