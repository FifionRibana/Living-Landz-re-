use bevy::prelude::*;

use shared::{BuildingCategory, BuildingCategoryData, GameState};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct BuildingCategoriesTable {
    pool: PgPool,
}

impl BuildingCategoriesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS building_categories (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL UNIQUE
            )"#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("✓ Building categories Database schema ready");
        Ok(())
    }

    pub async fn fill(&self) -> Result<Vec<BuildingCategoryData>, sqlx::Error> {
        let categories = [
            BuildingCategory::Natural,
            BuildingCategory::Infrastructure,
            BuildingCategory::Structure,
            BuildingCategory::Defense,
        ]
        .into_iter()
        .map(|c| c.to_name())
        .collect::<Vec<String>>();

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new("INSERT INTO building_categories (name)");

        query_builder.push_values(categories.iter(), |mut b, category| {
            b.push_bind(category);
        });
        
        query_builder.push(" ON CONFLICT (name) DO NOTHING");

        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Building categories content initialized");

        let building_categories = sqlx::query("SELECT id, name FROM building_categories")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r: sqlx::postgres::PgRow| {
                let id = r.get("id");
                BuildingCategoryData {
                    id,
                    name: r.get("name"),
                }
            })
            .collect::<Vec<_>>();
        Ok(building_categories)
    }
}
