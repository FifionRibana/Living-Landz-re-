use bincode::{Decode, Encode};
use sqlx::prelude::FromRow;

// ============ LOOKUPS ============
#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ActionStatus {
    pub id: i16,
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ActionType {
    pub id: i16,
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ActionSpecificType {
    pub id: i16,
    pub name: String,
    pub archived: bool,
}