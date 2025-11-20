use bevy::prelude::*;

use shared::{BuildingCategoryEnum, BuildingSpecificTypeEnum};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct BuildingTypesTable {
    pool: PgPool,
}

impl BuildingTypesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize_buildings(&self) -> Result<(), sqlx::Error> {
        let types = vec![
            ("Cedar", BuildingCategoryEnum::Natural, BuildingSpecificTypeEnum::Tree, "A cedar tree"),
            ("Larch", BuildingCategoryEnum::Natural, BuildingSpecificTypeEnum::Tree, "A larch tree"),
            ("Oak", BuildingCategoryEnum::Natural, BuildingSpecificTypeEnum::Tree, "An oak tree"),
            ("Blacksmith", BuildingCategoryEnum::ManufacturingWorkshops, BuildingSpecificTypeEnum::ManufacturingWorkshop, "A blacksmith workshop"),
        ];

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO buildings.building_types (name, category_id, specific_type_id, description)"
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

        let result = query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!("✓ Building types initialized ({} rows)", result.rows_affected());
        Ok(())
    }
}
