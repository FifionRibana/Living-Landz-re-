use super::slot_layout::SlotLayout;
use crate::{BiomeTypeEnum, BuildingTypeEnum};
use bevy::prelude::Vec2;
use bincode::{Decode, Encode};

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
                interior_layout: SlotLayout::hex_range(7, 1), // Center filled hexagon
                exterior_layout: SlotLayout::hex_line_vertical(4), // Ring around at radius 3
            },
            BuildingTypeEnum::BlastFurnace => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(6, 1),  // Larger ring at radius 4
            },
            BuildingTypeEnum::Bloomery => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(6, 3),  // Ring around at radius 3
            },
            BuildingTypeEnum::CarpenterShop => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Center filled hexagon
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::GlassFactory => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Larger center area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },

            // Agriculture
            BuildingTypeEnum::Farm => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Large exterior fields
            },

            // Animal Breeding
            BuildingTypeEnum::Cowshed => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(19, 4), // Outdoor pasture
            },
            BuildingTypeEnum::Piggery => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Outdoor area
            },
            BuildingTypeEnum::Sheepfold => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Large grazing area
            },
            BuildingTypeEnum::Stable => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Exercise yard
            },

            // Entertainment - Layout en amphithéâtre (centered)
            BuildingTypeEnum::Theater => Self {
                interior_layout: SlotLayout::custom(vec![
                    // Scène centrale (4 slots)
                    Vec2::new(-64.0, 80.0),
                    Vec2::new(-21.0, 80.0),
                    Vec2::new(21.0, 80.0),
                    Vec2::new(64.0, 80.0),
                    // Rangée 2 (5 slots)
                    Vec2::new(-80.0, 30.0),
                    Vec2::new(-40.0, 30.0),
                    Vec2::new(0.0, 30.0),
                    Vec2::new(40.0, 30.0),
                    Vec2::new(80.0, 30.0),
                    // Rangée 3 (6 slots)
                    Vec2::new(-100.0, -20.0),
                    Vec2::new(-60.0, -20.0),
                    Vec2::new(-20.0, -20.0),
                    Vec2::new(20.0, -20.0),
                    Vec2::new(60.0, -20.0),
                    Vec2::new(100.0, -20.0),
                    // Fond (5 slots)
                    Vec2::new(-80.0, -70.0),
                    Vec2::new(-40.0, -70.0),
                    Vec2::new(0.0, -70.0),
                    Vec2::new(40.0, -70.0),
                    Vec2::new(80.0, -70.0),
                ]),
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },

            // Cult - Layout en croix (nef + transepts, compact and centered)
            BuildingTypeEnum::Temple => Self {
                interior_layout: SlotLayout::custom(vec![
                    // Nef centrale (5 slots verticaux)
                    Vec2::new(0.0, 80.0),
                    Vec2::new(0.0, 40.0),
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, -40.0),
                    Vec2::new(0.0, -80.0),
                    // Transept gauche (5 slots)
                    Vec2::new(-80.0, 0.0),
                    Vec2::new(-60.0, 20.0),
                    Vec2::new(-40.0, 0.0),
                    Vec2::new(-60.0, -20.0),
                    Vec2::new(-40.0, -20.0),
                    // Transept droit (5 slots)
                    Vec2::new(80.0, 0.0),
                    Vec2::new(60.0, 20.0),
                    Vec2::new(40.0, 0.0),
                    Vec2::new(60.0, -20.0),
                    Vec2::new(40.0, -20.0),
                ]),
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },

            // Commerce
            BuildingTypeEnum::Bakehouse => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Baking area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Brewery => Self {
                interior_layout: SlotLayout::hex_range(19, 2), // Brewing vats
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Distillery => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Distilling equipment
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Slaughterhouse => Self {
                interior_layout: SlotLayout::hex_range(19, 2), // Processing area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::IceHouse => Self {
                interior_layout: SlotLayout::hex_ring(6, 1), // Cold storage
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Market => Self {
                interior_layout: SlotLayout::hex_range(37, 3), // Large market stalls
                exterior_layout: SlotLayout::hex_range(0, 0),  // No exterior
                                                               // exterior_layout: SlotLayout::hex_ring(18, 5),  // Outdoor vendors
            },

            // Natural - Trees
            // Trees: small center (trunk/canopy) with surrounding area
            BuildingTypeEnum::Cedar | BuildingTypeEnum::Larch | BuildingTypeEnum::Oak => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(19, 2), // Around the tree
            },
        }
    }

    /// Get slot configuration for terrain/biome type (no building)
    pub fn for_terrain_type(biome: BiomeTypeEnum) -> Self {
        match biome {
            // Open terrains - easier to move, no interior (open field)
            BiomeTypeEnum::Grassland | BiomeTypeEnum::Savanna => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 2), // Open area
            },
            // Forests - moderate difficulty, no interior
            BiomeTypeEnum::TropicalSeasonalForest
            | BiomeTypeEnum::TropicalRainForest
            | BiomeTypeEnum::TropicalDeciduousForest
            | BiomeTypeEnum::TemperateRainForest
            | BiomeTypeEnum::Taiga => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 2), // Forest clearing
            },
            // Wetlands - difficult, no interior
            BiomeTypeEnum::Wetland => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 2), // Marshy area
            },
            // Mountains/Tundra - very difficult, no interior
            BiomeTypeEnum::Tundra => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 2), // Cold terrain
            },
            // Deserts - difficult, no interior
            BiomeTypeEnum::Desert | BiomeTypeEnum::ColdDesert => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 2), // Desert area
            },
            // Water/Ice - very limited, no interior
            BiomeTypeEnum::Ocean
            | BiomeTypeEnum::DeepOcean
            | BiomeTypeEnum::Lake
            | BiomeTypeEnum::Ice => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(7, 1), // Very limited water access
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
        assert_eq!(blacksmith.interior_slots(), 7); // hex_range(7, 1)
        assert_eq!(blacksmith.exterior_slots(), 6); // hex_ring(6, 3)
        assert_eq!(blacksmith.total_slots(), 13);
        assert!(blacksmith.has_interior());
    }

    #[test]
    fn test_terrain_slot_config() {
        let grassland = SlotConfiguration::for_terrain_type(BiomeTypeEnum::Grassland);
        assert_eq!(grassland.interior_slots(), 0); // hex_range(0, 0)
        assert_eq!(grassland.exterior_slots(), 19); // hex_range(19, 2)
        assert!(!grassland.has_interior());
    }

    #[test]
    fn test_theater_custom_layout() {
        let theater = SlotConfiguration::for_building_type(BuildingTypeEnum::Theater);
        // Theater devrait avoir 20 slots intérieurs (amphithéâtre)
        assert_eq!(theater.interior_slots(), 20);
        assert_eq!(theater.exterior_slots(), 12); // hex_ring(12, 5)
        assert!(theater.has_interior());

        // Vérifier que c'est un layout custom
        use crate::SlotLayoutType;
        match theater.interior_layout.layout_type {
            SlotLayoutType::Custom { ref positions } => {
                assert_eq!(
                    positions.len(),
                    20,
                    "Theater should have 20 custom positions"
                );
            }
            _ => panic!("Theater should use Custom layout type"),
        }
    }

    #[test]
    fn test_temple_custom_layout() {
        let temple = SlotConfiguration::for_building_type(BuildingTypeEnum::Temple);
        // Temple devrait avoir 15 slots intérieurs (en croix)
        assert_eq!(temple.interior_slots(), 15);
        assert_eq!(temple.exterior_slots(), 18); // hex_ring(18, 5)
        assert!(temple.has_interior());

        // Vérifier que c'est un layout custom
        use crate::SlotLayoutType;
        match temple.interior_layout.layout_type {
            SlotLayoutType::Custom { ref positions } => {
                assert_eq!(
                    positions.len(),
                    15,
                    "Temple should have 15 custom positions"
                );
            }
            _ => panic!("Temple should use Custom layout type"),
        }
    }
}
