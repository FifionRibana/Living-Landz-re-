use bevy::{log::tracing, prelude::*};
use std::collections::HashMap;

use shared::protocol::{
    ConstructionCostNet, GameDataPayload, HarvestYieldNet, ItemDefinitionNet, RecipeNet,
    TranslationEntry,
};

/// Client-side cache for static game data received from the server at login.
#[derive(Resource, Default)]
pub struct GameDataCache {
    pub items: Vec<ItemDefinitionNet>,
    pub recipes: Vec<RecipeNet>,
    pub construction_costs: Vec<ConstructionCostNet>,
    pub harvest_yields: Vec<HarvestYieldNet>,
    /// Translations indexed by (entity_type, entity_id, language_id, field) → value
    translations: HashMap<(String, i32, i16, String), String>,
    pub loaded: bool,
    pub dev_mode: bool,
}

impl GameDataCache {
    /// Populate the cache from the server payload.
    pub fn load_from_payload(&mut self, payload: GameDataPayload) {
        self.items = payload.items;
        self.recipes = payload.recipes;
        self.construction_costs = payload.construction_costs;
        self.harvest_yields = payload.harvest_yields;

        self.translations.clear();
        for t in payload.translations {
            self.translations
                .insert((t.entity_type, t.entity_id, t.language_id, t.field), t.value);
        }

        self.loaded = true;
        self.dev_mode = payload.dev_mode;
        if self.dev_mode {
            tracing::warn!("⚡ DEV MODE — resource checks disabled client-side");
        }
    }

    /// Translated name for an item (falls back to definition name).
    pub fn item_name(&self, item_id: i32, lang_id: i16) -> String {
        self.get_translation("item", item_id, "name", lang_id)
            .unwrap_or_else(|| {
                self.items
                    .iter()
                    .find(|i| i.id == item_id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| format!("Item #{}", item_id))
            })
    }

    /// Translated name for a recipe.
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

    /// Translated name for a building type.
    pub fn building_type_name(&self, bt_id: i32, lang_id: i16) -> String {
        self.get_translation("building_type", bt_id, "name", lang_id)
            .unwrap_or_else(|| format!("Building #{}", bt_id))
    }

    /// Construction costs for a building type.
    pub fn building_costs(&self, building_type_id: i32) -> Vec<&ConstructionCostNet> {
        self.construction_costs
            .iter()
            .filter(|c| c.building_type_id == building_type_id)
            .collect()
    }

    /// Recipes that require a specific building type.
    pub fn recipes_for_building(&self, building_type_id: i16) -> Vec<&RecipeNet> {
        self.recipes
            .iter()
            .filter(|r| r.required_building_type_id == Some(building_type_id))
            .collect()
    }

    /// Item definition by ID.
    pub fn get_item(&self, item_id: i32) -> Option<&ItemDefinitionNet> {
        self.items.iter().find(|i| i.id == item_id)
    }

    fn get_translation(
        &self,
        entity_type: &str,
        entity_id: i32,
        field: &str,
        lang_id: i16,
    ) -> Option<String> {
        self.translations
            .get(&(
                entity_type.to_string(),
                entity_id,
                lang_id,
                field.to_string(),
            ))
            .cloned()
    }
}
