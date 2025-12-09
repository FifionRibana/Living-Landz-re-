use bevy::prelude::*;
use shared::{
    EquipmentSlotEnum, FullItemData, ItemDefinition, ItemInstance, ItemTypeEnum, Recipe,
    RecipeIngredient, ResourceCategoryEnum, SkillEnum, WorldPosition,
};
use sqlx::{PgPool, Row, types::chrono};
use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct ResourcesTable {
    pool: PgPool,
}

impl ResourcesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============ ITEM DEFINITIONS ============

    /// Charge une définition d'item par son ID
    pub async fn load_item_definition(&self, item_id: i32) -> Result<ItemDefinition, String> {
        let row = sqlx::query(
            r#"
            SELECT i.id, i.name, i.item_type_id, i.category_id,
                   i.weight_kg, i.volume_liters, i.base_price,
                   i.is_perishable, i.base_decay_rate_per_day,
                   i.is_equipable, i.equipment_slot_id,
                   i.is_craftable, i.description, i.image_url
            FROM resources.items i
            WHERE i.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load item definition: {}", e))?;

        // Charger les modificateurs de stats
        let stat_modifiers = self.load_item_stat_modifiers(item_id).await?;

        Ok(ItemDefinition {
            id: row.get("id"),
            name: row.get("name"),
            item_type: ItemTypeEnum::from_id(row.get("item_type_id"))
                .unwrap_or(ItemTypeEnum::Unknown),
            category: row
                .get::<Option<i16>, _>("category_id")
                .and_then(ResourceCategoryEnum::from_id),
            weight_kg: {
                let val: f64 = row.try_get("weight_kg").unwrap_or(0.0);
                val as f32
            },
            volume_liters: {
                let val: f64 = row.try_get("volume_liters").unwrap_or(0.0);
                val as f32
            },
            base_price: row.get("base_price"),
            is_perishable: row.get("is_perishable"),
            base_decay_rate_per_day: {
                let val: f64 = row.try_get("base_decay_rate_per_day").unwrap_or(0.0);
                val as f32
            },
            is_equipable: row.get("is_equipable"),
            equipment_slot: row
                .get::<Option<i16>, _>("equipment_slot_id")
                .and_then(EquipmentSlotEnum::from_id),
            is_craftable: row.get("is_craftable"),
            description: row.get("description"),
            image_url: row.get("image_url"),
            stat_modifiers,
        })
    }

    /// Charge tous les modificateurs de stats pour un item
    async fn load_item_stat_modifiers(&self, item_id: i32) -> Result<HashMap<String, i32>, String> {
        let rows = sqlx::query(
            r#"
            SELECT stat_name, modifier_value
            FROM resources.item_stat_modifiers
            WHERE item_id = $1
            "#,
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load stat modifiers: {}", e))?;

        let mut modifiers = HashMap::new();
        for row in rows {
            modifiers.insert(row.get("stat_name"), row.get("modifier_value"));
        }

        Ok(modifiers)
    }

    /// Charge toutes les définitions d'items
    pub async fn load_all_item_definitions(&self) -> Result<Vec<ItemDefinition>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM resources.items WHERE archived = FALSE
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load item IDs: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            let item_id: i32 = row.get("id");
            if let Ok(item) = self.load_item_definition(item_id).await {
                items.push(item);
            }
        }

        Ok(items)
    }

    // ============ ITEM INSTANCES ============

    /// Crée une nouvelle instance d'item
    pub async fn create_item_instance(
        &self,
        item_id: i32,
        quality: f32,
        owner_unit_id: Option<u64>,
        world_position: Option<WorldPosition>,
    ) -> Result<u64, String> {
        let instance_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO resources.item_instances
            (item_id, quality, current_decay, last_decay_update, owner_unit_id,
             world_cell_q, world_cell_r, world_chunk_x, world_chunk_y)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(item_id)
        .bind(quality as f64)
        .bind(0.0) // current_decay
        .bind(chrono::Utc::now())
        .bind(owner_unit_id.map(|id| id as i64))
        .bind(world_position.map(|p| p.cell_q))
        .bind(world_position.map(|p| p.cell_r))
        .bind(world_position.map(|p| p.chunk_x))
        .bind(world_position.map(|p| p.chunk_y))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create item instance: {}", e))?;

        Ok(instance_id as u64)
    }

    /// Charge une instance d'item
    pub async fn load_item_instance(&self, instance_id: u64) -> Result<ItemInstance, String> {
        let row = sqlx::query(
            r#"
            SELECT id, item_id, quality, current_decay, last_decay_update,
                   owner_unit_id, world_cell_q, world_cell_r, world_chunk_x, world_chunk_y,
                   created_at
            FROM resources.item_instances
            WHERE id = $1
            "#,
        )
        .bind(instance_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load item instance: {}", e))?;

        let world_position = if row.get::<Option<i32>, _>("world_cell_q").is_some() {
            Some(WorldPosition {
                cell_q: row.get("world_cell_q"),
                cell_r: row.get("world_cell_r"),
                chunk_x: row.get("world_chunk_x"),
                chunk_y: row.get("world_chunk_y"),
            })
        } else {
            None
        };

        Ok(ItemInstance {
            id: row.get::<i64, _>("id") as u64,
            item_id: row.get("item_id"),
            quality: {
                let val: f64 = row.try_get("quality").unwrap_or(1.0);
                val as f32
            },
            current_decay: {
                let val: f64 = row.try_get("current_decay").unwrap_or(0.0);
                val as f32
            },
            last_decay_update: row
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_decay_update")
                .map(|dt| dt.timestamp() as u64)
                .unwrap_or(0),
            owner_unit_id: row.get::<Option<i64>, _>("owner_unit_id").map(|id| id as u64),
            world_position,
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").timestamp() as u64,
        })
    }

    /// Met à jour le decay d'une instance
    pub async fn update_item_instance_decay(
        &self,
        instance_id: u64,
        current_decay: f32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE resources.item_instances
            SET current_decay = $2, last_decay_update = NOW()
            WHERE id = $1
            "#,
        )
        .bind(instance_id as i64)
        .bind(current_decay as f64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update item decay: {}", e))?;

        Ok(())
    }

    /// Transfert une instance d'item à un propriétaire
    pub async fn transfer_item_instance(
        &self,
        instance_id: u64,
        new_owner_id: Option<u64>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE resources.item_instances
            SET owner_unit_id = $2,
                world_cell_q = NULL,
                world_cell_r = NULL,
                world_chunk_x = NULL,
                world_chunk_y = NULL
            WHERE id = $1
            "#,
        )
        .bind(instance_id as i64)
        .bind(new_owner_id.map(|id| id as i64))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to transfer item: {}", e))?;

        Ok(())
    }

    /// Place une instance d'item dans le monde
    pub async fn place_item_in_world(
        &self,
        instance_id: u64,
        position: WorldPosition,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE resources.item_instances
            SET owner_unit_id = NULL,
                world_cell_q = $2,
                world_cell_r = $3,
                world_chunk_x = $4,
                world_chunk_y = $5
            WHERE id = $1
            "#,
        )
        .bind(instance_id as i64)
        .bind(position.cell_q)
        .bind(position.cell_r)
        .bind(position.chunk_x)
        .bind(position.chunk_y)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to place item in world: {}", e))?;

        Ok(())
    }

    // ============ RECIPES ============

    /// Charge une recette par son ID
    pub async fn load_recipe(&self, recipe_id: i32) -> Result<Recipe, String> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, result_item_id, result_quantity,
                   required_skill_id, required_skill_level, craft_duration_seconds,
                   required_building_type_id
            FROM resources.recipes
            WHERE id = $1
            "#,
        )
        .bind(recipe_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load recipe: {}", e))?;

        // Charger les ingrédients
        let ingredients = self.load_recipe_ingredients(recipe_id).await?;

        Ok(Recipe {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            result_item_id: row.get("result_item_id"),
            result_quantity: row.get("result_quantity"),
            required_skill: row
                .get::<Option<i16>, _>("required_skill_id")
                .and_then(SkillEnum::from_id),
            required_skill_level: row.get("required_skill_level"),
            craft_duration_seconds: row.get("craft_duration_seconds"),
            required_building_type_id: row.get("required_building_type_id"),
            ingredients,
        })
    }

    /// Charge les ingrédients d'une recette
    async fn load_recipe_ingredients(&self, recipe_id: i32) -> Result<Vec<RecipeIngredient>, String> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, quantity
            FROM resources.recipe_ingredients
            WHERE recipe_id = $1
            "#,
        )
        .bind(recipe_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load recipe ingredients: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| RecipeIngredient {
                item_id: row.get("item_id"),
                quantity: row.get("quantity"),
            })
            .collect())
    }

    /// Charge toutes les recettes
    pub async fn load_all_recipes(&self) -> Result<Vec<Recipe>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM resources.recipes WHERE archived = FALSE
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load recipe IDs: {}", e))?;

        let mut recipes = Vec::new();
        for row in rows {
            let recipe_id: i32 = row.get("id");
            if let Ok(recipe) = self.load_recipe(recipe_id).await {
                recipes.push(recipe);
            }
        }

        Ok(recipes)
    }

    /// Charge les recettes craftables par un item donné
    pub async fn load_recipes_for_result_item(&self, item_id: i32) -> Result<Vec<Recipe>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM resources.recipes
            WHERE result_item_id = $1 AND archived = FALSE
            "#,
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load recipes for item: {}", e))?;

        let mut recipes = Vec::new();
        for row in rows {
            let recipe_id: i32 = row.get("id");
            if let Ok(recipe) = self.load_recipe(recipe_id).await {
                recipes.push(recipe);
            }
        }

        Ok(recipes)
    }

    // ============ FULL ITEM DATA ============

    /// Charge un item complet (définition + instance)
    pub async fn load_full_item(&self, instance_id: u64) -> Result<FullItemData, String> {
        let instance = self.load_item_instance(instance_id).await?;
        let definition = self.load_item_definition(instance.item_id).await?;

        Ok(FullItemData {
            definition,
            instance,
        })
    }
}
