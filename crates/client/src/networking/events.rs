// =============================================================================
// NETWORKING - Events
// =============================================================================

use bevy::prelude::*;

/// Thin wrapper around ServerMessage, fired by poll_server for all handlers to consume.
#[derive(Message, Debug)]
pub struct ServerEvent(pub shared::protocol::ServerMessage);
