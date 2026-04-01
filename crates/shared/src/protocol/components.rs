use bevy::prelude::*;
use lightyear::prelude::*;
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
pub struct OwnedByPlayer(pub PeerId);
