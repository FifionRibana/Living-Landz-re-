use bincode::{Decode, Encode};

use crate::RoadCategory;

/// Type de route (catégorie + variante)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RoadType {
    pub id: i32,
    pub category: RoadCategory,
    pub variant: String,
}

impl Default for RoadType {
    fn default() -> Self {
        Self {
            id: 1,
            category: RoadCategory::DirtPath,
            variant: "basic".to_string(),
        }
    }
}

impl RoadType {
    pub fn dirt_path(id: i32) -> Self {
        Self {
            id,
            category: RoadCategory::DirtPath,
            variant: "basic".to_string(),
        }
    }

    pub fn paved_road(id: i32) -> Self {
        Self {
            id,
            category: RoadCategory::PavedRoad,
            variant: "stone".to_string(),
        }
    }

    pub fn highway(id: i32) -> Self {
        Self {
            id,
            category: RoadCategory::Highway,
            variant: "cobblestone".to_string(),
        }
    }
}
