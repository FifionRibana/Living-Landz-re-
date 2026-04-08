use serde::{Deserialize, Serialize};

use crate::grid::GridCell;
use crate::protocol::{
    CharacterData, ColorData, GameDataPayload, InventoryItemData, PlayerData,
    TerritoryContourChunkData,
};
use crate::{
    ActionStatusEnum, ActionTypeEnum, BuildingTypeEnum, OrganizationSummary, ProfessionEnum,
    ResourceSpecificTypeEnum, RoadChunkSdfData, SlotPosition, TerritoryBorderChunkSdfData,
    TerrainChunkId, UnitData,
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

// ─── Bulk data Messages ─────────────────────────────────────────────
// Bulk terrain/ocean/lake/global data now travels via HTTP (see #193).
// Only exploration patches remain on lightyear (incremental updates).

/// Internal struct used by the client to buffer terrain chunk data from HTTP.
/// Not a lightyear message — kept here for shared access between game_client and streaming.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainChunkDataMsg {
    pub chunk_id: TerrainChunkId,
    pub compressed_data: Vec<u8>,
}

/// Incremental exploration patch — sent on ExplorationChannel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExplorationPatchMsg {
    pub patch_x: i32,
    pub patch_y: i32,
    pub patch_width: i32,
    pub patch_height: i32,
    pub compressed_data: Vec<u8>,
}

// ─── Client → Server request Messages ────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RequestInventoryMsg {
    pub unit_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RequestOrganizationAtCellMsg {
    pub cell: GridCell,
}

// ─── Server → Client event Messages (#137 Step 2) ───────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventoryDataMsg {
    pub unit_id: u64,
    pub items: Vec<InventoryItemData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InventoryUpdateMsg {
    pub unit_id: u64,
    pub item_id: i32,
    pub quantity_delta: i32,
    pub new_total: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitProfessionChangedMsg {
    pub unit_id: u64,
    pub new_profession: ProfessionEnum,
    pub new_avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitWorkStatusUpdateMsg {
    pub unit_id: u64,
    pub working_on_action_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoadChunkSdfUpdateMsg {
    pub terrain_name: String,
    pub chunk_id: TerrainChunkId,
    pub road_sdf_data: RoadChunkSdfData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerritoryContourUpdateMsg {
    pub chunk_id: TerrainChunkId,
    pub contours: Vec<TerritoryContourChunkData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerritoryBorderSdfUpdateMsg {
    pub chunk_id: TerrainChunkId,
    pub border_sdf_data_list: Vec<TerritoryBorderChunkSdfData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TerritoryBorderCellsMsg {
    pub organization_id: u64,
    pub border_cells: Vec<GridCell>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PopulationChangedMsg {
    pub organization_id: u64,
    pub new_population: i32,
    pub named_unit_count: i32,
    pub population_capacity: i32,
    pub immigrant: Option<UnitData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HamletFoundedMsg {
    pub organization_id: u64,
    pub name: String,
    pub headquarters: GridCell,
    pub territory_cells: Vec<GridCell>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HamletFoundErrorMsg {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrganizationAtCellMsg {
    pub cell: GridCell,
    pub organization: Option<OrganizationSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitSlotUpdatedMsg {
    pub unit_id: u64,
    pub cell: GridCell,
    pub slot_position: Option<SlotPosition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugOrganizationCreatedMsg {
    pub organization_id: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugOrganizationDeletedMsg {
    pub organization_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DebugUnitSpawnedMsg {
    pub unit_data: UnitData,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugErrorMsg {
    pub reason: String,
}

// ─── Remaining client → server commands (migrated from tungstenite) ──

/// Client requests lord creation (character creation flow).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateLordMsg {
    pub first_name: String,
    pub gender: String,
    pub portrait_layers: String,
}

/// Client requests hamlet founding.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FoundHamletMsg;

/// Client moves a unit from one slot to another on the same cell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MoveUnitToSlotMsg {
    pub unit_id: u64,
    pub cell: GridCell,
    pub from_slot: SlotPosition,
    pub to_slot: SlotPosition,
}

/// Client assigns a unit to a slot on a cell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AssignUnitToSlotMsg {
    pub unit_id: u64,
    pub cell: GridCell,
    pub slot: SlotPosition,
}

/// Client sends a chat/action message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SendMessageMsg {
    pub content: String,
}

/// Debug: create organization.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugCreateOrganizationMsg {
    pub name: String,
    pub organization_type: crate::OrganizationType,
    pub cell: GridCell,
    pub parent_organization_id: Option<u64>,
}

/// Debug: delete organization.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugDeleteOrganizationMsg {
    pub organization_id: u64,
}

/// Debug: spawn unit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugSpawnUnitMsg {
    pub cell: GridCell,
}

/// Server response: lord created successfully.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LordCreatedMsg {
    pub unit_data: UnitData,
}

/// Server response: lord creation failed.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LordCreateErrorMsg {
    pub reason: String,
}