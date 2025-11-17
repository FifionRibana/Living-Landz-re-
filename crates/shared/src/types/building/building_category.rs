use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, sqlx::Type)]
#[sqlx(type_name = "building_category")]
pub enum BuildingCategory {
    Unknown,
    Natural,
    Urbanism,
    Cult,
    Dwellings,
    ManufacturingWorkshops,
    Entertainment,
    Agriculture,
    AnimalBreeding,
    Education,
    Military
}

impl BuildingCategory {
    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn from_str(name: &str) -> Self {
        match name {
            "natural" => BuildingCategory::Natural,
            "urbanism" => BuildingCategory::Urbanism,
            "cult" => BuildingCategory::Cult,
            "dwellings" => BuildingCategory::Dwellings,
            "manufacturing_workshops" => BuildingCategory::ManufacturingWorkshops,
            "entertainment" => BuildingCategory::Entertainment,
            "agriculture" => BuildingCategory::Agriculture,
            "animal_breeding" => BuildingCategory::AnimalBreeding,
            "education" => BuildingCategory::Education,
            "military" => BuildingCategory::Military,
            _ => BuildingCategory::Unknown,
        }
    }
}
