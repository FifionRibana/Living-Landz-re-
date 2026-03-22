use bevy::prelude::*;
use shared::LakeData;
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct LakeDataTable {
    pool: PgPool,
}

impl LakeDataTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_lake_data(&self, lake_data: LakeData) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO terrain.lake_data (name, width, height, mask_data, sdf_width, sdf_height, sdf_data, world_width, world_height, generated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (name)
            DO UPDATE SET
                width = $2, height = $3, mask_data = $4,
                sdf_width = $5, sdf_height = $6, sdf_data = $7,
                world_width = $8, world_height = $9, generated_at = $10
            "#,
        )
        .bind(&lake_data.name)
        .bind(lake_data.width as i32)
        .bind(lake_data.height as i32)
        .bind(&lake_data.mask_values)
        .bind(lake_data.sdf_width as i32)
        .bind(lake_data.sdf_height as i32)
        .bind(&lake_data.sdf_values)
        .bind(lake_data.world_width)
        .bind(lake_data.world_height)
        .bind(lake_data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_lake_data(&self, name: &str) -> Result<Option<LakeData>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT width, height, mask_data, sdf_width, sdf_height, sdf_data, world_width, world_height, generated_at
             FROM terrain.lake_data
             WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| LakeData {
            name: name.to_string(),
            width: r.get::<i32, _>("width") as usize,
            height: r.get::<i32, _>("height") as usize,
            mask_values: r.get("mask_data"),
            sdf_width: r.get::<i32, _>("sdf_width") as usize,
            sdf_height: r.get::<i32, _>("sdf_height") as usize,
            sdf_values: r.get("sdf_data"),
            world_width: r.get("world_width"),
            world_height: r.get("world_height"),
            generated_at: r.get::<i64, _>("generated_at") as u64,
        }))
    }
}