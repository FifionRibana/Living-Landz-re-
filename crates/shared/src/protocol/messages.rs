// use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};
// use crate::types::*;
use crate::{
    BiomeChunkData, BuildingData, BuildingSpecificTypeEnum, ResourceSpecificTypeEnum,
    TerrainChunkId,
    grid::{CellData, GridCell},
    types::TerrainChunkData,
};

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
        building_specific_type: BuildingSpecificTypeEnum,
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
        player_id: u64,
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

    /// Pong (ping answer)
    Pong,
}
