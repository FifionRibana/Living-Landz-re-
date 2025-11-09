use bevy::prelude::*;

use shared::{
    BuildingCategory, BuildingData, BuildingType, TerrainChunkId,
    grid::{CellData, GridCell},
};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct BuildingsTable {
    pool: PgPool,
}

impl BuildingsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buildings (
                id BIGINT PRIMARY KEY,
                building_type_id INT NOT NULL REFERENCES building_types(id),
                variant VARCHAR NOT NULL,
                
                chunk_x INT NOT NULL,
                chunk_y INT NOT NULL,
                cell_q INT NOT NULL,
                cell_r INT NOT NULL,

                quality FLOAT NOT NULL DEFAULT 1.0,
                durability FLOAT NOT NULL DEFAULT 1.0,
                damage FLOAT NOT NULL DEFAULT 1.0,

                created_at BIGINT NOT NULL,

                UNIQUE(cell_q, cell_r),
                UNIQUE(cell_q, cell_r, chunk_x, chunk_y)
            );"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_buildings_type ON buildings(building_type_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_buildings_chunk ON buildings(chunk_x, chunk_y)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_buildings_created ON buildings(created_at)")
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Buildings Database schema ready");
        Ok(())
    }

    pub async fn save_buildings(&self, buildings: &[BuildingData]) -> Result<(), sqlx::Error> {
        const BATCH_SIZE: usize = 1000;

        let chunks: Vec<_> = buildings.chunks(BATCH_SIZE).collect();
        println!(
            "Inserting {} buildings in {} chunks",
            buildings.len(),
            chunks.len()
        );
        // Écrire les données

        let mut tx = self.pool.begin().await?;
        for chunk in chunks {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO buildings (id, building_type_id, variant, chunk_x, chunk_y, cell_q, cell_r, created_at)",
            );

            query_builder.push_values(chunk.iter(), |mut b, building| {
                b.push_bind(building.id as i64)
                    .push_bind(building.building_type.id)
                    .push_bind(building.building_type.variant.clone())
                    .push_bind(building.chunk.x)
                    .push_bind(building.chunk.y)
                    .push_bind(building.cell.q)
                    .push_bind(building.cell.r)
                    .push_bind(building.created_at as i64);
            });

            query_builder.push(
                r#"
                    ON CONFLICT (cell_q, cell_r)
                    DO UPDATE SET
                        building_type_id = EXCLUDED.building_type_id,
                        variant = EXCLUDED.variant,
                        quality = EXCLUDED.quality,
                        durability = EXCLUDED.durability,
                        damage = EXCLUDED.damage
                "#,
            );

            query_builder.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn load_chunk_buildings(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<Vec<BuildingData>, sqlx::Error> {
        let row = sqlx::query(r#"
            SELECT b.id, b.cell_q, b.cell_r, b.building_type_id, b.variant, b.created_at, bt.name as category
            FROM buildings b
            JOIN building_types bt ON b.building_type_id = bt.id
            WHERE chunk_x = $1 AND chunk_y = $2
        "#)
            .bind(chunk_id.x)
            .bind(chunk_id.y)
            .fetch_all(&self.pool)
            .await?;

        let buildings = row
            .iter()
            .map(|r| BuildingData {
                id: (r.get::<i64, &str>("id") as i64) as u64,
                building_type: BuildingType {
                    id: r.get("building_type_id"),
                    category: BuildingCategory::from_str(r.get("category")),
                    variant: r.get("variant"),
                },
                cell: GridCell {
                    q: r.get("cell_q"),
                    r: r.get("cell_r"),
                },
                chunk: chunk_id.clone(),
                created_at: (r.get::<i64, &str>("created_at") as i64) as u64,
            })
            .collect::<Vec<_>>();

        Ok(buildings)
    }
}
