use bevy::prelude::*;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use crate::{
    BuildingCategory, BuildingSpecificType, BuildingType, ConstructionCost, HarvestYield,
    ItemDefinition, Recipe, RecipeIngredient, ResourceCategory, ResourceSpecificType, ResourceType,
    TranslationKey, atlas::TreeAtlas,
};

#[derive(Resource)]
pub struct GameState {
    pub pool: PgPool,

    // Lookup table cache
    pub building_categories: Vec<BuildingCategory>,
    pub building_specific_types: Vec<BuildingSpecificType>,
    pub resource_categories: Vec<ResourceCategory>,
    pub resource_specific_types: Vec<ResourceSpecificType>,

    // Principal cache
    pub building_types: Vec<BuildingType>,
    pub resource_types: Vec<ResourceType>,

    pub tree_atlas: TreeAtlas,

    // Economic cache
    pub item_definitions: Vec<ItemDefinition>,
    pub recipes: Vec<Recipe>,
    pub construction_costs: HashMap<i32, Vec<ConstructionCost>>,
    pub harvest_yields: Vec<HarvestYield>,
    pub translations: HashMap<TranslationKey, String>,
}

impl GameState {
    pub fn new(pool: PgPool) -> Self {
        let mut tree_atlas = TreeAtlas::default();
        tree_atlas.load();

        Self {
            pool,
            building_categories: Vec::new(),
            building_specific_types: Vec::new(),
            resource_categories: Vec::new(),
            resource_specific_types: Vec::new(),
            building_types: Vec::new(),
            resource_types: Vec::new(),
            tree_atlas,
            item_definitions: Vec::new(),
            recipes: Vec::new(),
            construction_costs: HashMap::new(),
            harvest_yields: Vec::new(),
            translations: HashMap::new(),
        }
    }

    // ============ INITIALIZATION ============

