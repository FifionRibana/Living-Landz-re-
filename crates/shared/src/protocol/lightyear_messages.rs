use serde::{Deserialize, Serialize};

use crate::grid::GridCell;
use crate::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId};

// ─── Client → Server Messages ───────────────────────────────────────

/// Client requests a unit move. Replaces ClientMessage::ActionMoveUnit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActionMoveUnitMsg {
    pub unit_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
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