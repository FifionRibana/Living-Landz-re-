use bincode::{Decode, Encode};

use crate::{ResourceCategoryEnum, ResourceSpecificTypeEnum, ResourceType};


pub trait ResourceSpecificData: Clone {
    fn category(&self) -> ResourceCategoryEnum;
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WoodData {}

impl ResourceSpecificData for WoodData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Wood
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MetalData {}

impl ResourceSpecificData for MetalData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Metal
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RockData {}

impl ResourceSpecificData for RockData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::CrudeMaterial
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct OreData {}

impl ResourceSpecificData for OreData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::CrudeMaterial
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FoodData {}

impl ResourceSpecificData for FoodData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Food
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FurnitureData {}

impl ResourceSpecificData for FurnitureData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Furniture
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WeaponryData {}

impl ResourceSpecificData for WeaponryData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Weaponry
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct JewelryData {}

impl ResourceSpecificData for JewelryData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Jewelry
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MeatData {}

impl ResourceSpecificData for MeatData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Meat
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FruitsData {}

impl ResourceSpecificData for FruitsData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Fruits
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct VegetablesData {}

impl ResourceSpecificData for VegetablesData {
    fn category(&self) -> ResourceCategoryEnum {
        ResourceCategoryEnum::Vegetables
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
    pub fn category(&self) -> ResourceCategoryEnum {
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
            Self::Unknown() => ResourceCategoryEnum::Unknown,
        }
    }

    pub fn to_specific_type_id(&self) -> i16 {
        match self {
            Self::Wood(_) => 1,
            Self::Metal(_) => 2,
            Self::Rock(_) => 3,
            Self::Ore(_) => 4,
            Self::Food(_) => 5,
            Self::Furniture(_) => 6,
            Self::Weaponry(_) => 7,
            Self::Jewelry(_) => 8,
            Self::Meat(_) => 9,
            Self::Fruits(_) => 10,
            Self::Vegetables(_) => 11,
            Self::Unknown() => 0,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceBaseData {
    pub id: u64,
    pub resource_type: ResourceType,
    pub resource_specific_type_id: ResourceSpecificTypeEnum,

    pub created_at: u64,

    pub quality: Option<f32>,
    pub decay_rate: Option<f32>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceData {
    pub base_data: ResourceBaseData,
    pub specific_data: ResourceSpecific,
}
