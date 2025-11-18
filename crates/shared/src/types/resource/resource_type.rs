use crate::{BiomeType, ResourceCategory};
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct ResourceType {
    pub id: i32,
    pub category: ResourceCategory,
}

impl ResourceType {

    pub fn wood(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Wood,
        }
    }
    pub fn metal(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Metal,
        }
    }
    pub fn food(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Food,
        }
    }
    pub fn furniture(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Furniture,
        }
    }
    pub fn weaponry(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Weaponry,
        }
    }
    pub fn jewelry(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Jewelry,
        }
    }
    pub fn meat(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Meat,
        }
    }
    pub fn fruits(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Fruits,
        }
    }
    pub fn vegetables(id: i32) -> Self {
        Self {
            id,
            category: ResourceCategory::Vegetables,
        }
    }
}
