//! Legacy handler helpers — kept for shared utility functions used by the lightyear action handler.
//! The tungstenite WebSocket handler has been removed (#138).

use shared::grid::{GridCell, GridConfig};
use shared::UnitData;
use sqlx::Row;

use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;
use shared::GameState;
use shared::protocol::{
    ConstructionCostNet, GameDataPayload, HarvestYieldNet,
    ItemDefinitionNet, RecipeIngredientNet, RecipeNet, TranslationEntry,
};

/// Claim a cell and its 6 hex neighbors for an organization's territory.
pub(crate) async fn claim_cell_and_neighbors(
    db_tables: &DatabaseTables,
    org_id: u64,
    center: &GridCell,
    claimed_cells: &mut Vec<GridCell>,
) {
    if db_tables
        .organizations
        .add_territory_cell(org_id, center)
        .await
        .is_ok()
    {
        claimed_cells.push(*center);
    }

    for neighbor in center.neighbors() {
        let already_taken = sqlx::query_scalar::<_, i64>(
            "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
        )
        .bind(neighbor.q)
        .bind(neighbor.r)
        .fetch_optional(&db_tables.pool)
        .await
        .ok()
        .flatten()
        .is_some();

        if !already_taken {
            if db_tables
                .organizations
                .add_territory_cell(org_id, &neighbor)
                .await
                .is_ok()
            {
                claimed_cells.push(neighbor);
            }
        }
    }
}

/// Build the GameDataPayload from the cached GameState.
pub(crate) fn build_game_data_payload(game_state: &GameState, dev_config: &DevConfig) -> GameDataPayload {
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
        .flat_map(|(_, costs)| costs.iter().map(|cost| ConstructionCostNet {
            building_type_id: cost.building_type_id,
            item_id: cost.item_id,
            quantity: cost.quantity,
        }))
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

/// Ensure the area around the lord's spawn is explored (voronoi zones).
pub(crate) async fn ensure_spawn_explored(
    lord: Option<UnitData>,
    db_tables: &DatabaseTables,
    player_id: i64,
    grid_config: &GridConfig,
) {
    if let Some(ref lord) = lord {
        let spawn_cell = GridCell {
            q: lord.current_cell.q,
            r: lord.current_cell.r,
        };
        let world_seed = crate::exploration::EXPLORATION_WORLD_SEED;

        let zone_ids =
            crate::exploration::zones_in_radius(&spawn_cell, 20, &grid_config.layout, world_seed);

        match db_tables
            .exploration_voronoi
            .mark_explored(&zone_ids, player_id)
            .await
        {
            Ok(newly) => {
                if !newly.is_empty() {
                    tracing::info!(
                        "Explored {} voronoi zones around spawn ({},{})",
                        newly.len(),
                        spawn_cell.q,
                        spawn_cell.r
                    );
                }
            }
            Err(e) => tracing::error!("Failed to explore spawn area: {}", e),
        }
    }
}
