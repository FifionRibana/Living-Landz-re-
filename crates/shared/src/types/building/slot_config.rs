use bincode::{Decode, Encode};
use bevy::prelude::Vec2;
use crate::{BiomeTypeEnum, BuildingTypeEnum};
use super::slot_layout::SlotLayout;

/// Configuration for unit slot positions within a cell
#[derive(Debug, Clone, Encode, Decode)]
pub struct SlotConfiguration {
    pub interior_layout: SlotLayout,
    pub exterior_layout: SlotLayout,
}

impl Default for SlotConfiguration {
    fn default() -> Self {
        Self {
            interior_layout: SlotLayout::hex_grid(0, 0, 0),
            exterior_layout: SlotLayout::hex_grid(1, 1, 1),
        }
    }
}

impl SlotConfiguration {
    /// Get slot configuration for a specific building type
    pub fn for_building_type(building_type: BuildingTypeEnum) -> Self {
        match building_type {
            // Manufacturing Workshops
            BuildingTypeEnum::Blacksmith => Self {
                interior_layout: SlotLayout::hex_grid(6, 3, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::BlastFurnace => Self {
                interior_layout: SlotLayout::hex_grid(8, 4, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },
            BuildingTypeEnum::Bloomery => Self {
                interior_layout: SlotLayout::hex_grid(4, 2, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::CarpenterShop => Self {
                interior_layout: SlotLayout::hex_grid(6, 3, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::GlassFactory => Self {
                interior_layout: SlotLayout::hex_grid(10, 5, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },

            // Agriculture
            BuildingTypeEnum::Farm => Self {
                interior_layout: SlotLayout::hex_grid(8, 4, 2),
                exterior_layout: SlotLayout::hex_ring(12, 2),
            },

            // Animal Breeding
            BuildingTypeEnum::Cowshed => Self {
                interior_layout: SlotLayout::hex_grid(10, 5, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },
            BuildingTypeEnum::Piggery => Self {
                interior_layout: SlotLayout::hex_grid(8, 4, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },
            BuildingTypeEnum::Sheepfold => Self {
                interior_layout: SlotLayout::hex_grid(12, 4, 3),
                exterior_layout: SlotLayout::hex_ring(8, 2),
            },
            BuildingTypeEnum::Stable => Self {
                interior_layout: SlotLayout::hex_grid(10, 5, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },

            // Entertainment - Layout en amphithéâtre
            BuildingTypeEnum::Theater => Self {
                interior_layout: SlotLayout::custom(vec![
                    // Scène (4 slots)
                    Vec2::new(-96.0, 150.0), Vec2::new(-32.0, 150.0),
                    Vec2::new(32.0, 150.0), Vec2::new(96.0, 150.0),
                    // Rangée 2 (5 slots)
                    Vec2::new(-120.0, 80.0), Vec2::new(-60.0, 80.0), Vec2::new(0.0, 80.0),
                    Vec2::new(60.0, 80.0), Vec2::new(120.0, 80.0),
                    // Rangée 3 (6 slots)
                    Vec2::new(-150.0, 10.0), Vec2::new(-90.0, 10.0), Vec2::new(-30.0, 10.0),
                    Vec2::new(30.0, 10.0), Vec2::new(90.0, 10.0), Vec2::new(150.0, 10.0),
                    // Fond (5 slots)
                    Vec2::new(-120.0, -60.0), Vec2::new(-60.0, -60.0), Vec2::new(0.0, -60.0),
                    Vec2::new(60.0, -60.0), Vec2::new(120.0, -60.0),
                ]),
                exterior_layout: SlotLayout::hex_ring(8, 2),
            },

            // Cult - Layout en croix (nef + transepts)
            BuildingTypeEnum::Temple => Self {
                interior_layout: SlotLayout::custom(vec![
                    // Nef centrale (5 slots verticaux)
                    Vec2::new(0.0, 120.0), Vec2::new(0.0, 60.0), Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, -60.0), Vec2::new(0.0, -120.0),
                    // Transept gauche (5 slots)
                    Vec2::new(-120.0, 0.0), Vec2::new(-90.0, 30.0), Vec2::new(-60.0, 0.0),
                    Vec2::new(-90.0, -30.0), Vec2::new(-60.0, -30.0),
                    // Transept droit (5 slots)
                    Vec2::new(120.0, 0.0), Vec2::new(90.0, 30.0), Vec2::new(60.0, 0.0),
                    Vec2::new(90.0, -30.0), Vec2::new(60.0, -30.0),
                ]),
                exterior_layout: SlotLayout::hex_ring(10, 2),
            },

            // Commerce
            BuildingTypeEnum::Bakehouse => Self {
                interior_layout: SlotLayout::hex_grid(6, 3, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::Brewery => Self {
                interior_layout: SlotLayout::hex_grid(10, 5, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },
            BuildingTypeEnum::Distillery => Self {
                interior_layout: SlotLayout::hex_grid(8, 4, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::Slaughterhouse => Self {
                interior_layout: SlotLayout::hex_grid(8, 4, 2),
                exterior_layout: SlotLayout::hex_ring(6, 1),
            },
            BuildingTypeEnum::IceHouse => Self {
                interior_layout: SlotLayout::hex_grid(6, 3, 2),
                exterior_layout: SlotLayout::hex_ring(4, 1),
            },
            BuildingTypeEnum::Market => Self {
                interior_layout: SlotLayout::hex_grid(15, 5, 3),
                exterior_layout: SlotLayout::hex_ring(10, 2),
            },

            // Natural - Trees
            // Trees don't have interior/exterior distinction, just use a simple ring layout
            BuildingTypeEnum::Cedar | BuildingTypeEnum::Larch | BuildingTypeEnum::Oak => Self {
                interior_layout: SlotLayout::hex_ring(4, 1),
                exterior_layout: SlotLayout::hex_ring(8, 2),
            },
        }
    }

    /// Get slot configuration for terrain/biome type (no building)
    pub fn for_terrain_type(biome: BiomeTypeEnum) -> Self {
        match biome {
            // Open terrains - easier to move
            BiomeTypeEnum::Grassland | BiomeTypeEnum::Savanna => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(5, 5, 1),
            },
            // Forests - moderate difficulty
            BiomeTypeEnum::TropicalSeasonalForest
            | BiomeTypeEnum::TropicalRainForest
            | BiomeTypeEnum::TropicalDeciduousForest
            | BiomeTypeEnum::TemperateRainForest
            | BiomeTypeEnum::Taiga => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(8, 4, 2),
            },
            // Wetlands - difficult
            BiomeTypeEnum::Wetland => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(10, 5, 2),
            },
            // Mountains/Tundra - very difficult
            BiomeTypeEnum::Tundra => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(12, 4, 3),
            },
            // Deserts - difficult
            BiomeTypeEnum::Desert | BiomeTypeEnum::ColdDesert => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(10, 5, 2),
            },
            // Water/Ice - very limited
            BiomeTypeEnum::Ocean
            | BiomeTypeEnum::DeepOcean
            | BiomeTypeEnum::Lake
            | BiomeTypeEnum::Ice => Self {
                interior_layout: SlotLayout::hex_grid(0, 0, 0),
                exterior_layout: SlotLayout::hex_grid(2, 2, 1),
            },
            // Default/Undefined
            BiomeTypeEnum::Undefined => Self::default(),
        }
    }

    /// Get total number of available slots
    pub fn total_slots(&self) -> usize {
        self.interior_layout.count + self.exterior_layout.count
    }

    /// Check if configuration has interior slots
    pub fn has_interior(&self) -> bool {
        self.interior_layout.count > 0
    }

    /// Get interior slot count
    pub fn interior_slots(&self) -> usize {
        self.interior_layout.count
    }

    /// Get exterior slot count
    pub fn exterior_slots(&self) -> usize {
        self.exterior_layout.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_slot_config() {
        let blacksmith = SlotConfiguration::for_building_type(BuildingTypeEnum::Blacksmith);
        assert_eq!(blacksmith.interior_slots(), 6);
        assert_eq!(blacksmith.exterior_slots(), 4);
        assert_eq!(blacksmith.total_slots(), 10);
        assert!(blacksmith.has_interior());
    }

    #[test]
    fn test_terrain_slot_config() {
        let grassland = SlotConfiguration::for_terrain_type(BiomeTypeEnum::Grassland);
        assert_eq!(grassland.interior_slots(), 0);
        assert_eq!(grassland.exterior_slots(), 5);
        assert!(!grassland.has_interior());
    }

    #[test]
    fn test_theater_custom_layout() {
        let theater = SlotConfiguration::for_building_type(BuildingTypeEnum::Theater);
        // Theater devrait avoir 20 slots intérieurs (amphithéâtre)
        assert_eq!(theater.interior_slots(), 20);
        assert_eq!(theater.exterior_slots(), 8);
        assert!(theater.has_interior());

        // Vérifier que c'est un layout custom
        match theater.interior_layout.layout_type {
            super::slot_layout::SlotLayoutType::Custom { ref positions } => {
                assert_eq!(positions.len(), 20, "Theater should have 20 custom positions");
            }
            _ => panic!("Theater should use Custom layout type"),
        }
    }

    #[test]
    fn test_temple_custom_layout() {
        let temple = SlotConfiguration::for_building_type(BuildingTypeEnum::Temple);
        // Temple devrait avoir 15 slots intérieurs (en croix)
        assert_eq!(temple.interior_slots(), 15);
        assert_eq!(temple.exterior_slots(), 10);
        assert!(temple.has_interior());

        // Vérifier que c'est un layout custom
        match temple.interior_layout.layout_type {
            super::slot_layout::SlotLayoutType::Custom { ref positions } => {
                assert_eq!(positions.len(), 15, "Temple should have 15 custom positions");
            }
            _ => panic!("Temple should use Custom layout type"),
        }
    }
}
