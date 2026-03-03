use bincode::{Decode, Encode};

use crate::{BiomeTypeEnum, BuildingTypeEnum, ProfessionEnum};

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
    /// Resource costs
    pub costs: Vec<ResourceCost>,
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
            required_profession: None,
            duration_ticks: 1,
            executable: true,
        }
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

    pub fn with_profession(mut self, prof: ProfessionEnum) -> Self {
        self.required_profession = Some(prof);
        self
    }

    pub fn with_duration(mut self, ticks: u32) -> Self {
        self.duration_ticks = ticks;
        self
    }
}
