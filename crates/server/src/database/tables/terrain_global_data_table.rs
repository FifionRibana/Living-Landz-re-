use bevy::prelude::*;
use shared::TerrainGlobalData;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Resource, Clone)]
pub struct TerrainGlobalDataTable {
    pool: PgPool,
}

impl TerrainGlobalDataTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_terrain_global_data(
        &self,
        data: TerrainGlobalData,
    ) -> Result<(), sqlx::Error> {
        let bytes = bincode::encode_to_vec(&data, bincode::config::standard())
            .expect("Failed to encode terrain global data");

        sqlx::query(
            r#"
            INSERT INTO terrain.terrain_global_data (name, data, generated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (name)
            DO UPDATE SET data = $2, generated_at = $3
            "#,
        )
        .bind(&data.name)
        .bind(&bytes)
        .bind(data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_terrain_global_data(
        &self,
        name: &str,
    ) -> Result<Option<TerrainGlobalData>, sqlx::Error> {
        let row =
            sqlx::query("SELECT data FROM terrain.terrain_global_data WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.and_then(|r| {
            let bytes: Vec<u8> = r.get("data");
            match bincode::decode_from_slice(&bytes[..], bincode::config::standard()) {
                Ok((data, _)) => Some(data),
                Err(e) => {
                    tracing::warn!(
                        "Failed to deserialize terrain global data for '{}': {}",
                        name,
                        e
                    );
                    None
                }
            }
        }))
    }

    pub async fn clear_terrain_global_data(&self, name: &str) -> Result<(), sqlx::Error> {
        tracing::warn!("🗑️  Clearing {} terrain global data from database...", name);

        sqlx::query("DELETE FROM terrain.terrain_global_data WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Terrain global data cleared");
        Ok(())
    }

    pub async fn save_sdf_global_data(
        &self,
        data: shared::SdfGlobalData,
    ) -> Result<(), sqlx::Error> {
        let bytes = bincode::encode_to_vec(&data, bincode::config::standard())
            .expect("Failed to encode SDF global data");

        sqlx::query(
            r#"
            INSERT INTO terrain.sdf_global_data (name, data, generated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (name)
            DO UPDATE SET data = $2, generated_at = $3
            "#,
        )
        .bind(&data.name)
        .bind(&bytes)
        .bind(data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_sdf_global_data(
        &self,
        name: &str,
    ) -> Result<Option<shared::SdfGlobalData>, sqlx::Error> {
        let row =
            sqlx::query("SELECT data FROM terrain.sdf_global_data WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.and_then(|r| {
            let bytes: Vec<u8> = r.get("data");
            match bincode::decode_from_slice(&bytes[..], bincode::config::standard()) {
                Ok((data, _)) => Some(data),
                Err(e) => {
                    tracing::warn!(
                        "Failed to deserialize SDF global data for '{}': {}",
                        name,
                        e
                    );
                    None
                }
            }
        }))
    }
}
