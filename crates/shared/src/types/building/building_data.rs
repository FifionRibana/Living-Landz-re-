use crate::{BuildingCategoryEnum, BuildingSpecificTypeEnum, TerrainChunkId, TreeTypeEnum, grid::GridCell};
use bincode::{Decode, Encode};

pub trait BuildingSpecificData: Clone {
    fn category(&self) -> BuildingCategoryEnum;
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum BuildingSpecific {
    Unknown(),
    Tree(TreeData),
    ManufacturingWorkshop(ManufacturingWorkshopData),
    // ...
}

impl BuildingSpecific {
    pub fn category(&self) -> BuildingCategoryEnum {
        match self {
            Self::Tree(t) => t.category(),
            Self::ManufacturingWorkshop(w) => w.category(),
            Self::Unknown() => BuildingCategoryEnum::Unknown,
        }
    }

    pub fn to_specific_type_id(&self) -> i16 {
        match self {
            Self::Tree(_) => 1,
            Self::ManufacturingWorkshop(_) => 2,
            Self::Unknown() => 0,
        }
    }
}

// BUILDINGS
#[derive(Debug, Clone, Encode, Decode)]
pub struct TreeData {
    pub density: f32,
    pub age: i32,
    pub tree_type: TreeTypeEnum,
    pub variant: i32,
}

impl BuildingSpecificData for TreeData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Natural
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ManufacturingWorkshopData {}

impl BuildingSpecificData for ManufacturingWorkshopData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::ManufacturingWorkshops
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BuildingBaseData {
    pub id: u64,
    pub category: BuildingCategoryEnum,
    pub specific_type: BuildingSpecificTypeEnum,
    pub chunk: TerrainChunkId,
    pub cell: GridCell,

    pub created_at: u64,

    pub quality: f32,
    pub durability: f32,
    pub damage: f32,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BuildingData {
    pub base_data: BuildingBaseData,
    pub specific_data: BuildingSpecific,
}
