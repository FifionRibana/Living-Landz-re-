use bincode::{Decode, Encode};
// use crate::types::*;
use crate::{
    BiomeChunkData, BuildingData, BuildingTypeEnum, ContourSegmentData, OceanData, OrganizationSummary, OrganizationType, ResourceSpecificTypeEnum, RoadChunkSdfData, SlotPosition, TerrainChunkId, UnitData, grid::{CellData, GridCell}, types::TerrainChunkData
};

/// Simplified Player data for network protocol (without timestamps)
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerData {
    pub id: i64,
    pub family_name: String,
    pub language_id: i16,
    pub coat_of_arms_id: Option<i64>,
    pub motto: Option<String>,
    pub origin_location: String,
}

/// Simplified Character data for network protocol (without timestamps)
#[derive(Debug, Clone, Encode, Decode)]
pub struct CharacterData {
    pub id: i64,
    pub player_id: i64,
    pub first_name: String,
    pub family_name: String,
    pub second_name: Option<String>,
    pub nickname: Option<String>,
    pub coat_of_arms_id: Option<i64>,
    pub image_id: Option<i64>,
    pub motto: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorData {
    pub fn from_array(color: [f32; 4]) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Territory contour data for a specific organization in a specific chunk
#[derive(Debug, Clone, Encode, Decode)]
pub struct TerritoryContourChunkData {
    pub organization_id: u64,
    pub chunk_id: TerrainChunkId,
    /// Contour segments: [start.x, start.y, end.x, end.y, normal.x, normal.y, ...]
    /// Flattened array of ContourSegment data (6 floats per segment)
    pub segments: Vec<ContourSegmentData>,
    /// Border color (RGBA)
    pub border_color: ColorData,
    /// Fill color (RGBA)
    pub fill_color: ColorData,
}

/// Messages Client → Server
#[derive(Debug, Clone, Encode, Decode)]
pub enum ClientMessage {
    /// Initial connection (legacy - kept for backward compatibility)
    Login {
        username: String,
        // password_hash: String,
    },

    /// Register a new account with password
    RegisterAccount {
        family_name: String,
        password: String,
    },

    /// Login with password authentication
    LoginWithPassword {
        family_name: String,
        password: String,
    },
    RequestTerrainChunks {
        terrain_name: String,
        terrain_chunk_ids: Vec<TerrainChunkId>,
    },
    RequestTerrains {
        terrain_names: Vec<String>,
    },
    RequestOceanData {
        world_name: String,
    },

    ActionBuildBuilding {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        building_type: BuildingTypeEnum,
    },
    ActionBuildRoad {
        player_id: u64,
        start_cell: GridCell,
        end_cell: GridCell,
    },
    ActionMoveUnit {
        player_id: u64,
        unit_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
    },
    /// Move a unit to a specific slot within a cell
    MoveUnitToSlot {
        unit_id: u64,
        cell: GridCell,
        from_slot: SlotPosition,
        to_slot: SlotPosition,
    },
    /// Assign a unit to a slot (initial assignment, no previous slot)
    AssignUnitToSlot {
        unit_id: u64,
        cell: GridCell,
        slot: SlotPosition,
    },
    ActionSendMessage {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        receivers: Vec<u64>,
        content: String,
    },
    ActionHarvestResource {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        resource_specific_type: ResourceSpecificTypeEnum,
    },
    ActionCraftResource {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        recipe_id: String,
        quantity: u32,
    },

    // ========================================================================
    // DEBUG COMMANDS
    // ========================================================================
    /// Debug: Create an organization at a specific cell
    DebugCreateOrganization {
        name: String,
        organization_type: OrganizationType,
        cell: GridCell,
        parent_organization_id: Option<u64>,
    },

    /// Debug: Delete an organization
    DebugDeleteOrganization {
        organization_id: u64,
    },

    /// Debug: Spawn a random unit at a cell
    DebugSpawnUnit {
        cell: GridCell,
    },

    /// Debug: Regenerate territory contours for all organizations
    DebugRegenerateAllContours,

    /// Request organization info for a cell
    RequestOrganizationAtCell {
        cell: GridCell,
    },

    /// Ping (keep alive)
    Ping,
}

/// Messages Server → Client
#[derive(Debug, Clone, Encode, Decode)]
pub enum ServerMessage {
    /// Connection acknowledgement
    LoginSuccess {
        player: PlayerData,
        character: Option<CharacterData>,
    },

    /// Connection error
    LoginError {
        reason: String,
    },

    /// Registration successful
    RegisterSuccess {
        message: String,
    },

    /// Registration failed
    RegisterError {
        reason: String,
    },

    TerrainChunkData {
        terrain_chunk_data: TerrainChunkData,
        biome_chunk_data: Vec<BiomeChunkData>,
        cell_data: Vec<CellData>,
        building_data: Vec<BuildingData>,
        unit_data: Vec<UnitData>,
    },

    // OrganizationData {
    //     organization_data: OrganizationData,
    // },
    OceanData {
        ocean_data: OceanData,
    },

    /// Road SDF data update for a specific chunk (sent separately to avoid message size limits)
    RoadChunkSdfUpdate {
        terrain_name: String,
        chunk_id: TerrainChunkId,
        road_sdf_data: RoadChunkSdfData,
    },

    /// Territory contour data update for a specific chunk (contains all organizations with borders in this chunk)
    TerritoryContourUpdate {
        chunk_id: TerrainChunkId,
        contours: Vec<TerritoryContourChunkData>,
    },

    /// [DEPRECATED] Territory border SDF data update - replaced by TerritoryContourUpdate
    TerritoryBorderSdfUpdate {
        chunk_id: TerrainChunkId,
        /// Multiple SDFs, one per organization in this chunk
        border_sdf_data_list: Vec<crate::TerritoryBorderChunkSdfData>,
    },

    /// Territory border cells for debugging (cells at the frontier of territories)
    TerritoryBorderCells {
        organization_id: u64,
        border_cells: Vec<GridCell>,
    },

    ActionSuccess {
        command_id: u64,
    },

    ActionError {
        reason: String,
    },

    /// Action status update sent to the player who initiated the action
    ActionStatusUpdate {
        action_id: u64,
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        status: crate::ActionStatusEnum,
        action_type: crate::ActionTypeEnum,
        completion_time: u64,
    },

    /// Action result broadcast to all players in the chunk after completion
    ActionCompleted {
        action_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        action_type: crate::ActionTypeEnum,
    },

    /// Unit slot position updated (broadcast to all clients viewing the cell)
    UnitSlotUpdated {
        unit_id: u64,
        cell: GridCell,
        slot_position: Option<SlotPosition>,
    },

    // ========================================================================
    // DEBUG RESPONSES
    // ========================================================================
    /// Debug: Organization created successfully
    DebugOrganizationCreated {
        organization_id: u64,
        name: String,
    },

    /// Debug: Organization deleted successfully
    DebugOrganizationDeleted {
        organization_id: u64,
    },

    /// Debug: Unit spawned successfully
    DebugUnitSpawned {
        unit_data: UnitData,
    },

    /// Response with organization info at a cell
    OrganizationAtCell {
        cell: GridCell,
        organization: Option<OrganizationSummary>,
    },

    /// Debug error
    DebugError {
        reason: String,
    },

    /// Pong (ping answer)
    Pong,
}
