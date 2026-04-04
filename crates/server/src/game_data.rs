use crate::dev::DevConfig;
use shared::GameState;
use shared::protocol::{
    ConstructionCostNet, GameDataPayload, HarvestYieldNet, ItemDefinitionNet, RecipeIngredientNet,
    RecipeNet, TranslationEntry,
};

/// Build the GameDataPayload from the cached GameState.
pub fn build_game_data_payload(game_state: &GameState, dev_config: &DevConfig) -> GameDataPayload {
    let items = game_state
        .item_definitions
        .iter()
        .map(|item| ItemDefinitionNet {
            id: item.id,
            name: item.name.clone(),
            item_type_id: item.item_type.to_id(),
            category_id: item.category.as_ref().map(|c| c.to_id()),
            weight_kg: item.weight_kg,
            base_price: item.base_price,
            is_perishable: item.is_perishable,
            is_equipable: item.is_equipable,
            equipment_slot_id: item.equipment_slot.as_ref().map(|e| e.to_id()),
            is_craftable: item.is_craftable,
        })
        .collect();

    let recipes = game_state
        .recipes
        .iter()
        .map(|recipe| RecipeNet {
            id: recipe.id,
            name: recipe.name.clone(),
            result_item_id: recipe.result_item_id,
            result_quantity: recipe.result_quantity,
            required_skill_id: recipe.required_skill.as_ref().map(|s| s.to_id()),
            required_skill_level: recipe.required_skill_level,
            craft_duration_seconds: recipe.craft_duration_seconds,
            required_building_type_id: recipe.required_building_type_id,
            ingredients: recipe
                .ingredients
                .iter()
                .map(|i| RecipeIngredientNet {
                    item_id: i.item_id,
                    quantity: i.quantity,
                })
                .collect(),
        })
        .collect();

    let construction_costs = game_state
        .construction_costs
        .iter()
        .flat_map(|(_, costs)| {
            costs.iter().map(|cost| ConstructionCostNet {
                building_type_id: cost.building_type_id,
                item_id: cost.item_id,
                quantity: cost.quantity,
            })
        })
        .collect();

    let harvest_yields = game_state
        .harvest_yields
        .iter()
        .map(|hy| HarvestYieldNet {
            resource_specific_type_id: hy.resource_specific_type_id,
            result_item_id: hy.result_item_id,
            base_quantity: hy.base_quantity,
            required_profession_id: hy.required_profession_id,
            duration_seconds: hy.duration_seconds,
        })
        .collect();

    let translations = game_state
        .translations
        .iter()
        .map(|(key, value)| TranslationEntry {
            entity_type: key.entity_type.clone(),
            entity_id: key.entity_id,
            language_id: key.language_id,
            field: key.field.clone(),
            value: value.clone(),
        })
        .collect();

    GameDataPayload {
        items,
        recipes,
        construction_costs,
        harvest_yields,
        translations,
        dev_mode: dev_config.dev_mode,
    }
}
