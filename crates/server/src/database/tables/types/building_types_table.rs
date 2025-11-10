use std::collections::HashMap;

use bevy::prelude::*;

use shared::{BuildingCategory, BuildingSpecificType, BuildingTypeData, GameState, TreeType};
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
            r#"
            CREATE TYPE building_category AS ENUM (
                'Unknown',
                'Natural',
                'Structure',
                'Infrastructure',
                'Defense'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TYPE building_specific_type AS ENUM (
                'Tree',
                'Unknown'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TYPE tree_type AS ENUM (
                'Cedar',
                'Larch',
                'Oak'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS building_types (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL,
                category building_category NOT NULL,
                specific_type building_specific_type NOT NULL,
                description TEXT,

                UNIQUE(name, category, specific_type)
            )"#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("✓ Building types Database schema ready");
        Ok(())
    }

    pub async fn fill(&self) -> Result<Vec<BuildingTypeData>, sqlx::Error> {
        let types = vec![
            (
                TreeType::Cedar,
                BuildingCategory::Natural,
                BuildingSpecificType::Tree,
                "A cedar tree",
            ),
            (
                TreeType::Larch,
                BuildingCategory::Natural,
                BuildingSpecificType::Tree,
                "A larch tree",
            ),
            (
                TreeType::Oak,
                BuildingCategory::Natural,
                BuildingSpecificType::Tree,
                "An oak tree",
            ),
        ];

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO building_types (name, category, specific_type, description)",
        );

        query_builder.push_values(
            types.iter(),
            |mut b, (name, category, specific_type, description)| {
                b.push_bind(name.to_name())
                    .push_bind(category)
                    .push_bind(specific_type)
                    .push_bind(description);
            },
        );

        query_builder.push(" ON CONFLICT (name, category, specific_type) DO NOTHING");

        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Building categories content initialized");

        let building_types = sqlx::query(
            r#"
            SELECT id, name, category, specific_type, description
            FROM building_types
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
                specific_type: r.get("specific_type"),
                description: r.get("description"),
            }
        })
        .collect::<Vec<_>>();

        Ok(building_types)
    }
}
