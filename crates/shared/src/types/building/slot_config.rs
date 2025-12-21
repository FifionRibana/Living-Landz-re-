use bincode::{Decode, Encode};
use crate::{BiomeTypeEnum, BuildingTypeEnum};

/// Configuration for unit slot positions within a cell
#[derive(Debug, Clone, Encode, Decode)]
pub struct SlotConfiguration {
    pub interior_slots: usize,
    pub exterior_slots: usize,
    pub interior_grid_size: (usize, usize), // (rows, cols)
    pub exterior_grid_size: (usize, usize), // (rows, cols)
}

impl Default for SlotConfiguration {
    fn default() -> Self {
        Self {
            interior_slots: 0,
            exterior_slots: 1,
            interior_grid_size: (0, 0),
            exterior_grid_size: (1, 1),
        }
    }
}

impl SlotConfiguration {
    /// Get slot configuration for a specific building type
    pub fn for_building_type(building_type: BuildingTypeEnum) -> Self {
        match building_type {
            // Manufacturing Workshops
            BuildingTypeEnum::Blacksmith => Self {
                interior_slots: 6,
                exterior_slots: 4,
                interior_grid_size: (2, 3),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::BlastFurnace => Self {
                interior_slots: 8,
                exterior_slots: 6,
                interior_grid_size: (2, 4),
                exterior_grid_size: (2, 3),
            },
            BuildingTypeEnum::Bloomery => Self {
                interior_slots: 4,
                exterior_slots: 4,
                interior_grid_size: (2, 2),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::CarpenterShop => Self {
                interior_slots: 6,
                exterior_slots: 4,
                interior_grid_size: (2, 3),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::GlassFactory => Self {
                interior_slots: 10,
                exterior_slots: 6,
                interior_grid_size: (2, 5),
                exterior_grid_size: (2, 3),
            },

            // Agriculture
            BuildingTypeEnum::Farm => Self {
                interior_slots: 8,
                exterior_slots: 12,
                interior_grid_size: (2, 4),
                exterior_grid_size: (3, 4),
            },

            // Animal Breeding
            BuildingTypeEnum::Cowshed => Self {
                interior_slots: 10,
                exterior_slots: 6,
                interior_grid_size: (2, 5),
                exterior_grid_size: (2, 3),
            },
            BuildingTypeEnum::Piggery => Self {
                interior_slots: 8,
                exterior_slots: 6,
                interior_grid_size: (2, 4),
                exterior_grid_size: (2, 3),
            },
            BuildingTypeEnum::Sheepfold => Self {
                interior_slots: 12,
                exterior_slots: 8,
                interior_grid_size: (3, 4),
                exterior_grid_size: (2, 4),
            },
            BuildingTypeEnum::Stable => Self {
                interior_slots: 10,
                exterior_slots: 6,
                interior_grid_size: (2, 5),
                exterior_grid_size: (2, 3),
            },

            // Entertainment
            BuildingTypeEnum::Theater => Self {
                interior_slots: 20,
                exterior_slots: 8,
                interior_grid_size: (4, 5),
                exterior_grid_size: (2, 4),
            },

            // Cult
            BuildingTypeEnum::Temple => Self {
                interior_slots: 15,
                exterior_slots: 10,
                interior_grid_size: (3, 5),
                exterior_grid_size: (2, 5),
            },

            // Commerce
            BuildingTypeEnum::Bakehouse => Self {
                interior_slots: 6,
                exterior_slots: 4,
                interior_grid_size: (2, 3),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::Brewery => Self {
                interior_slots: 10,
                exterior_slots: 6,
                interior_grid_size: (2, 5),
                exterior_grid_size: (2, 3),
            },
            BuildingTypeEnum::Distillery => Self {
                interior_slots: 8,
                exterior_slots: 4,
                interior_grid_size: (2, 4),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::Slaughterhouse => Self {
                interior_slots: 8,
                exterior_slots: 6,
                interior_grid_size: (2, 4),
                exterior_grid_size: (2, 3),
            },
            BuildingTypeEnum::IceHouse => Self {
                interior_slots: 6,
                exterior_slots: 4,
                interior_grid_size: (2, 3),
                exterior_grid_size: (2, 2),
            },
            BuildingTypeEnum::Market => Self {
                interior_slots: 15,
                exterior_slots: 10,
                interior_grid_size: (3, 5),
                exterior_grid_size: (2, 5),
            },
        }
    }

    /// Get slot configuration for terrain/biome type (no building)
    pub fn for_terrain_type(biome: BiomeTypeEnum) -> Self {
        match biome {
            // Open terrains - easier to move
            BiomeTypeEnum::Grassland | BiomeTypeEnum::Savanna => Self {
                interior_slots: 0,
                exterior_slots: 5,
                interior_grid_size: (0, 0),
                exterior_grid_size: (1, 5),
            },
            // Forests - moderate difficulty
            BiomeTypeEnum::TropicalSeasonalForest
            | BiomeTypeEnum::TropicalRainForest
            | BiomeTypeEnum::TropicalDeciduousForest
            | BiomeTypeEnum::TemperateRainForest
            | BiomeTypeEnum::Taiga => Self {
                interior_slots: 0,
                exterior_slots: 8,
                interior_grid_size: (0, 0),
                exterior_grid_size: (2, 4),
            },
            // Wetlands - difficult
            BiomeTypeEnum::Wetland => Self {
                interior_slots: 0,
                exterior_slots: 10,
                interior_grid_size: (0, 0),
                exterior_grid_size: (2, 5),
            },
            // Mountains/Tundra - very difficult
            BiomeTypeEnum::Tundra => Self {
                interior_slots: 0,
                exterior_slots: 12,
                interior_grid_size: (0, 0),
                exterior_grid_size: (3, 4),
            },
            // Deserts - difficult
            BiomeTypeEnum::Desert | BiomeTypeEnum::ColdDesert => Self {
                interior_slots: 0,
                exterior_slots: 10,
                interior_grid_size: (0, 0),
                exterior_grid_size: (2, 5),
            },
            // Water/Ice - very limited
            BiomeTypeEnum::Ocean
            | BiomeTypeEnum::DeepOcean
            | BiomeTypeEnum::Lake
            | BiomeTypeEnum::Ice => Self {
                interior_slots: 0,
                exterior_slots: 2,
                interior_grid_size: (0, 0),
                exterior_grid_size: (1, 2),
            },
            // Default/Undefined
            BiomeTypeEnum::Undefined => Self::default(),
        }
    }

    /// Get total number of available slots
    pub fn total_slots(&self) -> usize {
        self.interior_slots + self.exterior_slots
    }

    /// Check if configuration has interior slots
    pub fn has_interior(&self) -> bool {
        self.interior_slots > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_slot_config() {
        let blacksmith = SlotConfiguration::for_building_type(BuildingTypeEnum::Blacksmith);
        assert_eq!(blacksmith.interior_slots, 6);
        assert_eq!(blacksmith.exterior_slots, 4);
        assert_eq!(blacksmith.total_slots(), 10);
        assert!(blacksmith.has_interior());
    }

    #[test]
    fn test_terrain_slot_config() {
        let grassland = SlotConfiguration::for_terrain_type(BiomeTypeEnum::Grassland);
        assert_eq!(grassland.interior_slots, 0);
        assert_eq!(grassland.exterior_slots, 5);
        assert!(!grassland.has_interior());
    }
}
