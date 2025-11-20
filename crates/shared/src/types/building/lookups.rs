use bincode::{Decode, Encode};
use sqlx::prelude::FromRow;

// ============ LOOKUPS ============
#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct BuildingCategory {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct BuildingSpecificType {
    pub id: i16,
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct BuildingType {
    pub id: i32,
    pub name: String,
    pub category_id: i16,
    pub specific_type_id: i16,
    pub description: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct TreeType {
    pub id: i16,
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct DwellingsType {
    pub id: i16,
    pub name: String,
    pub archived: bool
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct UrbanismType {
    pub id: i16,
    pub name: String,
    pub archived: bool
}

#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct WorkshopType {
    pub id: i16,
    pub name: String,
    pub archived: bool
}