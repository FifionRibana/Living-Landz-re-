use std::collections::HashMap;

use bevy::prelude::*;

use shared::{BuildingCategory, BuildingTypeData, GameState, TreeType};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct BuildingTypesTable {
    pool: PgPool,
}

impl BuildingTypesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS building_types (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL UNIQUE,
                category_id INT NOT NULL REFERENCES building_categories(id),
                description TEXT
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        tracing::info!("✓ Building types Database schema ready");
        Ok(())
    }

    pub async fn fill(&self, game_state: &GameState) -> Result<Vec<BuildingTypeData>, sqlx::Error> {
        let types = vec![
            (TreeType::Cedar, BuildingCategory::Natural, "A cedar tree"),
            (TreeType::Larch, BuildingCategory::Natural, "A larch tree"),
            (TreeType::Oak, BuildingCategory::Natural, "An oak tree"),
        ];

        let mut tx = self.pool.begin().await?;
        let mut query_builder =
            sqlx::QueryBuilder::new("INSERT INTO building_types (name, category_id, description)");

        query_builder.push_values(types.iter(), |mut b, (name, category, description)| {
            b.push_bind(name.to_name())
                .push_bind(game_state.get_building_category_id(&category.to_name()))
                .push_bind(description);
        });

        query_builder.push(" ON CONFLICT (name) DO NOTHING");
        
        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Building categories content initialized");

        let building_types = sqlx::query(
            r#"
            SELECT bt.id, bt.name, bc.name as category 
            FROM building_types bt
            JOIN building_categories bc ON bt.category_id = bc.id
        "#,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            let id = r.get("id");

            BuildingTypeData {
                id,
                name: r.get("name"),
                category: r.get("category"),
            }
        })
        .collect::<Vec<_>>();

        Ok(building_types)
    }
}
