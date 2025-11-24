// use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};
// use crate::types::*;
use crate::{
    BiomeChunkData, BuildingData, BuildingSpecificTypeEnum, BuildingTypeEnum,
    ResourceSpecificTypeEnum, TerrainChunkId,
    grid::{CellData, GridCell},
    types::TerrainChunkData,
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

/// Messages Client → Server
#[derive(Debug, Clone, Encode, Decode)]
pub enum ClientMessage {
    /// Initial connection
    Login {
        username: String,
        // password_hash: String,
    },
    RequestTerrainChunks {
        terrain_name: String,
        terrain_chunk_ids: Vec<TerrainChunkId>,
    },
    RequestTerrains {
        terrain_names: Vec<String>,
    },

    ActionBuildBuilding {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        building_type: BuildingTypeEnum,
    },
    ActionBuildRoad {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
    },
    ActionMoveUnit {
        player_id: u64,
        unit_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
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

    TerrainChunkData {
        terrain_chunk_data: TerrainChunkData,
        biome_chunk_data: Vec<BiomeChunkData>,
        cell_data: Vec<CellData>,
        building_data: Vec<BuildingData>,
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

    /// Pong (ping answer)
    Pong,
}
