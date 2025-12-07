use bincode::{Decode, Encode};

/// Catégories de routes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RoadCategory {
    /// Chemin de terre (par défaut)
    DirtPath,
    /// Route pavée (future)
    PavedRoad,
    /// Grande route (future)
    Highway,
}

impl Default for RoadCategory {
    fn default() -> Self {
        RoadCategory::DirtPath
    }
}

impl RoadCategory {
    pub fn to_string(&self) -> String {
        match self {
            RoadCategory::DirtPath => "Chemin de terre".to_string(),
            RoadCategory::PavedRoad => "Route pavée".to_string(),
            RoadCategory::Highway => "Grande route".to_string(),
        }
    }
}
