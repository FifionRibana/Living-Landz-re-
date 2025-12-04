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

    pub async fn save_terrain(&self, terrain_data: TerrainChunkData) -> Result<(), sqlx::Error> {
        let terrain_bytes = bincode::encode_to_vec(&terrain_data, bincode::config::standard())
            .expect("Failed to encode terrain data");

        sqlx::query(
            r#"
            INSERT INTO terrain.terrains (name, chunk_x, chunk_y, data, generated_at)
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
        let row = sqlx::query("SELECT data, generated_at FROM terrain.terrains WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3")
            .bind(name)
            .bind(terrain_chunk_id.x)
            .bind(terrain_chunk_id.y)
            .fetch_optional(&self.pool)
            .await?;

        let terrain_row = row.and_then(|r| {
            let terrain_bytes: Vec<u8> = r.get("data");
            match bincode::decode_from_slice(&terrain_bytes[..], bincode::config::standard()) {
                Ok((terrain_data, _)) => Some(terrain_data),
                Err(e) => {
                    tracing::warn!(
                        "Failed to deserialize terrain chunk ({},{}) for '{}': {}. \
                        This is likely due to a schema change. Consider clearing and regenerating the terrain.",
                        terrain_chunk_id.x, terrain_chunk_id.y, name, e
                    );
                    None
                }
            }
        });

        let biome_rows = sqlx::query(
            "SELECT biome_id, data, generated_at FROM terrain.terrain_biomes WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3"
        )
            .bind(name)
            .bind(terrain_chunk_id.x)
            .bind(terrain_chunk_id.y)
            .fetch_all(&self.pool)
            .await?;

        let biomes: Vec<BiomeChunkData> = biome_rows.iter().filter_map(|r| {
            let biome_bytes: Vec<u8> = r.get("data");
            match bincode::decode_from_slice(&biome_bytes[..], bincode::config::standard()) {
                Ok((biome_data, _)) => Some(biome_data),
                Err(e) => {
                    tracing::warn!(
                        "Failed to deserialize biome chunk ({},{}) for '{}': {}. Skipping.",
                        terrain_chunk_id.x, terrain_chunk_id.y, name, e
                    );
                    None
                }
            }
        }).collect();

        let biomes_option = if biomes.is_empty() {
            None
        } else {
            Some(biomes)
        };

        Ok((terrain_row, biomes_option))
    }

    pub async fn clear_terrain(&self, name: &str) -> Result<(), sqlx::Error> {
        tracing::warn!("🗑️  Clearing {} terrain from database...", name);

        sqlx::query("DELETE FROM terrain.terrains WHERE name = $1")
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
            INSERT INTO terrain.terrain_biomes (name, chunk_x, chunk_y, biome_id, data, generated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, chunk_x, chunk_y, biome_id)
            DO UPDATE SET data = $5, generated_at = $6
            "#,
        )
        .bind(terrain_biome_data.name)
        .bind(terrain_biome_data.id.x)
        .bind(terrain_biome_data.id.y)
        .bind(terrain_biome_data.id.biome.to_id())
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
        let row = sqlx::query("SELECT data, generated_at FROM terrain.terrain_biomes WHERE name = $1 AND chunk_x = $2 AND chunk_y = $3 AND biome_id = $4")
            .bind(name)
            .bind(biome_chunk_id.x)
            .bind(biome_chunk_id.y)
            .bind(biome_chunk_id.biome.to_id())
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

        sqlx::query("DELETE FROM terrain.terrain_biomes WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Database cleared");
        Ok(())
    }
}
