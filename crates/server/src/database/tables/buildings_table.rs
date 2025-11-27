use bevy::prelude::*;

use shared::{
    AgricultureData, AgricultureTypeEnum, AnimalBreedingData, AnimalBreedingTypeEnum,
    BuildingBaseData, BuildingCategoryEnum, BuildingData, BuildingSpecific,
    BuildingSpecificTypeEnum, CommerceData, CommerceTypeEnum, CultData, CultTypeEnum,
    EntertainmentData, EntertainmentTypeEnum, ManufacturingWorkshopData,
    ManufacturingWorkshopTypeEnum, TerrainChunkId, TreeData, TreeTypeEnum, grid::GridCell,
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
                        building_type_id = EXCLUDED.building_type_id,
                        category_id = EXCLUDED.category_id,
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
                        tree_type_id = EXCLUDED.tree_type_id,
                        density = EXCLUDED.density,
                        age = EXCLUDED.age,
                        variant = EXCLUDED.variant
                "#,
            );

            tree_query_builder.build().execute(&mut *tx).await?;

            // MANUFACTURING WORKSHOPS
            let workshops: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::ManufacturingWorkshop(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.workshop_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !workshops.is_empty() {
                let mut workshop_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.manufacturing_workshops (building_id, workshop_type_id, variant)",
                );

                workshop_query_builder.push_values(
                    workshops,
                    |mut b, (id, workshop_type, variant)| {
                        b.push_bind(id)
                            .push_bind(workshop_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                workshop_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            workshop_type_id = EXCLUDED.workshop_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                workshop_query_builder.build().execute(&mut *tx).await?;
            }

            // AGRICULTURE
            let agriculture: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::Agriculture(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.agriculture_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !agriculture.is_empty() {
                let mut agriculture_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.agriculture (building_id, agriculture_type_id, variant)",
                );

                agriculture_query_builder.push_values(
                    agriculture,
                    |mut b, (id, agriculture_type, variant)| {
                        b.push_bind(id)
                            .push_bind(agriculture_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                agriculture_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            agriculture_type_id = EXCLUDED.agriculture_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                agriculture_query_builder.build().execute(&mut *tx).await?;
            }

            // ANIMAL BREEDING
            let animal_breeding: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::AnimalBreeding(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.animal_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !animal_breeding.is_empty() {
                let mut animal_breeding_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.animal_breeding (building_id, animal_type_id, variant)",
                );

                animal_breeding_query_builder.push_values(
                    animal_breeding,
                    |mut b, (id, animal_type, variant)| {
                        b.push_bind(id)
                            .push_bind(animal_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                animal_breeding_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            animal_type_id = EXCLUDED.animal_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                animal_breeding_query_builder.build().execute(&mut *tx).await?;
            }

            // ENTERTAINMENT
            let entertainment: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::Entertainment(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.entertainment_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !entertainment.is_empty() {
                let mut entertainment_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.entertainment (building_id, entertainment_type_id, variant)",
                );

                entertainment_query_builder.push_values(
                    entertainment,
                    |mut b, (id, entertainment_type, variant)| {
                        b.push_bind(id)
                            .push_bind(entertainment_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                entertainment_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            entertainment_type_id = EXCLUDED.entertainment_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                entertainment_query_builder.build().execute(&mut *tx).await?;
            }

            // CULT
            let cult: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::Cult(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.cult_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !cult.is_empty() {
                let mut cult_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.cult (building_id, cult_type_id, variant)",
                );

                cult_query_builder.push_values(
                    cult,
                    |mut b, (id, cult_type, variant)| {
                        b.push_bind(id)
                            .push_bind(cult_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                cult_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            cult_type_id = EXCLUDED.cult_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                cult_query_builder.build().execute(&mut *tx).await?;
            }

            // COMMERCE
            let commerce: Vec<_> = chunk.iter().filter_map(|b| {
                if let BuildingSpecific::Commerce(data) = &b.specific_data {
                    Some((b.base_data.id as i64, data.commerce_type, data.variant))
                } else {
                    None
                }
            }).collect();

            if !commerce.is_empty() {
                let mut commerce_query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO buildings.commerce (building_id, commerce_type_id, variant)",
                );

                commerce_query_builder.push_values(
                    commerce,
                    |mut b, (id, commerce_type, variant)| {
                        b.push_bind(id)
                            .push_bind(commerce_type.to_id())
                            .push_bind(variant as i32);
                    },
                );

                commerce_query_builder.push(
                    r#"
                        ON CONFLICT (building_id)
                        DO UPDATE SET
                            commerce_type_id = EXCLUDED.commerce_type_id,
                            variant = EXCLUDED.variant
                    "#,
                );

                commerce_query_builder.build().execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    /// Crée un nouveau bâtiment en construction
    pub async fn create_building(
        &self,
        building_data: &BuildingData,
    ) -> Result<(), String> {
        let mut tx = self.pool.begin().await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Insert into buildings_base
        sqlx::query(
            r#"
            INSERT INTO buildings.buildings_base
                (id, building_type_id, category_id, chunk_x, chunk_y, cell_q, cell_r, created_at, quality, durability, damage, is_built)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(building_data.base_data.id as i64)
        .bind(building_data.base_data.specific_type.to_id())
        .bind(building_data.base_data.category.to_id())
        .bind(building_data.base_data.chunk.x)
        .bind(building_data.base_data.chunk.y)
        .bind(building_data.base_data.cell.q)
        .bind(building_data.base_data.cell.r)
        .bind(building_data.base_data.created_at as i64)
        .bind(building_data.base_data.quality as f64)
        .bind(building_data.base_data.durability as f64)
        .bind(building_data.base_data.damage as f64)
        .bind(false)    // is_built = false (en construction)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create building: {}", e))?;

        // Insert into specific instance table
        match &building_data.specific_data {
            BuildingSpecific::ManufacturingWorkshop(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.manufacturing_workshops (building_id, workshop_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.workshop_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert workshop data: {}", e))?;
            }
            BuildingSpecific::Agriculture(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.agriculture (building_id, agriculture_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.agriculture_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert agriculture data: {}", e))?;
            }
            BuildingSpecific::AnimalBreeding(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.animal_breeding (building_id, animal_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.animal_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert animal breeding data: {}", e))?;
            }
            BuildingSpecific::Entertainment(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.entertainment (building_id, entertainment_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.entertainment_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert entertainment data: {}", e))?;
            }
            BuildingSpecific::Cult(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.cult (building_id, cult_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.cult_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert cult data: {}", e))?;
            }
            BuildingSpecific::Commerce(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.commerce (building_id, commerce_type_id, variant)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.commerce_type.to_id())
                .bind(data.variant as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert commerce data: {}", e))?;
            }
            BuildingSpecific::Tree(data) => {
                sqlx::query(
                    r#"
                    INSERT INTO buildings.trees (building_id, tree_type_id, density, age, variant)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(building_data.base_data.id as i64)
                .bind(data.tree_type.to_id())
                .bind(data.density)
                .bind(data.age)
                .bind(data.variant)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to insert tree data: {}", e))?;
            }
            BuildingSpecific::Unknown() => {
                // No specific data to insert
            }
        }

        tx.commit().await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

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
                b.quality, b.durability, b.damage, b.created_at
            FROM buildings.buildings_base b
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
            let building_type_id: i32 = r.get("building_type_id");
            let specific_type = BuildingSpecificTypeEnum::from_id(building_type_id as i16)
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

            let specific_data = match specific_type {
                BuildingSpecificTypeEnum::Tree => {
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
                BuildingSpecificTypeEnum::ManufacturingWorkshop => {
                    let workshop = sqlx::query(
                        r#"
                            SELECT workshop_type_id, variant
                            FROM buildings.manufacturing_workshops
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let workshop_type = ManufacturingWorkshopTypeEnum::from_id(workshop.get("workshop_type_id"))
                        .unwrap_or(ManufacturingWorkshopTypeEnum::Blacksmith);

                    BuildingSpecific::ManufacturingWorkshop(ManufacturingWorkshopData {
                        workshop_type,
                        variant: workshop.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::Agriculture => {
                    let agriculture = sqlx::query(
                        r#"
                            SELECT agriculture_type_id, variant
                            FROM buildings.agriculture
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let agriculture_type = AgricultureTypeEnum::from_id(agriculture.get("agriculture_type_id"))
                        .unwrap_or(AgricultureTypeEnum::Farm);

                    BuildingSpecific::Agriculture(AgricultureData {
                        agriculture_type,
                        variant: agriculture.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::AnimalBreeding => {
                    let animal_breeding = sqlx::query(
                        r#"
                            SELECT animal_type_id, variant
                            FROM buildings.animal_breeding
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let animal_type = AnimalBreedingTypeEnum::from_id(animal_breeding.get("animal_type_id"))
                        .unwrap_or(AnimalBreedingTypeEnum::Cowshed);

                    BuildingSpecific::AnimalBreeding(AnimalBreedingData {
                        animal_type,
                        variant: animal_breeding.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::Entertainment => {
                    let entertainment = sqlx::query(
                        r#"
                            SELECT entertainment_type_id, variant
                            FROM buildings.entertainment
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let entertainment_type = EntertainmentTypeEnum::from_id(entertainment.get("entertainment_type_id"))
                        .unwrap_or(EntertainmentTypeEnum::Theater);

                    BuildingSpecific::Entertainment(EntertainmentData {
                        entertainment_type,
                        variant: entertainment.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::Cult => {
                    let cult = sqlx::query(
                        r#"
                            SELECT cult_type_id, variant
                            FROM buildings.cult
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let cult_type = CultTypeEnum::from_id(cult.get("cult_type_id"))
                        .unwrap_or(CultTypeEnum::Temple);

                    BuildingSpecific::Cult(CultData {
                        cult_type,
                        variant: cult.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::Commerce => {
                    let commerce = sqlx::query(
                        r#"
                            SELECT commerce_type_id, variant
                            FROM buildings.commerce
                            WHERE building_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let commerce_type = CommerceTypeEnum::from_id(commerce.get("commerce_type_id"))
                        .unwrap_or(CommerceTypeEnum::Bakehouse);

                    BuildingSpecific::Commerce(CommerceData {
                        commerce_type,
                        variant: commerce.get::<i32, &str>("variant") as u32,
                    })
                }
                BuildingSpecificTypeEnum::Unknown => BuildingSpecific::Unknown(),
            };

            buildings.push(BuildingData {
                base_data,
                specific_data,
            });
        }

        Ok(buildings)
    }
}
