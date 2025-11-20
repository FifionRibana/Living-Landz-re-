use bincode::{Decode, Encode};
use sqlx::prelude::FromRow;

use crate::{
    ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum, BuildingCategoryEnum,
    BuildingSpecific, BuildingSpecificTypeEnum, ResourceSpecificTypeEnum, TerrainChunkId,
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

pub trait SpecificActionData: Clone + Send + Sync {
    fn action_type(&self) -> ActionTypeEnum;
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
    pub building_specific_type: BuildingSpecificTypeEnum,
    // TODO: Add the recipe and resources used
}

impl SpecificActionData for BuildBuildingAction {
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::BuildBuilding
    }

    fn duration_ms(&self, context: &ActionContext) -> u64 {
        match self.building_specific_type {
            _ => 15_000,
        }
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), String> {
        if matches!(
            self.building_specific_type,
            BuildingSpecificTypeEnum::Unknown
        ) {
            return Err("building type cannot be unknown".to_string());
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
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::BuildBuilding
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
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::MoveUnit
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

// Structure pour représenter un receiver en DB
#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct SendMessageReceiver {
    pub id: i64,
    pub action_id: i64,
    pub receiver_id: i64,
}

impl SpecificActionData for SendMessageAction {
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::SendMessage
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
    pub resource_specific_type: ResourceSpecificTypeEnum,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

impl SpecificActionData for HarvestResourceAction {
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::HarvestResource
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
    fn action_type(&self) -> ActionTypeEnum {
        ActionTypeEnum::CraftResource
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
    pub fn to_specific_type_id(&self) -> i16 {
        match self {
            Self::BuildBuilding(_) => 1,
            Self::BuildRoad(_) => 2,
            Self::MoveUnit(_) => 3,
            Self::SendMessage(_) => 4,
            Self::HarvestResource(_) => 5,
            Self::CraftResource(_) => 6,
            Self::Unknown() => 0,
        }
    }

    pub fn action_type(&self) -> ActionTypeEnum {
        match self {
            Self::BuildBuilding(a) => a.action_type(),
            Self::BuildRoad(a) => a.action_type(),
            Self::MoveUnit(a) => a.action_type(),
            Self::SendMessage(a) => a.action_type(),
            Self::HarvestResource(a) => a.action_type(),
            Self::CraftResource(a) => a.action_type(),
            Self::Unknown() => ActionTypeEnum::Unknown,
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

// ============ SCHEDULED ACTION (EN BASE) ============
#[derive(Clone, Debug, Encode, Decode)]
pub struct ActionBaseData {
    pub player_id: u64,
    pub chunk: TerrainChunkId,
    pub cell: GridCell,

    pub action_type: ActionTypeEnum,
    pub action_specific_type: ActionSpecificTypeEnum,

    pub start_time: u64,
    pub duration_ms: u64,
    pub completion_time: u64,

    pub status: ActionStatusEnum,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ActionData {
    pub base_data: ActionBaseData,
    pub specific_data: SpecificAction,
}
