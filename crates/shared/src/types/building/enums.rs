use bincode::{Decode, Encode};

use crate::BiomeTypeEnum;

// ============ ENUMS RUST ============
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BuildingCategoryEnum {
    Unknown = 0,
    Natural = 1,
    Urbanism = 2,
    Cult = 3,
    Dwellings = 4,
    ManufacturingWorkshops = 5,
    Entertainment = 6,
    Agriculture = 7,
    AnimalBreeding = 8,
    Education = 9,
    Military = 10,
}

impl BuildingCategoryEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Natural),
            2 => Some(Self::Urbanism),
            3 => Some(Self::Cult),
            4 => Some(Self::Dwellings),
            5 => Some(Self::ManufacturingWorkshops),
            6 => Some(Self::Entertainment),
            7 => Some(Self::Agriculture),
            8 => Some(Self::AnimalBreeding),
            9 => Some(Self::Education),
            10 => Some(Self::Military),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BuildingSpecificTypeEnum {
    Unknown = 0,
    Tree = 1,
    ManufacturingWorkshop = 2,
}

impl BuildingSpecificTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Tree),
            2 => Some(Self::ManufacturingWorkshop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum TreeTypeEnum {
    Cedar = 1,
    Larch = 2,
    Oak = 3,
}

impl TreeTypeEnum {
    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Cedar => "Cedar",
            Self::Larch => "Larch",
            Self::Oak => "Oak",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Cedar => "cedar",
            Self::Larch => "larch",
            Self::Oak => "oak",
        }
    }

    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Cedar),
            2 => Some(Self::Larch),
            3 => Some(Self::Oak),
            _ => None,
        }
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "cedar" => Some(Self::Cedar),
            "larch" => Some(Self::Larch),
            "oak" => Some(Self::Oak),
            _ => None,
        }
    }

    pub fn iter() -> impl Iterator<Item = TreeTypeEnum> {
        [
            TreeTypeEnum::Cedar,
            TreeTypeEnum::Larch,
            TreeTypeEnum::Oak,
        ]
        .into_iter()
    }
    
    pub fn from_biome(biome: BiomeTypeEnum) -> Vec<TreeTypeEnum> {
        match biome {
            BiomeTypeEnum::Savanna => vec![],
            BiomeTypeEnum::Grassland => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::TropicalSeasonalForest => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::TropicalRainForest => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::TropicalDeciduousForest => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::TemperateRainForest => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::Wetland => vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak],
            BiomeTypeEnum::Taiga => vec![],
            _ => vec![]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum DwellingsTypeEnum {
    House = 1,
}

impl DwellingsTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::House),
            _ => None,
        }
    }
    
    pub fn iter() -> impl Iterator<Item = DwellingsTypeEnum> {
        [
            DwellingsTypeEnum::House,
        ]
        .into_iter()
    }
}

pub enum UrbanismTypeEnum {
    Path = 1,
    Road = 2,
    PavedRoad = 3,
    Avenue = 4,
}

impl UrbanismTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Path),
            2 => Some(Self::Road),
            3 => Some(Self::PavedRoad),
            4 => Some(Self::Avenue),
            _ => None,
        }
    }
    
    pub fn iter() -> impl Iterator<Item = UrbanismTypeEnum> {
        [
            UrbanismTypeEnum::Path,
            UrbanismTypeEnum::Road,
            UrbanismTypeEnum::PavedRoad,
            UrbanismTypeEnum::Avenue,
        ]
        .into_iter()
    }
}
