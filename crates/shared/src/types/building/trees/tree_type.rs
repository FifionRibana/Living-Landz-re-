use crate::{BiomeTypeEnum};
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Encode, Decode)]
#[sqlx(type_name = "tree_type")]
pub enum TreeType {
    Cedar,
    Larch,
    Oak,
}

impl TreeType {
    pub fn from_biome(biome: BiomeTypeEnum) -> Vec<TreeType> {
        match biome {
            BiomeTypeEnum::Savanna => vec![],
            BiomeTypeEnum::Grassland => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::TropicalSeasonalForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::TropicalRainForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::TropicalDeciduousForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::TemperateRainForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::Wetland => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeTypeEnum::Taiga => vec![],
            _ => vec![]
        }
    }

    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn iter() -> impl Iterator<Item = TreeType> {
        [
            TreeType::Cedar,
            TreeType::Larch,
            TreeType::Oak,
        ]
        .into_iter()
    }
}
