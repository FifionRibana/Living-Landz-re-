use bincode::{Decode, Encode};

// ============ ENUMS RUST ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum BiomeTypeEnum {
    Undefined = 0,
    Ocean = 1,
    DeepOcean = 2,
    Desert = 3,
    Savanna = 4,
    Grassland = 5,
    TropicalSeasonalForest = 6,
    TropicalRainForest = 7,
    TropicalDeciduousForest = 8,
    TemperateRainForest = 9,
    Wetland = 10,
    Taiga = 11,
    Tundra = 12,
    Lake = 13,
    ColdDesert = 14,
    Ice = 15,
}

impl Default for BiomeTypeEnum {
    fn default() -> Self {
        BiomeTypeEnum::DeepOcean
    }
}

impl BiomeTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Undefined),
            1 => Some(Self::Ocean),
            2 => Some(Self::DeepOcean),
            3 => Some(Self::Desert),
            4 => Some(Self::Savanna),
            5 => Some(Self::Grassland),
            6 => Some(Self::TropicalSeasonalForest),
            7 => Some(Self::TropicalRainForest),
            8 => Some(Self::TropicalDeciduousForest),
            9 => Some(Self::TemperateRainForest),
            10 => Some(Self::Wetland),
            11 => Some(Self::Taiga),
            12 => Some(Self::Tundra),
            13 => Some(Self::Lake),
            14 => Some(Self::ColdDesert),
            15 => Some(Self::Ice),
            _ => None,
        }
    }
    
    pub fn iter() -> impl Iterator<Item = BiomeTypeEnum> {
        [
            BiomeTypeEnum::Ocean,
            BiomeTypeEnum::DeepOcean,
            BiomeTypeEnum::Desert,
            BiomeTypeEnum::Savanna,
            BiomeTypeEnum::Grassland,
            BiomeTypeEnum::TropicalSeasonalForest,
            BiomeTypeEnum::TropicalRainForest,
            BiomeTypeEnum::TropicalDeciduousForest,
            BiomeTypeEnum::TemperateRainForest,
            BiomeTypeEnum::Wetland,
            BiomeTypeEnum::Taiga,
            BiomeTypeEnum::Tundra,
            BiomeTypeEnum::Lake,
            BiomeTypeEnum::ColdDesert,
            BiomeTypeEnum::Ice,
        ]
        .into_iter()
    }
}