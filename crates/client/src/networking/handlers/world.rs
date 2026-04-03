use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// All world data messages are now handled by lightyear (#137).
pub fn handle_world_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::TerrainChunkData { .. } => {}
            ServerMessage::OceanData { .. } => {}
            ServerMessage::LakeData { .. } => {}
            ServerMessage::TerrainGlobalData { .. } => {}
            ServerMessage::RoadChunkSdfUpdate { .. } => {}
            ServerMessage::ExplorationMap { .. } => {}
            ServerMessage::ExplorationPatch { .. } => {}
            _ => {}
        }
    }
}
