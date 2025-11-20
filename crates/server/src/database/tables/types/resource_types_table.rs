use bevy::prelude::*;

use shared::{ResourceCategoryEnum, ResourceSpecificTypeEnum};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct ResourceTypesTable {
    pool: PgPool,
}

impl ResourceTypesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize_resources(&self) -> Result<(), sqlx::Error> {
        let types = vec![
            ("Cedar", ResourceCategoryEnum::Wood, ResourceSpecificTypeEnum::Wood, "Cedar wood"),
            ("Iron", ResourceCategoryEnum::Metal, ResourceSpecificTypeEnum::Metal, "Iron ingots"),
            ("Iron ore", ResourceCategoryEnum::CrudeMaterial, ResourceSpecificTypeEnum::Ore, "Iron ore"),
            ("Granite", ResourceCategoryEnum::CrudeMaterial, ResourceSpecificTypeEnum::Mineral, "Granite block"),
            ("Clay", ResourceCategoryEnum::CrudeMaterial, ResourceSpecificTypeEnum::Mineral, "Clay block")
        ];

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO resources.resource_types (name, category_id, specific_type_id, description)",
        );

        query_builder.push_values(
            types.iter(),
            |mut b, (name, category, specific_type, description)| {
                b.push_bind(name)
                    .push_bind(category.to_id())
                    .push_bind(specific_type.to_id())
                    .push_bind(description);
            },
        );

        query_builder.push(" ON CONFLICT (name, category_id, specific_type_id) DO NOTHING");

        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Resource types content initialized");
        Ok(())
    }
}
