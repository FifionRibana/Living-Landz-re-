use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode, sqlx::Type, PartialEq, Eq, Hash,
)]
#[sqlx(type_name = "biome_type")]
pub enum BiomeType {
    Undefined,
    Ocean,
    DeepOcean,
    Desert,
    Savanna,
    Grassland,
    TropicalSeasonalForest,
    TropicalRainForest,
    TropicalDeciduousForest,
    TemperateRainForest,
    Wetland,
    Taiga,
    Tundra,
    Lake,
    ColdDesert,
    Ice,
}

impl Default for BiomeType {
    fn default() -> Self {
        BiomeType::DeepOcean
    }
}

impl BiomeType {
    pub fn iter() -> impl Iterator<Item = BiomeType> {
        [
            BiomeType::Ocean,
            BiomeType::DeepOcean,
            BiomeType::Desert,
            BiomeType::Savanna,
            BiomeType::Grassland,
            BiomeType::TropicalSeasonalForest,
            BiomeType::TropicalRainForest,
            BiomeType::TropicalDeciduousForest,
            BiomeType::TemperateRainForest,
            BiomeType::Wetland,
            BiomeType::Taiga,
            BiomeType::Tundra,
            BiomeType::Lake,
            BiomeType::ColdDesert,
            BiomeType::Ice,
        ]
        .into_iter()
    }
}