    pub async fn initialize_caches(&mut self) -> Result<(), sqlx::Error> {
        self.building_categories = sqlx::query_as::<_, BuildingCategory>(
            "SELECT * FROM buildings.building_categories ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.building_specific_types = sqlx::query_as::<_, BuildingSpecificType>(
            "SELECT * FROM buildings.building_specific_types WHERE archived = FALSE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_categories = sqlx::query_as::<_, ResourceCategory>(
            "SELECT * FROM resources.resource_categories ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_specific_types = sqlx::query_as::<_, ResourceSpecificType>(
            "SELECT * FROM resources.resource_specific_types WHERE archived = FALSE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        // Charger les caches principaux
        self.building_types = sqlx::query_as::<_, BuildingType>(
            r#"
            SELECT 
                id,
                name,
                category_id,
                specific_type_id,
                description,
                archived
            FROM buildings.building_types
            WHERE archived = FALSE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_types = sqlx::query_as::<_, ResourceType>(
            r#"
            SELECT 
                id,
                name,
                category_id,
                specific_type_id,
                description,
                archived
            FROM resources.resource_types
            WHERE archived = FALSE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // --- Item definitions ---
        let item_rows = sqlx::query(
            r#"SELECT id, name, item_type_id, category_id, weight_kg, volume_liters,
                      base_price, is_perishable, base_decay_rate_per_day,
                      is_equipable, equipment_slot_id, is_craftable, description, image_url
               FROM resources.items WHERE archived = FALSE ORDER BY id"#,
        )
        .fetch_all(&self.pool)
        .await?;

        self.item_definitions = item_rows
            .iter()
            .map(|row| {
                use crate::{EquipmentSlotEnum, ItemTypeEnum, ResourceCategoryEnum};
                ItemDefinition {
                    id: row.get("id"),
                    name: row.get("name"),
                    item_type: ItemTypeEnum::from_id(row.get("item_type_id"))
                        .unwrap_or(ItemTypeEnum::Unknown),
                    category: row
                        .get::<Option<i16>, _>("category_id")
                        .and_then(ResourceCategoryEnum::from_id),
                    weight_kg: row.try_get::<f64, _>("weight_kg").unwrap_or(0.0) as f32,
                    volume_liters: row.try_get::<f64, _>("volume_liters").unwrap_or(0.0) as f32,
                    base_price: row.get("base_price"),
                    is_perishable: row.get("is_perishable"),
                    base_decay_rate_per_day: row
                        .try_get::<f64, _>("base_decay_rate_per_day")
                        .unwrap_or(0.0) as f32,
                    is_equipable: row.get("is_equipable"),
                    equipment_slot: row
                        .get::<Option<i16>, _>("equipment_slot_id")
                        .and_then(EquipmentSlotEnum::from_id),
                    is_craftable: row.get("is_craftable"),
                    description: row.get("description"),
                    image_url: row.get("image_url"),
                    stat_modifiers: std::collections::HashMap::new(),
                }
            })
            .collect();

        // --- Recipes with ingredients ---
        let recipe_rows = sqlx::query(
            r#"SELECT id, name, description, result_item_id, result_quantity,
                      required_skill_id, required_skill_level, craft_duration_seconds,
                      required_building_type_id
               FROM resources.recipes WHERE archived = FALSE ORDER BY id"#,
        )
        .fetch_all(&self.pool)
        .await?;

        self.recipes = Vec::new();
        for rrow in &recipe_rows {
            let recipe_id: i32 = rrow.get("id");
            let ingredient_rows = sqlx::query(
                "SELECT item_id, quantity FROM resources.recipe_ingredients WHERE recipe_id = $1",
            )
            .bind(recipe_id)
            .fetch_all(&self.pool)
            .await?;

            let ingredients: Vec<RecipeIngredient> = ingredient_rows
                .iter()
                .map(|ir| RecipeIngredient {
                    item_id: ir.get("item_id"),
                    quantity: ir.get("quantity"),
                })
                .collect();

            self.recipes.push(Recipe {
                id: recipe_id,
                name: rrow.get("name"),
                description: rrow.get("description"),
                result_item_id: rrow.get("result_item_id"),
                result_quantity: rrow.get("result_quantity"),
                required_skill: rrow
                    .get::<Option<i16>, _>("required_skill_id")
                    .and_then(crate::SkillEnum::from_id),
                required_skill_level: rrow.get("required_skill_level"),
                craft_duration_seconds: rrow.get("craft_duration_seconds"),
                required_building_type_id: rrow.get("required_building_type_id"),
                ingredients,
            });
        }

        // --- Construction costs ---
        let cost_rows = sqlx::query(
            "SELECT building_type_id, item_id, quantity FROM buildings.construction_costs",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        self.construction_costs.clear();
        for crow in &cost_rows {
            let cost = ConstructionCost {
                building_type_id: crow.get("building_type_id"),
                item_id: crow.get("item_id"),
                quantity: crow.get("quantity"),
            };
            self.construction_costs
                .entry(cost.building_type_id)
                .or_default()
                .push(cost);
        }

        // --- Harvest yields ---
        let yield_rows = sqlx::query(
            r#"SELECT id, resource_specific_type_id, result_item_id, base_quantity,
                      quality_min, quality_max, required_profession_id,
                      required_tool_item_id, tool_bonus_quantity, duration_seconds
               FROM resources.harvest_yields"#,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        self.harvest_yields = yield_rows
            .iter()
            .map(|yr| HarvestYield {
                id: yr.get("id"),
                resource_specific_type_id: yr.get("resource_specific_type_id"),
                result_item_id: yr.get("result_item_id"),
                base_quantity: yr.get("base_quantity"),
                quality_min: yr.try_get::<f64, _>("quality_min").unwrap_or(0.5) as f32,
                quality_max: yr.try_get::<f64, _>("quality_max").unwrap_or(1.0) as f32,
                required_profession_id: yr.get("required_profession_id"),
                required_tool_item_id: yr.get("required_tool_item_id"),
                tool_bonus_quantity: yr.try_get("tool_bonus_quantity").unwrap_or(0),
                duration_seconds: yr.get("duration_seconds"),
            })
            .collect();

        // --- Translations ---
        let translation_rows = sqlx::query(
            "SELECT entity_type, entity_id, language_id, field, value FROM game.translations",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        self.translations.clear();
        for trow in &translation_rows {
            let key = TranslationKey {
                entity_type: trow.get("entity_type"),
                entity_id: trow.get("entity_id"),
                language_id: trow.get("language_id"),
                field: trow.get("field"),
            };
            self.translations.insert(key, trow.get("value"));
        }

        Ok(())
    }

    // ============ ECONOMY HELPERS ============

    /// Translated name for an item
    pub fn item_name(&self, item_id: i32, lang_id: i16) -> String {
        self.get_translation("item", item_id, "name", lang_id)
            .unwrap_or_else(|| {
                self.item_definitions
                    .iter()
                    .find(|i| i.id == item_id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| format!("Item #{}", item_id))
            })
    }

    /// Translated name for a recipe
    pub fn recipe_name(&self, recipe_id: i32, lang_id: i16) -> String {
        self.get_translation("recipe", recipe_id, "name", lang_id)
            .unwrap_or_else(|| {
                self.recipes
                    .iter()
                    .find(|r| r.id == recipe_id)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| format!("Recipe #{}", recipe_id))
            })
    }

    /// Translated name for a building type
    pub fn building_type_name(&self, bt_id: i32, lang_id: i16) -> String {
        self.get_translation("building_type", bt_id, "name", lang_id)
            .unwrap_or_else(|| {
                self.building_types
                    .iter()
                    .find(|bt| bt.id == bt_id)
                    .map(|bt| bt.name.clone())
                    .unwrap_or_else(|| format!("Building #{}", bt_id))
            })
    }

    fn get_translation(
        &self,
        entity_type: &str,
        entity_id: i32,
        field: &str,
        lang_id: i16,
    ) -> Option<String> {
        let key = TranslationKey {
            entity_type: entity_type.to_string(),
            entity_id,
            language_id: lang_id,
            field: field.to_string(),
        };
        self.translations.get(&key).cloned()
    }

    /// Recipes for a specific building type
    pub fn recipes_for_building(&self, building_type_id: i16) -> Vec<&Recipe> {
        self.recipes
            .iter()
            .filter(|r| r.required_building_type_id == Some(building_type_id))
            .collect()
    }

    /// Construction costs for a building type
    pub fn building_costs(&self, building_type_id: i32) -> &[ConstructionCost] {
        self.construction_costs
            .get(&building_type_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Harvest yields for a resource type
    pub fn harvest_yields_for(&self, resource_type_id: i16) -> Vec<&HarvestYield> {
        self.harvest_yields
            .iter()
            .filter(|h| h.resource_specific_type_id == resource_type_id)
            .collect()
    }

    /// Item definition by ID
    pub fn get_item_definition(&self, item_id: i32) -> Option<&ItemDefinition> {
        self.item_definitions.iter().find(|i| i.id == item_id)
    }

    // ============ BUILDING CATEGORIES ============

    pub fn get_building_category(&self, id: i16) -> Option<&BuildingCategory> {
        self.building_categories.iter().find(|bc| bc.id == id)
    }

    pub fn get_building_category_id(&self, name: &str) -> Option<i16> {
        self.building_categories
            .iter()
            .find(|bc| bc.name.eq_ignore_ascii_case(name))
            .map(|bc| bc.id)
    }

    // ============ BUILDING SPECIFIC TYPES ============

    pub fn get_building_specific_type(&self, id: i16) -> Option<&BuildingSpecificType> {
        self.building_specific_types.iter().find(|bst| bst.id == id)
    }

    pub fn get_building_specific_type_id(&self, name: &str) -> Option<i16> {
        self.building_specific_types
            .iter()
            .find(|bst| bst.name.eq_ignore_ascii_case(name))
            .map(|bst| bst.id)
    }

    // ============ BUILDING TYPES ============

    pub fn get_building_type(&self, id: i32) -> Option<&BuildingType> {
        self.building_types.iter().find(|bt| bt.id == id)
    }

    pub fn get_building_type_by_name(&self, name: &str) -> Option<&BuildingType> {
        self.building_types
            .iter()
            .find(|bt| bt.name.eq_ignore_ascii_case(name))
    }

    pub fn get_building_type_id(&self, name: &str) -> Option<i32> {
        self.get_building_type_by_name(name).map(|bt| bt.id)
    }

    pub fn get_buildings_by_category(&self, category_id: i16) -> Vec<&BuildingType> {
        self.building_types
            .iter()
            .filter(|bt| bt.category_id == category_id)
            .collect()
    }

    pub fn get_buildings_by_specific_type(&self, specific_type_id: i16) -> Vec<&BuildingType> {
        self.building_types
            .iter()
            .filter(|bt| bt.specific_type_id == specific_type_id)
            .collect()
    }

    // ============ RESOURCE CATEGORIES ============

    pub fn get_resource_category(&self, id: i16) -> Option<&ResourceCategory> {
        self.resource_categories.iter().find(|rc| rc.id == id)
    }

    pub fn get_resource_category_id(&self, name: &str) -> Option<i16> {
        self.resource_categories
            .iter()
            .find(|rc| rc.name.eq_ignore_ascii_case(name))
            .map(|rc| rc.id)
    }

    // ============ RESOURCE SPECIFIC TYPES ============

    pub fn get_resource_specific_type(&self, id: i16) -> Option<&ResourceSpecificType> {
        self.resource_specific_types.iter().find(|bst| bst.id == id)
    }

    pub fn get_resource_specific_type_id(&self, name: &str) -> Option<i16> {
        self.resource_specific_types
            .iter()
            .find(|bst| bst.name.eq_ignore_ascii_case(name))
            .map(|bst| bst.id)
    }

    // ============ RESOURCE TYPES ============

    pub fn get_resource_type(&self, id: i32) -> Option<&ResourceType> {
        self.resource_types.iter().find(|rt| rt.id == id)
    }

    pub fn get_resource_type_by_name(&self, name: &str) -> Option<&ResourceType> {
        self.resource_types
            .iter()
            .find(|rt| rt.name.eq_ignore_ascii_case(name))
    }

    pub fn get_resource_type_id(&self, name: &str) -> Option<i32> {
        self.get_resource_type_by_name(name).map(|rt| rt.id)
    }

    pub fn get_resources_by_category(&self, category_id: i16) -> Vec<&ResourceType> {
        self.resource_types
            .iter()
            .filter(|rt| rt.category_id == category_id)
            .collect()
    }

    pub fn get_resources_by_specific_type(&self, specific_type_id: i16) -> Vec<&ResourceType> {
        self.resource_types
            .iter()
            .filter(|rt| rt.specific_type_id == specific_type_id)
            .collect()
    }
}
