use bincode::{Decode, Encode};
use std::collections::HashMap;

use crate::protocol::{ConstructionCostNet, ItemDefinitionNet, RecipeNet};
use crate::{BiomeTypeEnum, BuildingTypeEnum, ProfessionEnum};

// ─── Game data ref ─────────────────────────────────────────

/// Read-only view of DB game data, passed into action resolution.
/// Built by the client from GameDataCache before calling available_actions().
pub struct GameDataRef<'a> {
    pub items: &'a [ItemDefinitionNet],
    pub recipes: &'a [RecipeNet],
    pub construction_costs: &'a [ConstructionCostNet],
    /// Pre-resolved translated item names (item_id -> display name)
    pub item_names: HashMap<i32, String>,
}

impl GameDataRef<'_> {
    pub fn item_name(&self, item_id: i32) -> String {
        self.item_names
            .get(&item_id)
            .cloned()
            .unwrap_or_else(|| {
                self.items
                    .iter()
                    .find(|i| i.id == item_id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| format!("#{}", item_id))
            })
    }

    pub fn recipes_for_building(&self, building_type_id: i16) -> Vec<&RecipeNet> {
        self.recipes
            .iter()
            .filter(|r| r.required_building_type_id == Some(building_type_id))
            .collect()
    }

    pub fn building_costs(&self, building_type_id: i32) -> Vec<&ConstructionCostNet> {
        self.construction_costs
            .iter()
            .filter(|c| c.building_type_id == building_type_id)
            .collect()
    }
}

// ─── View context ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ActionViewContext {
    Map,
    Cell,
}

// ─── Action context ─────────────────────────────────────────

/// Everything the action system needs to determine what's available.
/// Computed client-side from game state, also usable server-side for validation.
#[derive(Debug, Clone)]
pub struct UIActionContext {
    pub view: ActionViewContext,
    /// Building on the current cell (None = empty terrain)
    pub building: Option<BuildingTypeEnum>,
    /// Terrain biome of the current cell
    pub terrain: BiomeTypeEnum,
    /// Professions of all currently selected units
    pub selected_professions: Vec<ProfessionEnum>,
    /// Whether at least one adjacent cell has a road
    pub has_adjacent_road: bool,
}

impl UIActionContext {
    /// Does any selected unit have this profession?
    pub fn has_profession(&self, profession: &ProfessionEnum) -> bool {
        self.selected_professions.contains(profession)
    }

    /// Does any selected unit match any of the given professions?
    pub fn has_any_profession(&self, professions: &[ProfessionEnum]) -> bool {
        professions.iter().any(|p| self.selected_professions.contains(p))
    }

    pub fn is_cell_view(&self) -> bool {
        self.view == ActionViewContext::Cell
    }

    pub fn is_map_view(&self) -> bool {
        self.view == ActionViewContext::Map
    }

    pub fn has_building(&self) -> bool {
        self.building.is_some()
    }
}

// ─── Resource cost ──────────────────────────────────────────

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceCost {
    pub name: String,
    pub quantity: u32,
}

impl ResourceCost {
    pub fn new(name: &str, quantity: u32) -> Self {
        Self {
            name: name.to_string(),
            quantity,
        }
    }
}

// ─── Action entry ───────────────────────────────────────────

/// A concrete action that can be displayed in the action panel.
#[derive(Debug, Clone)]
pub struct ActionEntry {
    /// Unique identifier (e.g. "produce_bread", "build_blacksmith", "train_warrior")
    pub id: String,
    /// Display name
    pub name: String,
    /// Short description
    pub description: String,
    /// Icon asset path
    pub icon: String,
    /// Resource costs (inputs consumed)
    pub costs: Vec<ResourceCost>,
    /// Resource outputs (items produced)
    pub outputs: Vec<ResourceCost>,
    /// Which profession is needed to execute this (None = any)
    pub required_profession: Option<ProfessionEnum>,
    /// Duration in game ticks
    pub duration_ticks: u32,
    /// Whether this action is currently executable (enough resources, etc.)
    /// Client can set this to false to grey out
    pub executable: bool,
}

impl ActionEntry {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            icon: String::new(),
            costs: Vec::new(),
            outputs: Vec::new(),
            required_profession: None,
            duration_ticks: 1,
            executable: true,
        }
    }

    /// Create an ActionEntry from a DB recipe (RecipeNet).
    /// Uses numeric recipe ID in the action ID for server compatibility.
    pub fn from_recipe_net(recipe: &RecipeNet, game_data: &GameDataRef) -> Self {
        let mut entry = Self::new(
            &format!("produce_{}", recipe.id),
            &recipe.name,
        )
        .with_description(&recipe.name)
        .with_icon("ui/icons/cog.png")
        .with_duration(recipe.craft_duration_seconds as u32);

        for ing in &recipe.ingredients {
            let name = game_data.item_name(ing.item_id);
            entry = entry.with_cost(&name, ing.quantity as u32);
        }

        // Output
        let result_name = game_data.item_name(recipe.result_item_id);
        entry = entry.with_output(&result_name, recipe.result_quantity as u32);

        entry
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn with_cost(mut self, name: &str, qty: u32) -> Self {
        self.costs.push(ResourceCost {
            name: name.to_string(),
            quantity: qty,
        });
        self
    }

    pub fn with_output(mut self, name: &str, qty: u32) -> Self {
        self.outputs.push(ResourceCost {
            name: name.to_string(),
            quantity: qty,
        });
        self
    }

    pub fn with_profession(mut self, prof: ProfessionEnum) -> Self {
        self.required_profession = Some(prof);
        self
    }

    pub fn with_duration(mut self, ticks: u32) -> Self {
        self.duration_ticks = ticks;
        self
    }
}
