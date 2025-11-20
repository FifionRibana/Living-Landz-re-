use bincode::{Decode, Encode};
use sqlx::FromRow;

// ============ LOOKUPS ============
#[derive(Debug, Clone, Encode, Decode, FromRow)]
pub struct BiomeType {
    pub id: i16,
    pub name: String,
}