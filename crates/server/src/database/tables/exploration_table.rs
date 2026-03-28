use bevy::prelude::*;
use shared::TerrainChunkId;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Resource, Clone)]
pub struct ExplorationTable {
    pool: PgPool,
}

impl ExplorationTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mark_explored(
        &self,
        chunks: &[TerrainChunkId],
        player_id: i64,
    ) -> Result<Vec<TerrainChunkId>, sqlx::Error> {
        let mut newly_explored = Vec::new();

        for chunk in chunks {
            let result = sqlx::query(
                r#"
                INSERT INTO terrain.explored_chunks (chunk_x, chunk_y, explored_by)
                VALUES ($1, $2, $3)
                ON CONFLICT (chunk_x, chunk_y) DO NOTHING
                "#,
            )
            .bind(chunk.x)
            .bind(chunk.y)
            .bind(player_id)
            .execute(&self.pool)
            .await?;

            if result.rows_affected() > 0 {
                newly_explored.push(*chunk);
            }
        }

        Ok(newly_explored)
    }

    pub async fn load_exploration_map(
        &self,
        n_chunk_x: i32,
        n_chunk_y: i32,
    ) -> Result<Vec<u8>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT chunk_x, chunk_y FROM terrain.explored_chunks"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut data = vec![0u8; (n_chunk_x * n_chunk_y) as usize];

        for row in rows {
            let cx: i32 = row.get("chunk_x");
            let cy: i32 = row.get("chunk_y");
            if cx >= 0 && cx < n_chunk_x && cy >= 0 && cy < n_chunk_y {
                data[(cy * n_chunk_x + cx) as usize] = 255;
            }
        }

        Ok(data)
    }

    pub async fn is_explored(&self, chunk: &TerrainChunkId) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM terrain.explored_chunks WHERE chunk_x = $1 AND chunk_y = $2)"
        )
        .bind(chunk.x)
        .bind(chunk.y)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get(0))
    }
}