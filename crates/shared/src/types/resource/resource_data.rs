use bincode::{Decode, Encode};

use crate::{ResourceCategory, ResourceType};

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceBaseData {
    pub id: u64,
    pub resource_type: ResourceType,

    pub created_at: u64,

    pub quality: f32,
    pub decay_rate: f32,
}

pub trait ResourceSpecificData: Clone {
    fn category(&self) -> ResourceCategory;
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WoodData {}

impl ResourceSpecificData for WoodData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Wood
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MetalData {}

impl ResourceSpecificData for MetalData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Metal
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RockData {}

impl ResourceSpecificData for RockData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::CrudeMaterial
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct OreData {}

impl ResourceSpecificData for OreData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::CrudeMaterial
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FoodData {}

impl ResourceSpecificData for FoodData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Food
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FurnitureData {}

impl ResourceSpecificData for FurnitureData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Furniture
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WeaponryData {}

impl ResourceSpecificData for WeaponryData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Weaponry
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct JewelryData {}

impl ResourceSpecificData for JewelryData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Jewelry
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MeatData {}

impl ResourceSpecificData for MeatData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Meat
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FruitsData {}

impl ResourceSpecificData for FruitsData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Fruits
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct VegetablesData {}

impl ResourceSpecificData for VegetablesData {
    fn category(&self) -> ResourceCategory {
        ResourceCategory::Vegetables
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum ResourceSpecific {
    Unknown(),
    Wood(WoodData),
    Metal(MetalData),
    Rock(RockData),
    Ore(OreData),
    Food(FoodData),
    Furniture(FurnitureData),
    Weaponry(WeaponryData),
    Jewelry(JewelryData),
    Meat(MeatData),
    Fruits(FruitsData),
    Vegetables(VegetablesData),
    //...
}

impl ResourceSpecific {
    pub fn category(&self) -> ResourceCategory {
        match self {
            Self::Wood(r) => r.category(),
            Self::Metal(r) => r.category(),
            Self::Rock(r) => r.category(),
            Self::Ore(r) => r.category(),
            Self::Food(r) => r.category(),
            Self::Furniture(r) => r.category(),
            Self::Weaponry(r) => r.category(),
            Self::Jewelry(r) => r.category(),
            Self::Meat(r) => r.category(),
            Self::Fruits(r) => r.category(),
            Self::Vegetables(r) => r.category(),
            Self::Unknown() => ResourceCategory::Unknown,
        }
    }
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "resource_specific_type")]
pub enum ResourceSpecificType {
    Wood,
    Metal,
    Rock,
    Ore,
    Food,
    Furniture,
    Weaponry,
    Jewelry,
    Meat,
    Fruits,
    Vegetables,
    Unknown,
}

impl ResourceSpecificType {
    pub fn from_resource_specific(specific: &ResourceSpecific) -> Self {
        match specific {
            ResourceSpecific::Wood(_) => ResourceSpecificType::Wood,
            ResourceSpecific::Metal(_) => ResourceSpecificType::Metal,
            ResourceSpecific::Rock(_) => ResourceSpecificType::Rock,
            ResourceSpecific::Ore(_) => ResourceSpecificType::Ore,
            ResourceSpecific::Food(_) => ResourceSpecificType::Food,
            ResourceSpecific::Furniture(_) => ResourceSpecificType::Furniture,
            ResourceSpecific::Weaponry(_) => ResourceSpecificType::Weaponry,
            ResourceSpecific::Jewelry(_) => ResourceSpecificType::Jewelry,
            ResourceSpecific::Meat(_) => ResourceSpecificType::Meat,
            ResourceSpecific::Fruits(_) => ResourceSpecificType::Fruits,
            ResourceSpecific::Vegetables(_) => ResourceSpecificType::Vegetables,
            ResourceSpecific::Unknown() => ResourceSpecificType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceData {
    pub base_data: ResourceBaseData,
    pub specific_data: ResourceSpecific,
}
