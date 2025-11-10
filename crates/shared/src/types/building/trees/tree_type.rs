use crate::BiomeType;
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Encode, Decode)]
#[sqlx(type_name = "tree_type")]
pub enum TreeType {
    Cedar,
    Larch,
    Oak,
}

impl TreeType {
    pub fn from_biome(biome: BiomeType) -> Vec<TreeType> {
        match biome {
            BiomeType::Savanna => vec![],
            BiomeType::Grassland => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::TropicalSeasonalForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::TropicalRainForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::TropicalDeciduousForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::TemperateRainForest => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::Wetland => vec![TreeType::Cedar, TreeType::Larch, TreeType::Oak],
            BiomeType::Taiga => vec![],
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
