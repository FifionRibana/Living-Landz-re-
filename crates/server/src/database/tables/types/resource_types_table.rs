use bevy::prelude::*;

use shared::{ResourceCategory, ResourceSpecificType, ResourceTypeData};
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
            r#"
            CREATE TYPE resource_category AS ENUM (
                'Unknown',
                'Wood',
                'Metal',
                'CrudeMaterial',
                'Food',
                'Furniture',
                'Weaponry',
                'Jewelry',
                'Meat',
                'Fruits',
                'Vegetables'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TYPE resource_specific_type AS ENUM (
                'Cedar',
                'Unknown'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS resource_types (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL UNIQUE,
                category resource_category NOT NULL,
                specific_type resource_specific_type NOT NULL,
                description TEXT
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        tracing::info!("✓ Resource types Database schema ready");
        Ok(())
    }

    pub async fn fill(&self) -> Result<Vec<ResourceTypeData>, sqlx::Error> {
        let types = vec![(
            "cedar",
            ResourceCategory::Wood,
            ResourceSpecificType::Wood,
            "Cedar wood",
        ),
        (
            "iron",
            ResourceCategory::Metal,
            ResourceSpecificType::Metal,
            "iron ingots"
        ),
        (
            "iron ore",
            ResourceCategory::CrudeMaterial,
            ResourceSpecificType::Ore,
            "iron ore"
        ),(
            "granite",
            ResourceCategory::CrudeMaterial,
            ResourceSpecificType::Rock,
            "granite block"
        ),(
            "clay",
            ResourceCategory::CrudeMaterial,
            ResourceSpecificType::Rock,
            "clay block"
        )];

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO resource_types (name, category, specific_type, description)",
        );

        query_builder.push_values(
            types.iter(),
            |mut b, (name, category, specific_type, description)| {
                b.push_bind(name)
                    .push_bind(category)
                    .push_bind(specific_type)
                    .push_bind(description);
            },
        );

        query_builder.push(" ON CONFLICT (name, category, specific_type) DO NOTHING");

        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Resource types content initialized");

        let resource_types = sqlx::query(
            r#"
            SELECT id, name, category, specific_type, description
            FROM resource_types
        "#,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            let id = r.get("id");

            ResourceTypeData {
                id,
                name: r.get("name"),
                category: r.get("category"),
                specific_type: r.get("specific_type"),
                description: r.get("description"),
            }
        })
        .collect::<Vec<_>>();

        Ok(resource_types)
    }
}
