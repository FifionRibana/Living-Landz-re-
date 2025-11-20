use bincode::{Decode, Encode};
use sqlx::FromRow;
// use chrono::{DateTime, Utc};

// ============ LOOKUPS ============
#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ResourceCategory {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ResourceSpecificType {
    pub id: i16,
    pub name: String,
    pub category_id: i16,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct ResourceType {
    pub id: i32,
    pub name: String,
    pub category_id: i16,
    pub specific_type_id: i16,
    pub description: Option<String>,
    pub archived: bool,
}