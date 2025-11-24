use crate::{
    AgricultureTypeEnum, AnimalBreedingTypeEnum, BuildingCategoryEnum,
    BuildingSpecificTypeEnum, CommerceTypeEnum, CultTypeEnum, EntertainmentTypeEnum,
    ManufacturingWorkshopTypeEnum, TerrainChunkId, TreeTypeEnum, grid::GridCell,
};
use bincode::{Decode, Encode};

pub trait BuildingSpecificData: Clone {
    fn category(&self) -> BuildingCategoryEnum;
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum BuildingSpecific {
    Unknown(),
    Tree(TreeData),
    ManufacturingWorkshop(ManufacturingWorkshopData),
    Agriculture(AgricultureData),
    AnimalBreeding(AnimalBreedingData),
    Entertainment(EntertainmentData),
    Cult(CultData),
    Commerce(CommerceData),
}

impl BuildingSpecific {
    pub fn category(&self) -> BuildingCategoryEnum {
        match self {
            Self::Tree(t) => t.category(),
            Self::ManufacturingWorkshop(w) => w.category(),
            Self::Agriculture(a) => a.category(),
            Self::AnimalBreeding(a) => a.category(),
            Self::Entertainment(e) => e.category(),
            Self::Cult(c) => c.category(),
            Self::Commerce(c) => c.category(),
            Self::Unknown() => BuildingCategoryEnum::Unknown,
        }
    }

    pub fn to_specific_type_id(&self) -> i16 {
        match self {
            Self::Tree(_) => 1,
            Self::ManufacturingWorkshop(_) => 2,
            Self::Agriculture(_) => 3,
            Self::AnimalBreeding(_) => 4,
            Self::Entertainment(_) => 5,
            Self::Cult(_) => 6,
            Self::Commerce(_) => 7,
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
pub struct ManufacturingWorkshopData {
    pub workshop_type: ManufacturingWorkshopTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for ManufacturingWorkshopData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::ManufacturingWorkshops
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AgricultureData {
    pub agriculture_type: AgricultureTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for AgricultureData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Agriculture
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AnimalBreedingData {
    pub animal_type: AnimalBreedingTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for AnimalBreedingData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::AnimalBreeding
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EntertainmentData {
    pub entertainment_type: EntertainmentTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for EntertainmentData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Entertainment
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CultData {
    pub cult_type: CultTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for CultData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Cult
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CommerceData {
    pub commerce_type: CommerceTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for CommerceData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Commerce
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
