use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, sqlx::Type)]
#[sqlx(type_name = "resource_category")]
pub enum ResourceCategory {
    Unknown,
    Wood,
    Metal,
    CrudeMaterial,
    Food,
    Furniture,
    Weaponry,
    Jewelry,
    Meat,
    Fruits,
    Vegetables,
}

impl ResourceCategory {
    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn from_str(name: &str) -> Self {
        match name {
            "wood" => ResourceCategory::Wood,
            "metal" => ResourceCategory::Metal,
            "crude_material" => ResourceCategory::CrudeMaterial,
            "food" => ResourceCategory::Food,
            "furniture" => ResourceCategory::Furniture,
            "weaponry" => ResourceCategory::Weaponry,
            "jewelry" => ResourceCategory::Jewelry,
            "meat" => ResourceCategory::Meat,
            "fruits" => ResourceCategory::Fruits,
            "vegetables" => ResourceCategory::Vegetables,
            _ => ResourceCategory::Unknown,
        }
    }
}
