use bevy::prelude::*;

use shared::{
    BuildingBaseData, BuildingCategoryEnum, BuildingData, BuildingSpecific,
    BuildingSpecificTypeEnum, TerrainChunkId, TreeData, TreeTypeEnum, grid::GridCell,
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
                "INSERT INTO buildings.buildings_base (id, building_type_id, category_id, chunk_x, chunk_y, cell_q, cell_r, created_at, quality, durability, damage)",
            );

            query_builder.push_values(chunk.iter(), |mut b, building| {
                let building_data = &building.base_data;
                b.push_bind(building_data.id as i64)
                    .push_bind(building_data.specific_type.to_id())
                    .push_bind(building_data.category.to_id())
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
                "INSERT INTO buildings.trees (building_id, tree_type_id, density, age, variant)",
            );

            tree_query_builder.push_values(
                trees_iter,
                |mut b, (id, density, age, tree_type, variant)| {
                    b.push_bind(id)
                        .push_bind(tree_type.to_id())
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

    /// Crée un nouveau bâtiment en construction
    pub async fn create_building(
        &self,
        building_id: u64,
        building_type: BuildingSpecificTypeEnum,
        category: BuildingCategoryEnum,
        chunk: &TerrainChunkId,
        cell: &GridCell,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO buildings.buildings_base
                (id, building_type_id, category_id, chunk_x, chunk_y, cell_q, cell_r, created_at, quality, durability, damage, is_built)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(building_id as i64)
        .bind(building_type.to_id())
        .bind(category.to_id())
        .bind(chunk.x)
        .bind(chunk.y)
        .bind(cell.q)
        .bind(cell.r)
        .bind(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64)
        .bind(1.0_f64) // quality
        .bind(1.0_f64) // durability
        .bind(0.0_f64) // damage
        .bind(false)    // is_built = false (en construction)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create building: {}", e))?;

        Ok(())
    }

    /// Marque un bâtiment comme construit
    pub async fn mark_building_as_built(&self, building_id: u64) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE buildings.buildings_base
            SET is_built = true
            WHERE id = $1
            "#,
        )
        .bind(building_id as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to mark building as built: {}", e))?;

        Ok(())
    }

    /// Supprime un bâtiment (en cas d'échec de construction)
    pub async fn delete_building(&self, building_id: u64) -> Result<(), String> {
        // Supprime d'abord les données spécifiques (trees, etc.) - cascade devrait le faire
        // mais on peut être explicite
        sqlx::query(
            r#"
            DELETE FROM buildings.buildings_base
            WHERE id = $1
            "#,
        )
        .bind(building_id as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete building: {}", e))?;

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
                b.id, b.building_type_id, b.category_id,
                b.cell_q, b.cell_r,
                b.quality, b.durability, b.damage, b.created_at,
                bt.specific_type_id
            FROM buildings.buildings_base b
            JOIN buildings.building_types bt ON b.building_type_id = bt.id
            WHERE chunk_x = $1 AND chunk_y = $2 AND is_built = true
        "#,
        )
        .bind(chunk_id.x)
        .bind(chunk_id.y)
        .fetch_all(&self.pool)
        .await?;

        for r in base_buildings_rows {
            let id = r.get::<i64, &str>("id");
            let category = BuildingCategoryEnum::from_id(r.get("category_id"))
                .unwrap_or(BuildingCategoryEnum::Unknown);
            let specific_type = BuildingSpecificTypeEnum::from_id(r.get("specific_type_id"))
                .unwrap_or(BuildingSpecificTypeEnum::Unknown);

            let base_data = BuildingBaseData {
                id: id as u64,
                specific_type: specific_type.clone(),
                category: category.clone(),
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
                (BuildingCategoryEnum::Natural, BuildingSpecificTypeEnum::Tree) => {
                    let tree = sqlx::query(
                        r#"
                            SELECT tree_type_id, density, age, variant
                            FROM buildings.trees
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let tree_type = TreeTypeEnum::from_id(tree.get("tree_type_id"))
                        .unwrap_or(TreeTypeEnum::Cedar);

                    BuildingSpecific::Tree(TreeData {
                        tree_type,
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
