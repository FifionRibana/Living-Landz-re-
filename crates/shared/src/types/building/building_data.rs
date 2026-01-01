use crate::{
    AgricultureTypeEnum, AnimalBreedingTypeEnum, BuildingCategoryEnum,
    BuildingSpecificTypeEnum, CommerceTypeEnum, CultTypeEnum, EntertainmentTypeEnum,
    ManufacturingWorkshopTypeEnum, TerrainChunkId, TreeTypeEnum, grid::GridCell,
};
use bincode::{Decode, Encode};

pub trait BuildingSpecificData: Clone {
    fn category(&self) -> BuildingCategoryEnum;
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
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
#[derive(Debug, Copy, Clone, Encode, Decode)]
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

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct ManufacturingWorkshopData {
    pub workshop_type: ManufacturingWorkshopTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for ManufacturingWorkshopData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::ManufacturingWorkshops
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct AgricultureData {
    pub agriculture_type: AgricultureTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for AgricultureData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Agriculture
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct AnimalBreedingData {
    pub animal_type: AnimalBreedingTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for AnimalBreedingData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::AnimalBreeding
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct EntertainmentData {
    pub entertainment_type: EntertainmentTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for EntertainmentData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Entertainment
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct CultData {
    pub cult_type: CultTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for CultData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Cult
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct CommerceData {
    pub commerce_type: CommerceTypeEnum,
    pub variant: u32,
}

impl BuildingSpecificData for CommerceData {
    fn category(&self) -> BuildingCategoryEnum {
        BuildingCategoryEnum::Commerce
    }
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
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

#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct BuildingData {
    pub base_data: BuildingBaseData,
    pub specific_data: BuildingSpecific,
}

impl BuildingData {
    /// Convert BuildingData to BuildingTypeEnum for slot configuration
    pub fn to_building_type(&self) -> Option<crate::BuildingTypeEnum> {
        use crate::BuildingTypeEnum;

        match &self.specific_data {
            BuildingSpecific::ManufacturingWorkshop(data) => {
                use crate::ManufacturingWorkshopTypeEnum;
                match data.workshop_type {
                    ManufacturingWorkshopTypeEnum::Blacksmith => Some(BuildingTypeEnum::Blacksmith),
                    ManufacturingWorkshopTypeEnum::BlastFurnace => Some(BuildingTypeEnum::BlastFurnace),
                    ManufacturingWorkshopTypeEnum::Bloomery => Some(BuildingTypeEnum::Bloomery),
                    ManufacturingWorkshopTypeEnum::CarpenterShop => Some(BuildingTypeEnum::CarpenterShop),
                    ManufacturingWorkshopTypeEnum::GlassFactory => Some(BuildingTypeEnum::GlassFactory),
                }
            }
            BuildingSpecific::Agriculture(data) => {
                use crate::AgricultureTypeEnum;
                match data.agriculture_type {
                    AgricultureTypeEnum::Farm => Some(BuildingTypeEnum::Farm),
                }
            }
            BuildingSpecific::AnimalBreeding(data) => {
                use crate::AnimalBreedingTypeEnum;
                match data.animal_type {
                    AnimalBreedingTypeEnum::Cowshed => Some(BuildingTypeEnum::Cowshed),
                    AnimalBreedingTypeEnum::Piggery => Some(BuildingTypeEnum::Piggery),
                    AnimalBreedingTypeEnum::Sheepfold => Some(BuildingTypeEnum::Sheepfold),
                    AnimalBreedingTypeEnum::Stable => Some(BuildingTypeEnum::Stable),
                }
            }
            BuildingSpecific::Entertainment(data) => {
                use crate::EntertainmentTypeEnum;
                match data.entertainment_type {
                    EntertainmentTypeEnum::Theater => Some(BuildingTypeEnum::Theater),
                }
            }
            BuildingSpecific::Cult(data) => {
                use crate::CultTypeEnum;
                match data.cult_type {
                    CultTypeEnum::Temple => Some(BuildingTypeEnum::Temple),
                }
            }
            BuildingSpecific::Commerce(data) => {
                use crate::CommerceTypeEnum;
                match data.commerce_type {
                    CommerceTypeEnum::Bakehouse => Some(BuildingTypeEnum::Bakehouse),
                    CommerceTypeEnum::Brewery => Some(BuildingTypeEnum::Brewery),
                    CommerceTypeEnum::Distillery => Some(BuildingTypeEnum::Distillery),
                    CommerceTypeEnum::Slaughterhouse => Some(BuildingTypeEnum::Slaughterhouse),
                    CommerceTypeEnum::IceHouse => Some(BuildingTypeEnum::IceHouse),
                    CommerceTypeEnum::Market => Some(BuildingTypeEnum::Market),
                }
            }
            BuildingSpecific::Tree(data) => {
                Some(data.tree_type.to_building_type())
            }
            BuildingSpecific::Unknown() => None,
        }
    }
}
