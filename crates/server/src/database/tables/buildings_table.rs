use bevy::prelude::*;

use shared::{
    BuildingBaseData, BuildingCategory, BuildingData, BuildingSpecific, BuildingSpecificType, BuildingType, TerrainChunkId, TreeData, TreeType, grid::{CellData, GridCell}
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
            CREATE TABLE IF NOT EXISTS buildings_base (
                id BIGINT PRIMARY KEY,
                building_type_id SERIAL REFERENCES building_types(id),
                category building_category NOT NULL,
                
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

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_building_type_id ON buildings_base(building_type_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_buildings_chunk ON buildings_base(chunk_x, chunk_y)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_buildings_created ON buildings_base(created_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trees (
                building_id BIGINT PRIMARY KEY REFERENCES buildings_base(id) ON DELETE CASCADE,
                tree_type tree_type NOT NULL,
                density INT NOT NULL,
                age INT NOT NULL,
                variant INT NOT NULL
            );"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_buildings ON trees(building_id)")
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
                "INSERT INTO buildings_base (id, building_type_id, category, chunk_x, chunk_y, cell_q, cell_r, created_at, quality, durability, damage)",
            );

            query_builder.push_values(chunk.iter(), |mut b, building| {
                let building_data = &building.base_data;
                b.push_bind(building_data.id as i64)
                    .push_bind(building_data.building_type.id)
                    .push_bind(building_data.building_type.category)
                    .push_bind(building_data.chunk.x)
                    .push_bind(building_data.chunk.y)
                    .push_bind(building_data.cell.q)
                    .push_bind(building_data.cell.r)
                    .push_bind(building_data.created_at as i64)
                    .push_bind(building_data.quality)
                    .push_bind(building_data.durability)
                    .push_bind(building_data.damage);
            });

            query_builder.push(
                r#"
                    ON CONFLICT (cell_q, cell_r)
                    DO UPDATE SET
                        id = EXCLUDED.id,
                        category = EXCLUDED.category,
                        quality = EXCLUDED.quality,
                        durability = EXCLUDED.durability,
                        damage = EXCLUDED.damage,
                        created_at = EXCLUDED.created_at
                "#,
            );

            query_builder.build().execute(&mut *tx).await?;

            // TREES
            let trees_iter = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::Tree(tree_data) = &b.specific_data {
                    Some((
                        b.base_data.id as i64,
                        tree_data.density,
                        tree_data.age,
                        tree_data.tree_type,
                        tree_data.variant,
                    ))
                } else {
                    None
                }
            });

            let mut tree_query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO trees (building_id, tree_type, density, age, variant)",
            );

            tree_query_builder.push_values(
                trees_iter,
                |mut b, (id, density, age, tree_type, variant)| {
                    b.push_bind(id)
                        .push_bind(tree_type)
                        .push_bind(density)
                        .push_bind(age)
                        .push_bind(variant);
                },
            );

            tree_query_builder.push(
                r#"
                    ON CONFLICT (building_id)
                    DO UPDATE SET
                        tree_type = EXCLUDED.tree_type,
                        density = EXCLUDED.density,
                        age = EXCLUDED.age,
                        variant = EXCLUDED.variant
                "#,
            );

            tree_query_builder.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn load_chunk_buildings(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<Vec<BuildingData>, sqlx::Error> {
        let mut buildings = Vec::new();

        let base_buildings_rows = sqlx::query(
            r#"
            SELECT 
                b.id, b.building_type_id, b.category,
                b.cell_q, b.cell_r, 
                b.quality, b.durability, b.damage, b.created_at,
                bt.specific_type
            FROM buildings_base b
            JOIN building_types bt ON b.building_type_id = bt.id
            WHERE chunk_x = $1 AND chunk_y = $2
        "#,
        )
        .bind(chunk_id.x)
        .bind(chunk_id.y)
        .fetch_all(&self.pool)
        .await?;

        for r in base_buildings_rows {
            let id = r.get::<i64, &str>("id");
            let category = r.get::<BuildingCategory, &str>("category");
            let specific_type = r.get::<BuildingSpecificType, &str>("specific_type");

            let base_data = BuildingBaseData {
                id: id as u64,
                building_type: BuildingType {
                    id: r.get("building_type_id"),
                    variant: String::new(), // TODO REMOVE FROM HERE
                    category: BuildingCategory::Natural,
                },
                cell: GridCell {
                    q: r.get("cell_q"),
                    r: r.get("cell_r"),
                },
                chunk: chunk_id.clone(),
                quality: r.get::<f64, &str>("quality") as f32,
                durability: r.get::<f64, &str>("durability") as f32,
                damage: r.get::<f64, &str>("damage") as f32,
                created_at: (r.get::<i64, &str>("created_at") as i64) as u64,
            };

            let specific_data = match (category, specific_type) {
                (BuildingCategory::Natural, BuildingSpecificType::Tree) => {
                    let tree = sqlx::query(
                        r#"
                            SELECT tree_type, density, age, variant
                            FROM trees
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    BuildingSpecific::Tree(TreeData {
                        tree_type: tree.get::<TreeType, &str>("tree_type"),
                        density: tree.get::<i32, &str>("density") as f32,
                        age: tree.get("age"),
                        variant: tree.get("variant"),
                    })
                }
                _ => BuildingSpecific::Unknown(),
            };

            buildings.push(BuildingData {
                base_data,
                specific_data,
            });
        }

        Ok(buildings)
    }
}
