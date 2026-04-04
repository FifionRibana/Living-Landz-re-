use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// === Replicated Components ===

/// Lord position — the only replicated component for the prototype
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LordPosition {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub cell_q: i32,
    pub cell_r: i32,
}

/// Identifies which player owns this entity
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OwnedByPlayer(pub u64);

/// Position of a unit currently in transit (MoveUnit action InProgress).
/// The entity exists only while the unit is moving, and is despawned on completion.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MovingUnitPosition {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub cell_q: i32,
    pub cell_r: i32,
}

/// DB unit ID for a replicated moving unit entity.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MovingUnitId(pub u64);