use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, sqlx::Type)]
#[sqlx(type_name = "building_category")]
pub enum BuildingCategory {
    Unknown,
    Natural,
    Structure,
    Infrastructure,
    Defense,
}

impl BuildingCategory {
    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn from_str(name: &str) -> Self {
        match name {
            "natural" => BuildingCategory::Natural,
            "structure" => BuildingCategory::Structure,
            "infrastructure" => BuildingCategory::Infrastructure,
            "defense" => BuildingCategory::Defense,
            _ => BuildingCategory::Unknown,
        }
    }
}
