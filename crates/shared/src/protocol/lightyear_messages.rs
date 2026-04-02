use serde::{Deserialize, Serialize};

use crate::grid::GridCell;
use crate::protocol::{
    CharacterData, GameDataPayload, PlayerData,
};
use crate::{
    ActionStatusEnum, ActionTypeEnum, BuildingTypeEnum, OrganizationSummary, ProfessionEnum,
    ResourceSpecificTypeEnum, TerrainChunkId, UnitData,
};

// ─── Client → Server Messages ───────────────────────────────────────

/// Client requests a unit move. Replaces ClientMessage::ActionMoveUnit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionMoveUnitMsg {
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
}

/// Client requests building construction. Replaces ClientMessage::ActionBuildBuilding.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionBuildBuildingMsg {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub building_type: BuildingTypeEnum,
}

/// Client requests road construction. Replaces ClientMessage::ActionBuildRoad.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionBuildRoadMsg {
    pub start_cell: GridCell,
    pub end_cell: GridCell,
}

/// Client requests resource harvesting. Replaces ClientMessage::ActionHarvestResource.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionHarvestResourceMsg {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub resource_specific_type: ResourceSpecificTypeEnum,
    pub unit_ids: Vec<u64>,
}

/// Client requests resource crafting. Replaces ClientMessage::ActionCraftResource.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionCraftResourceMsg {
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub recipe_id: String,
    pub quantity: u32,
    pub unit_ids: Vec<u64>,
}

/// Client requests unit training. Replaces ClientMessage::ActionTrainUnit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionTrainUnitMsg {
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub target_profession: ProfessionEnum,
}

/// Client requests exploration. Replaces ClientMessage::ActionExplore.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionExploreMsg {
    pub cell: GridCell,
    pub radius: i32,
}

// ─── Server → Client Messages ───────────────────────────────────────

/// Server reports action status (Pending, InProgress, Completed, Failed).
/// Replaces ServerMessage::ActionStatusUpdate for lightyear-routed actions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionStatusMsg {
    pub action_id: u64,
    pub player_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub status: ActionStatusEnum,
    pub action_type: ActionTypeEnum,
    pub completion_time: u64,
    pub action_name: Option<String>,
    pub unit_ids: Vec<u64>,
}

/// Server reports an action error.
/// Replaces ServerMessage::ActionError for lightyear-routed actions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionErrorMsg {
    pub reason: String,
}

/// Server notifies a non-lord unit moved.
/// Replaces ServerMessage::UnitPositionUpdated for lightyear-routed actions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitPositionUpdatedMsg {
    pub unit_id: u64,
    pub from_cell: GridCell,
    pub from_chunk: TerrainChunkId,
    pub to_cell: GridCell,
    pub to_chunk: TerrainChunkId,
}

/// Server notifies an action completed (broadcast to chunk).
/// Replaces ServerMessage::ActionCompleted for lightyear-routed actions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionCompletedMsg {
    pub action_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub action_type: ActionTypeEnum,
}

// ─── Post-login data Messages (server → client) ─────────────────────

/// Sent by server after lightyear connection is verified.
/// Contains player identity and character info.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginSuccessMsg {
    pub player: PlayerData,
    pub character: Option<CharacterData>,
}

/// Lord unit data — sent after connection.
/// None if the player hasn't created a lord yet.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LordDataMsg {
    pub lord: Option<UnitData>,
}

/// Player's organization data — sent after connection.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerOrganizationDataMsg {
    pub organization: Option<OrganizationSummary>,
}

/// Static game data (items, recipes, costs, yields) — sent once after connection.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameDataMsg {
    pub payload: GameDataPayload,
}