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
            BuildingTypeEnum::Forge => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Center filled hexagon
                exterior_layout: SlotLayout::hex_line_vertical(4), // Ring around at radius 3
            },
            BuildingTypeEnum::Fonderie => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(6, 1),  // Larger ring at radius 4
            },
            BuildingTypeEnum::AtelierCharpentier => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Center filled hexagon
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Verrerie => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Larger center area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },

            // Agriculture
            BuildingTypeEnum::Ferme => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Large exterior fields
            },

            // Animal Breeding
            BuildingTypeEnum::Etable => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(19, 4), // Outdoor pasture
            },
            BuildingTypeEnum::Porcherie => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Outdoor area
            },
            BuildingTypeEnum::Bergerie => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Large grazing area
            },
            BuildingTypeEnum::Ecurie => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_range(19, 4), // Exercise yard
            },

            // Entertainment - Layout en amphithéâtre (centered)
            BuildingTypeEnum::Theatre => Self {
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
            BuildingTypeEnum::LieuDeCulte => Self {
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
            BuildingTypeEnum::Cuisine => Self {
                interior_layout: SlotLayout::hex_range(7, 1), // Baking area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Brasserie => Self {
                interior_layout: SlotLayout::hex_range(19, 2), // Brewing vats
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Abattoir => Self {
                interior_layout: SlotLayout::hex_range(19, 2), // Processing area
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::Glaciere => Self {
                interior_layout: SlotLayout::hex_ring(6, 1), // Cold storage
                exterior_layout: SlotLayout::hex_line_vertical(4),
            },
            BuildingTypeEnum::PlaceMarche => Self {
                interior_layout: SlotLayout::hex_range(37, 3), // Large market stalls
                exterior_layout: SlotLayout::hex_range(0, 0),  // No exterior
            },

            // Natural - Trees
            // Trees: small center (trunk/canopy) with surrounding area
            BuildingTypeEnum::Cedar | BuildingTypeEnum::Larch | BuildingTypeEnum::Oak => Self {
                interior_layout: SlotLayout::hex_range(0, 0), // No interior
                exterior_layout: SlotLayout::hex_ring(19, 2), // Around the tree
            },

            // Default for all other building types
            _ => Self::default(),
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
        let forge = SlotConfiguration::for_building_type(BuildingTypeEnum::Forge);
        assert_eq!(forge.interior_slots(), 7); // hex_range(7, 1)
        assert_eq!(forge.exterior_slots(), 4); // hex_line_vertical(4)
        assert_eq!(forge.total_slots(), 11);
        assert!(forge.has_interior());
    }

    #[test]
    fn test_terrain_slot_config() {
        let grassland = SlotConfiguration::for_terrain_type(BiomeTypeEnum::Grassland);
        assert_eq!(grassland.interior_slots(), 0); // hex_range(0, 0)
        assert_eq!(grassland.exterior_slots(), 19); // hex_range(19, 2)
        assert!(!grassland.has_interior());
    }

    #[test]
    fn test_theatre_custom_layout() {
        let theatre = SlotConfiguration::for_building_type(BuildingTypeEnum::Theatre);
        // Theatre devrait avoir 20 slots intérieurs (amphithéâtre)
        assert_eq!(theatre.interior_slots(), 20);
        assert_eq!(theatre.exterior_slots(), 4); // hex_line_vertical(4)
        assert!(theatre.has_interior());

        // Vérifier que c'est un layout custom
        use crate::SlotLayoutType;
        match theatre.interior_layout.layout_type {
            SlotLayoutType::Custom { ref positions } => {
                assert_eq!(
                    positions.len(),
                    20,
                    "Theatre should have 20 custom positions"
                );
            }
            _ => panic!("Theatre should use Custom layout type"),
        }
    }

    #[test]
    fn test_lieu_de_culte_custom_layout() {
        let lieu_de_culte = SlotConfiguration::for_building_type(BuildingTypeEnum::LieuDeCulte);
        // LieuDeCulte devrait avoir 15 slots intérieurs (en croix)
        assert_eq!(lieu_de_culte.interior_slots(), 15);
        assert_eq!(lieu_de_culte.exterior_slots(), 4); // hex_line_vertical(4)
        assert!(lieu_de_culte.has_interior());

        // Vérifier que c'est un layout custom
        use crate::SlotLayoutType;
        match lieu_de_culte.interior_layout.layout_type {
            SlotLayoutType::Custom { ref positions } => {
                assert_eq!(
                    positions.len(),
                    15,
                    "LieuDeCulte should have 15 custom positions"
                );
            }
            _ => panic!("LieuDeCulte should use Custom layout type"),
        }
    }
}
