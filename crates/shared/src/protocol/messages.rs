// use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
// use crate::types::*;
use crate::{BiomeChunkData, TerrainChunkId, types::TerrainChunkData};

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
    },
    
    /// Pong (ping answer)
    Pong,
}
