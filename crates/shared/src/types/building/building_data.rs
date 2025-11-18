use crate::{BuildingCategory, BuildingType, TerrainChunkId, TreeType, grid::GridCell};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct BuildingBaseData {
    pub id: u64,
    pub building_type: BuildingType,
    pub chunk: TerrainChunkId,
    pub cell: GridCell,

    pub created_at: u64,

    pub quality: f32,
    pub durability: f32,
    pub damage: f32,
}

pub trait BuildingSpecificData: Clone {
    fn category(&self) -> BuildingCategory;
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TreeData {
    pub density: f32,
    pub age: i32,
    pub tree_type: TreeType,
    pub variant: i32,
}

impl BuildingSpecificData for TreeData {
    fn category(&self) -> BuildingCategory {
        BuildingCategory::Natural
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ManufacturingWorkshopData {
}

impl BuildingSpecificData for ManufacturingWorkshopData {
    fn category(&self) -> BuildingCategory {
        BuildingCategory::ManufacturingWorkshops
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum BuildingSpecific {
    Unknown(),
    Tree(TreeData),
    ManufacturingWorkshop(ManufacturingWorkshopData),
    // ...
}

impl BuildingSpecific {
    pub fn category(&self) -> BuildingCategory {
        match self {
            Self::Tree(t) => t.category(),
            Self::ManufacturingWorkshop(w) => w.category(),
            Self::Unknown() => BuildingCategory::Unknown,
        }
    }
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "building_specific_type")]
pub enum BuildingSpecificType {
    Tree,
    ManufacturingWorkshops,
    Unknown,
}

impl BuildingSpecificType {
    pub fn from_building_specific(specific: &BuildingSpecific) -> Self {
        match specific {
            BuildingSpecific::Tree(_) => BuildingSpecificType::Tree,
            BuildingSpecific::ManufacturingWorkshop(_) => BuildingSpecificType::ManufacturingWorkshops,
            BuildingSpecific::Unknown() => BuildingSpecificType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BuildingData {
    pub base_data: BuildingBaseData,
    pub specific_data: BuildingSpecific,
}
