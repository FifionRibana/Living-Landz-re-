use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::networking::events::ServerEvent;

/// Territory messages are now handled by lightyear (#137).
pub fn handle_territory_events(mut events: MessageReader<ServerEvent>) {
    for event in events.read() {
        match &event.0 {
            ServerMessage::HamletFounded { .. } => {}
            ServerMessage::HamletFoundError { .. } => {}
            ServerMessage::TerritoryContourUpdate { .. } => {}
            ServerMessage::TerritoryBorderSdfUpdate { .. } => {}
            ServerMessage::TerritoryBorderCells { .. } => {}
            ServerMessage::OrganizationAtCell { .. } => {}
            _ => {}
        }
    }
}
