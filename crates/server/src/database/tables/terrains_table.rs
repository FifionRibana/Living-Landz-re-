use bevy::prelude::*;

use shared::BiomeChunkData;
use shared::BiomeChunkId;
use shared::TerrainChunkData;
use shared::TerrainChunkId;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Resource, Clone)]
pub struct TerrainsTable {
    pool: PgPool,
}

impl TerrainsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TYPE biome_type AS ENUM (
                'Ocean',
                'DeepOcean',
                'Desert',
                'Savanna',
                'Grassland',
                'TropicalSeasonalForest',
                'TropicalRainForest',
                'TropicalDeciduousForest',
                'TemperateRainForest',
                'Wetland',
                'Taiga',
                'Tundra',
                'Lake',
                'ColdDesert',
                'Ice'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS terrains (
                name VARCHAR(32) NOT NULL,
                chunk_x INT NOT NULL,
                chunk_y INT NOT NULL,
                data BYTEA NOT NULL,
                generated_at BIGINT NOT NULL,
                PRIMARY KEY (name, chunk_x, chunk_y)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_terrains_generated ON terrains(generated_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS terrain_biomes (
                name VARCHAR(32) NOT NULL,
                chunk_x INT NOT NULL,
                chunk_y INT NOT NULL,
                biome biome_type NOT NULL,
                data BYTEA NOT NULL,
                generated_at BIGINT NOT NULL,
                PRIMARY KEY (name, chunk_x, chunk_y, biome)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_terrain_biomes_generated ON terrain_biomes(generated_at)")
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Terrains Database schema ready");
        Ok(())
    }

    pub async fn save_terrain(&self, terrain_data: TerrainChunkData) -> Result<(), sqlx::Error> {
        let terrain_bytes = bincode::encode_to_vec(&terrain_data, bincode::config::standard())
            .expect("Failed to encode terrain data");

        sqlx::query(
            r#"
            INSERT INTO terrains (name, chunk_x, chunk_y, data, generated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (name, chunk_x, chunk_y)
            DO UPDATE SET data = $4, generated_at = $5
            "#,
        )
        .bind(terrain_data.name)
        .bind(terrain_data.id.x)
        .bind(terrain_data.id.y)
        .bind(&terrain_bytes)
        .bind(terrain_data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_terrain(
        &self,
        name: &str,
        terrain_chunk_id: &TerrainChunkId,
    ) -> Result<(Option<TerrainChunkData>, Option<Vec<BiomeChunkData>>), sqlx::Error> {
        let row = sqlx::query("SELECT data, generated_at FROM terrains WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3")
            .bind(name)
            .bind(terrain_chunk_id.x)
            .bind(terrain_chunk_id.y)
            .fetch_optional(&self.pool)
            .await?;

        let terrain_row = row.map(|r| {
            let terrain_bytes: Vec<u8> = r.get("data");
            let (terrain_data, _) =
                bincode::decode_from_slice(&terrain_bytes[..], bincode::config::standard())
                    .expect("Failed to deserialize terrain data");

            terrain_data
        });

        let biome_rows = sqlx::query(
            "SELECT biome, data, generated_at FROM terrain_biomes WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3"
        )
            .bind(name)
            .bind(terrain_chunk_id.x)
            .bind(terrain_chunk_id.y)
            .fetch_all(&self.pool)
            .await?;

        let biomes = biome_rows.iter().map(|r| {
            let biome_bytes: Vec<u8> = r.get("data");
            let (biome_data, _) =
                bincode::decode_from_slice(&biome_bytes[..], bincode::config::standard())
                    .expect("Failed to deserialize terrain data");

            Some(biome_data)
        }).collect();

        Ok((terrain_row, biomes))
    }

    pub async fn clear_terrain(&self, name: &str) -> Result<(), sqlx::Error> {
        tracing::warn!("🗑️  Clearing {} terrain from database...", name);

        sqlx::query("DELETE FROM terrains WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Database cleared");
        Ok(())
    }

    pub async fn save_terrain_biome(
        &self,
        terrain_biome_data: BiomeChunkData,
    ) -> Result<(), sqlx::Error> {
        let terrain_biome_bytes =
            bincode::encode_to_vec(&terrain_biome_data, bincode::config::standard())
                .expect("Failed to encode terrain data");

        sqlx::query(
            r#"
            INSERT INTO terrain_biomes (name, chunk_x, chunk_y, biome, data, generated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, chunk_x, chunk_y, biome)
            DO UPDATE SET data = $5, generated_at = $6
            "#,
        )
        .bind(terrain_biome_data.name)
        .bind(terrain_biome_data.id.x)
        .bind(terrain_biome_data.id.y)
        .bind(terrain_biome_data.id.biome)
        .bind(&terrain_biome_bytes)
        .bind(terrain_biome_data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_terrain_biome(
        &self,
        name: &str,
        biome_chunk_id: &BiomeChunkId,
    ) -> Result<Option<BiomeChunkData>, sqlx::Error> {
        let row = sqlx::query("SELECT data, generated_at FROM terrain_biomes WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3 AND biome = $4")
            .bind(name)
            .bind(biome_chunk_id.x)
            .bind(biome_chunk_id.y)
            .bind(biome_chunk_id.biome)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            let terrain_bytes: Vec<u8> = r.get("data");
            let (terrain_data, _) =
                bincode::decode_from_slice(&terrain_bytes[..], bincode::config::standard())
                    .expect("Failed to deserialize terrain biome data");

            terrain_data
        }))
    }

    pub async fn clear_terrain_biome(&self, name: &str) -> Result<(), sqlx::Error> {
        tracing::warn!("🗑️  Clearing {} terrain from database...", name);

        sqlx::query("DELETE FROM terrain_biomes WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Database cleared");
        Ok(())
    }
}
