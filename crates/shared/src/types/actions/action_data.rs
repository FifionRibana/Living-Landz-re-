use bincode::{Decode, Encode};

use crate::{
    ActionStatus, ActionType, BuildingCategory, BuildingType, ResourceType, TerrainChunkId,
    grid::GridCell,
};

pub struct ActionContext {
    pub player_id: u64,
    pub grid_cell: GridCell,
}

pub struct ValidationContext {
    pub player_id: u64,
    pub grid_cell: GridCell,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ActionBaseData {
    pub player_id: u64,
    pub chunk: TerrainChunkId,
    pub cell: GridCell,
    // pub action_type: ActionType,

    pub start_time: u64,
    pub duration_ms: u64,
    pub completion_time: u64,

    pub status: ActionStatus,
}

pub trait SpecificActionData: Clone + Send + Sync {
    fn action_type(&self) -> ActionType;
    fn duration_ms(&self, context: &ActionContext) -> u64;
    fn validate(&self, context: &ValidationContext) -> Result<(), String>;
}

// ============ ACTIONS ============
// BuildBuilding
#[derive(Clone, Debug, Encode, Decode)]
pub struct BuildBuildingAction {
    pub player_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub building_type: BuildingType,
    // TODO: Add the recipe and resources used
}

impl SpecificActionData for BuildBuildingAction {
    fn action_type(&self) -> ActionType {
        ActionType::BuildBuilding
    }

    fn duration_ms(&self, context: &ActionContext) -> u64 {
        match self.building_type {
            _ => 15_000,
        }
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), String> {
        if matches!(self.building_type.category, BuildingCategory::Unknown) {
            return Err("building category cannot be unknown".to_string());
        }
        Ok(())
    }
}

// BuildBuilding
#[derive(Clone, Debug, Encode, Decode)]
pub struct BuildRoadAction {
    pub player_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

impl SpecificActionData for BuildRoadAction {
    fn action_type(&self) -> ActionType {
        ActionType::BuildBuilding
    }

    fn duration_ms(&self, context: &ActionContext) -> u64 {
        1_000
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), String> {
        Ok(())
    }
}

// MoveUnit
#[derive(Clone, Debug, Encode, Decode)]
pub struct MoveUnitAction {
    pub player_id: u64,
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

impl SpecificActionData for MoveUnitAction {
    fn action_type(&self) -> ActionType {
        ActionType::MoveUnit
    }

    fn duration_ms(&self, context: &ActionContext) -> u64 {
        // Distance-based duration
        let distance = context
            .grid_cell
            .to_hex()
            .distance_to(context.grid_cell.to_hex());
        (distance as u64) * 1000 // 1s par hex
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), String> {
        if self.unit_id == 0 {
            return Err("unit_id cannot be 0".to_string());
        }
        // Vérifier que la cible est accessible, etc.
        Ok(())
    }
}

// SendMessage
#[derive(Clone, Debug, Encode, Decode)]
pub struct SendMessageAction {
    pub player_id: u64,
    pub receivers: Vec<u64>,
    pub content: String,
}

impl SpecificActionData for SendMessageAction {
    fn action_type(&self) -> ActionType {
        ActionType::SendMessage
    }

    fn duration_ms(&self, _context: &ActionContext) -> u64 {
        500 // Quasi-instantané
    }

    fn validate(&self, _context: &ValidationContext) -> Result<(), String> {
        if self.receivers.len() == 0 {
            return Err("receivers cannot be 0".to_string());
        }
        if self.content.is_empty() {
            return Err("content cannot be empty".to_string());
        }
        Ok(())
    }
}

// HarvestResource
#[derive(Clone, Debug, Encode, Decode)]
pub struct HarvestResourceAction {
    pub player_id: u64,
    pub resource_type: ResourceType,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

impl SpecificActionData for HarvestResourceAction {
    fn action_type(&self) -> ActionType {
        ActionType::HarvestResource
    }

    fn duration_ms(&self, _context: &ActionContext) -> u64 {
        5_000
    }

    fn validate(&self, _context: &ValidationContext) -> Result<(), String> {
        // if self.resource_type {
        //     return Err("resource_type cannot be empty".to_string());
        // }
        Ok(())
    }
}

// CraftResource
#[derive(Clone, Debug, Encode, Decode)]
pub struct CraftResourceAction {
    pub player_id: u64,
    pub recipe_id: String,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub quantity: u32,
}

impl SpecificActionData for CraftResourceAction {
    fn action_type(&self) -> ActionType {
        ActionType::CraftResource
    }

    fn duration_ms(&self, _context: &ActionContext) -> u64 {
        (self.quantity as u64) * 2_000 // 2s par item
    }

    fn validate(&self, _context: &ValidationContext) -> Result<(), String> {
        if self.recipe_id.is_empty() {
            return Err("recipe_id cannot be empty".to_string());
        }
        if self.quantity == 0 {
            return Err("quantity must be > 0".to_string());
        }
        Ok(())
    }
}

// ============ ENUM UNIFIÉ ============

#[derive(Clone, Debug, Encode, Decode)]
pub enum SpecificAction {
    Unknown(),
    BuildBuilding(BuildBuildingAction),
    BuildRoad(BuildRoadAction),
    MoveUnit(MoveUnitAction),
    SendMessage(SendMessageAction),
    HarvestResource(HarvestResourceAction),
    CraftResource(CraftResourceAction),
}

impl SpecificAction {
    pub fn action_type(&self) -> ActionType {
        match self {
            Self::BuildBuilding(a) => a.action_type(),
            Self::BuildRoad(a) => a.action_type(),
            Self::MoveUnit(a) => a.action_type(),
            Self::SendMessage(a) => a.action_type(),
            Self::HarvestResource(a) => a.action_type(),
            Self::CraftResource(a) => a.action_type(),
            Self::Unknown() => ActionType::Unknown,
        }
    }
    
    pub fn duration_ms(&self, context: &ActionContext) -> u64 {
        match self {
            Self::BuildBuilding(a) => a.duration_ms(context),
            Self::BuildRoad(a) => a.duration_ms(context),
            Self::MoveUnit(a) => a.duration_ms(context),
            Self::SendMessage(a) => a.duration_ms(context),
            Self::HarvestResource(a) => a.duration_ms(context),
            Self::CraftResource(a) => a.duration_ms(context),
            Self::Unknown() => 5_000,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, sqlx::Type)]
#[sqlx(type_name="specific_action_type")]
pub enum SpecificActionType {
    BuildBuilding,
    BuildRoad,
    MoveUnit,
    SendMessage,
    HarvestResource,
    CraftResource,
    Unknown,
}

impl SpecificActionType {
    pub fn from_specific_action(specific: &SpecificAction) -> Self {
        match specific {
            SpecificAction::BuildBuilding(_) => SpecificActionType::BuildBuilding,
            SpecificAction::BuildRoad(_) => SpecificActionType::BuildRoad,
            SpecificAction::MoveUnit(_) => SpecificActionType::MoveUnit,
            SpecificAction::SendMessage(_) => SpecificActionType::SendMessage,
            SpecificAction::HarvestResource(_) => SpecificActionType::HarvestResource,
            SpecificAction::CraftResource(_) => SpecificActionType::CraftResource,
            SpecificAction::Unknown() => SpecificActionType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ActionData {
    pub base_data: ActionBaseData,
    pub specific_data: SpecificAction,
}
