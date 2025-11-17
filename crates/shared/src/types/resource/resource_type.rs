use crate::BiomeType;
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Encode, Decode)]
#[sqlx(type_name = "resource_type")]
pub enum ResourceType {
    Wood,
    Iron,
    Stone,
    Clay,
}

impl ResourceType {

    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn iter() -> impl Iterator<Item = ResourceType> {
        [
            ResourceType::Wood,
            ResourceType::Iron,
            ResourceType::Stone,
            ResourceType::Clay,
        ]
        .into_iter()
    }
}
