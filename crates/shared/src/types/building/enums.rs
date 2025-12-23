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
    Commerce = 11,
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
            11 => Some(Self::Commerce),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum BuildingSpecificTypeEnum {
    Unknown = 0,
    Tree = 1,
    ManufacturingWorkshop = 2,
    Agriculture = 3,
    AnimalBreeding = 4,
    Entertainment = 5,
    Cult = 6,
    Commerce = 7,
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
            3 => Some(Self::Agriculture),
            4 => Some(Self::AnimalBreeding),
            5 => Some(Self::Entertainment),
            6 => Some(Self::Cult),
            7 => Some(Self::Commerce),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Tree => "Tree",
            Self::ManufacturingWorkshop => "ManufacturingWorkshop",
            Self::Agriculture => "Agriculture",
            Self::AnimalBreeding => "AnimalBreeding",
            Self::Entertainment => "Entertainment",
            Self::Cult => "Cult",
            Self::Commerce => "Commerce",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Tree => "tree",
            Self::ManufacturingWorkshop => "manufacturing_workshop",
            Self::Agriculture => "agriculture",
            Self::AnimalBreeding => "animal_breeding",
            Self::Entertainment => "entertainment",
            Self::Cult => "cult",
            Self::Commerce => "commerce",
        }
    }

    pub fn iter() -> impl Iterator<Item = BuildingSpecificTypeEnum> {
        [
            BuildingSpecificTypeEnum::Unknown,
            BuildingSpecificTypeEnum::Tree,
            BuildingSpecificTypeEnum::ManufacturingWorkshop,
            BuildingSpecificTypeEnum::Agriculture,
            BuildingSpecificTypeEnum::AnimalBreeding,
            BuildingSpecificTypeEnum::Entertainment,
            BuildingSpecificTypeEnum::Cult,
            BuildingSpecificTypeEnum::Commerce,
        ]
        .into_iter()
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
        [TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak].into_iter()
    }

    pub fn from_biome(biome: BiomeTypeEnum) -> Vec<TreeTypeEnum> {
        match biome {
            BiomeTypeEnum::Savanna => vec![],
            BiomeTypeEnum::Grassland => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::TropicalSeasonalForest => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::TropicalRainForest => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::TropicalDeciduousForest => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::TemperateRainForest => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::Wetland => {
                vec![TreeTypeEnum::Cedar, TreeTypeEnum::Larch, TreeTypeEnum::Oak]
            }
            BiomeTypeEnum::Taiga => vec![],
            _ => vec![],
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Cedar => BuildingTypeEnum::Cedar,
            Self::Larch => BuildingTypeEnum::Larch,
            Self::Oak => BuildingTypeEnum::Oak,
        }
    }

    pub fn from_building_type(building_type: BuildingTypeEnum) -> Option<Self> {
        match building_type {
            BuildingTypeEnum::Cedar => Some(Self::Cedar),
            BuildingTypeEnum::Larch => Some(Self::Larch),
            BuildingTypeEnum::Oak => Some(Self::Oak),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum BuildingTypeEnum {
    // ManufacturingWorkshops
    Blacksmith = 1,
    BlastFurnace = 2,
    Bloomery = 3,
    CarpenterShop = 4,
    GlassFactory = 5,
    // Agriculture
    Farm = 10,
    // AnimalBreeding
    Cowshed = 20,
    Piggery = 21,
    Sheepfold = 22,
    Stable = 23,
    // Entertainment
    Theater = 30,
    // Cult
    Temple = 40,
    // Commerce
    Bakehouse = 50,
    Brewery = 51,
    Distillery = 52,
    Slaughterhouse = 53,
    IceHouse = 54,
    Market = 55,
    // Natural - Trees (1000+ range)
    Cedar = 1001,
    Larch = 1002,
    Oak = 1003,
}

// Unknown
// Tree
// ManufacturingWorkshop
// Agriculture
// AnimalBreeding
// Entertainment
// Cult
// Commerce
impl BuildingTypeEnum {
    pub fn to_specific_type(&self) -> BuildingSpecificTypeEnum {
        match self {
            Self::Blacksmith => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            Self::BlastFurnace => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            Self::Bloomery => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            Self::CarpenterShop => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            Self::GlassFactory => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            Self::Farm => BuildingSpecificTypeEnum::Agriculture,
            Self::Cowshed => BuildingSpecificTypeEnum::AnimalBreeding,
            Self::Piggery => BuildingSpecificTypeEnum::AnimalBreeding,
            Self::Sheepfold => BuildingSpecificTypeEnum::AnimalBreeding,
            Self::Stable => BuildingSpecificTypeEnum::AnimalBreeding,
            Self::Theater => BuildingSpecificTypeEnum::Entertainment,
            Self::Temple => BuildingSpecificTypeEnum::Cult,
            Self::Bakehouse => BuildingSpecificTypeEnum::Commerce,
            Self::Brewery => BuildingSpecificTypeEnum::Commerce,
            Self::Distillery => BuildingSpecificTypeEnum::Commerce,
            Self::Slaughterhouse => BuildingSpecificTypeEnum::Commerce,
            Self::IceHouse => BuildingSpecificTypeEnum::Commerce,
            Self::Market => BuildingSpecificTypeEnum::Commerce,
            Self::Cedar => BuildingSpecificTypeEnum::Tree,
            Self::Larch => BuildingSpecificTypeEnum::Tree,
            Self::Oak => BuildingSpecificTypeEnum::Tree,
        }
    }

    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Blacksmith),
            2 => Some(Self::BlastFurnace),
            3 => Some(Self::Bloomery),
            4 => Some(Self::CarpenterShop),
            5 => Some(Self::GlassFactory),
            10 => Some(Self::Farm),
            20 => Some(Self::Cowshed),
            21 => Some(Self::Piggery),
            22 => Some(Self::Sheepfold),
            23 => Some(Self::Stable),
            30 => Some(Self::Theater),
            40 => Some(Self::Temple),
            50 => Some(Self::Bakehouse),
            51 => Some(Self::Brewery),
            52 => Some(Self::Distillery),
            53 => Some(Self::Slaughterhouse),
            54 => Some(Self::IceHouse),
            55 => Some(Self::Market),
            1001 => Some(Self::Cedar),
            1002 => Some(Self::Larch),
            1003 => Some(Self::Oak),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Blacksmith => "blacksmith",
            Self::BlastFurnace => "blast_furnace",
            Self::Bloomery => "bloomery",
            Self::CarpenterShop => "carpenter_shop",
            Self::GlassFactory => "glass_factory",
            Self::Farm => "farm",
            Self::Cowshed => "cowshed",
            Self::Piggery => "piggery",
            Self::Sheepfold => "sheepfold",
            Self::Stable => "stable",
            Self::Theater => "theater",
            Self::Temple => "temple",
            Self::Bakehouse => "bakehouse",
            Self::Brewery => "brewery",
            Self::Distillery => "distillery",
            Self::Slaughterhouse => "slaughterhouse",
            Self::IceHouse => "ice_house",
            Self::Market => "market",
            Self::Cedar => "cedar",
            Self::Larch => "larch",
            Self::Oak => "oak",
        }
    }

    pub fn category(&self) -> BuildingCategoryEnum {
        match self {
            Self::Blacksmith
            | Self::BlastFurnace
            | Self::Bloomery
            | Self::CarpenterShop
            | Self::GlassFactory => BuildingCategoryEnum::ManufacturingWorkshops,
            Self::Farm => BuildingCategoryEnum::Agriculture,
            Self::Cowshed | Self::Piggery | Self::Sheepfold | Self::Stable => {
                BuildingCategoryEnum::AnimalBreeding
            }
            Self::Theater => BuildingCategoryEnum::Entertainment,
            Self::Temple => BuildingCategoryEnum::Cult,
            Self::Bakehouse
            | Self::Brewery
            | Self::Distillery
            | Self::Slaughterhouse
            | Self::IceHouse
            | Self::Market => BuildingCategoryEnum::Commerce,
            Self::Cedar | Self::Larch | Self::Oak => BuildingCategoryEnum::Natural,
        }
    }

    pub fn iter() -> impl Iterator<Item = BuildingTypeEnum> {
        [
            BuildingTypeEnum::Blacksmith,
            BuildingTypeEnum::BlastFurnace,
            BuildingTypeEnum::Bloomery,
            BuildingTypeEnum::CarpenterShop,
            BuildingTypeEnum::GlassFactory,
            BuildingTypeEnum::Farm,
            BuildingTypeEnum::Cowshed,
            BuildingTypeEnum::Piggery,
            BuildingTypeEnum::Sheepfold,
            BuildingTypeEnum::Stable,
            BuildingTypeEnum::Theater,
            BuildingTypeEnum::Temple,
            BuildingTypeEnum::Bakehouse,
            BuildingTypeEnum::Brewery,
            BuildingTypeEnum::Distillery,
            BuildingTypeEnum::Slaughterhouse,
            BuildingTypeEnum::IceHouse,
            BuildingTypeEnum::Market,
            BuildingTypeEnum::Cedar,
            BuildingTypeEnum::Larch,
            BuildingTypeEnum::Oak,
        ]
        .into_iter()
    }

    pub fn to_tree_type(&self) -> Option<TreeTypeEnum> {
        TreeTypeEnum::from_building_type(*self)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum ManufacturingWorkshopTypeEnum {
    Blacksmith = 1,
    BlastFurnace = 2,
    Bloomery = 3,
    CarpenterShop = 4,
    GlassFactory = 5,
}

impl ManufacturingWorkshopTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Blacksmith),
            2 => Some(Self::BlastFurnace),
            3 => Some(Self::Bloomery),
            4 => Some(Self::CarpenterShop),
            5 => Some(Self::GlassFactory),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Blacksmith => "blacksmith",
            Self::BlastFurnace => "blast_furnace",
            Self::Bloomery => "bloomery",
            Self::CarpenterShop => "carpenter_shop",
            Self::GlassFactory => "glass_factory",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Blacksmith => BuildingTypeEnum::Blacksmith,
            Self::BlastFurnace => BuildingTypeEnum::BlastFurnace,
            Self::Bloomery => BuildingTypeEnum::Bloomery,
            Self::CarpenterShop => BuildingTypeEnum::CarpenterShop,
            Self::GlassFactory => BuildingTypeEnum::GlassFactory,
        }
    }

    pub fn iter() -> impl Iterator<Item = ManufacturingWorkshopTypeEnum> {
        [
            ManufacturingWorkshopTypeEnum::Blacksmith,
            ManufacturingWorkshopTypeEnum::BlastFurnace,
            ManufacturingWorkshopTypeEnum::Bloomery,
            ManufacturingWorkshopTypeEnum::CarpenterShop,
            ManufacturingWorkshopTypeEnum::GlassFactory,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum AgricultureTypeEnum {
    Farm = 10,
}

impl AgricultureTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            10 => Some(Self::Farm),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Farm => "farm",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Farm => BuildingTypeEnum::Farm,
        }
    }

    pub fn iter() -> impl Iterator<Item = AgricultureTypeEnum> {
        [AgricultureTypeEnum::Farm].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum AnimalBreedingTypeEnum {
    Cowshed = 20,
    Piggery = 21,
    Sheepfold = 22,
    Stable = 23,
}

impl AnimalBreedingTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            20 => Some(Self::Cowshed),
            21 => Some(Self::Piggery),
            22 => Some(Self::Sheepfold),
            23 => Some(Self::Stable),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Cowshed => "cowshed",
            Self::Piggery => "piggery",
            Self::Sheepfold => "sheepfold",
            Self::Stable => "stable",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Cowshed => BuildingTypeEnum::Cowshed,
            Self::Piggery => BuildingTypeEnum::Piggery,
            Self::Sheepfold => BuildingTypeEnum::Sheepfold,
            Self::Stable => BuildingTypeEnum::Stable,
        }
    }

    pub fn iter() -> impl Iterator<Item = AnimalBreedingTypeEnum> {
        [
            AnimalBreedingTypeEnum::Cowshed,
            AnimalBreedingTypeEnum::Piggery,
            AnimalBreedingTypeEnum::Sheepfold,
            AnimalBreedingTypeEnum::Stable,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum EntertainmentTypeEnum {
    Theater = 30,
}

impl EntertainmentTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            30 => Some(Self::Theater),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Theater => "theater",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Theater => BuildingTypeEnum::Theater,
        }
    }

    pub fn iter() -> impl Iterator<Item = EntertainmentTypeEnum> {
        [EntertainmentTypeEnum::Theater].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum CultTypeEnum {
    Temple = 40,
}

impl CultTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            40 => Some(Self::Temple),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Temple => "temple",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Temple => BuildingTypeEnum::Temple,
        }
    }

    pub fn iter() -> impl Iterator<Item = CultTypeEnum> {
        [CultTypeEnum::Temple].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum CommerceTypeEnum {
    Bakehouse = 50,
    Brewery = 51,
    Distillery = 52,
    Slaughterhouse = 53,
    IceHouse = 54,
    Market = 55,
}

impl CommerceTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            50 => Some(Self::Bakehouse),
            51 => Some(Self::Brewery),
            52 => Some(Self::Distillery),
            53 => Some(Self::Slaughterhouse),
            54 => Some(Self::IceHouse),
            55 => Some(Self::Market),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Bakehouse => "bakehouse",
            Self::Brewery => "brewery",
            Self::Distillery => "distillery",
            Self::Slaughterhouse => "slaughterhouse",
            Self::IceHouse => "ice_house",
            Self::Market => "market",
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Bakehouse => BuildingTypeEnum::Bakehouse,
            Self::Brewery => BuildingTypeEnum::Brewery,
            Self::Distillery => BuildingTypeEnum::Distillery,
            Self::Slaughterhouse => BuildingTypeEnum::Slaughterhouse,
            Self::IceHouse => BuildingTypeEnum::IceHouse,
            Self::Market => BuildingTypeEnum::Market,
        }
    }

    pub fn iter() -> impl Iterator<Item = CommerceTypeEnum> {
        [
            CommerceTypeEnum::Bakehouse,
            CommerceTypeEnum::Brewery,
            CommerceTypeEnum::Distillery,
            CommerceTypeEnum::Slaughterhouse,
            CommerceTypeEnum::IceHouse,
            CommerceTypeEnum::Market,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
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

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::House => "house",
        }
    }

    pub fn iter() -> impl Iterator<Item = DwellingsTypeEnum> {
        [DwellingsTypeEnum::House].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
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

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Road => "road",
            Self::PavedRoad => "paved_road",
            Self::Avenue => "avenue",
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
