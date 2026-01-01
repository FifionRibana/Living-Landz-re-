use bevy::prelude::*;
use shared::{BiomeTypeEnum, BuildingCategoryEnum, BuildingData};

/// Get the terrain background image path based on biome
pub fn get_terrain_background_path(biome: BiomeTypeEnum) -> String {
    format!("ui/backgrounds/terrain_{:?}.jpg", biome)
}

/// Get the building background image path
pub fn get_building_background_path(building: &BuildingData) -> String {
    if let Some(building_type) = &building.to_building_type() {
        format!("ui/backgrounds/building_{}.jpg", building_type.to_name_lowercase())
    } else {
        "ui/backgrounds/building_generic.png".to_string()
    }
}

/// Get the separator image path based on building category
/// Returns (left_separator_path, right_separator_path)
pub fn get_separator_paths(building: Option<&BuildingData>) -> (String, String) {
    let separator_name = if let Some(building) = building {
        if let Some(building_type) = building.to_building_type() {
            match building_type.category() {
                // Wide separators for large buildings
                BuildingCategoryEnum::Entertainment
                | BuildingCategoryEnum::Cult
                | BuildingCategoryEnum::AnimalBreeding => "ui_vertical_separator_stone_wide",
                // Thin separators for smaller buildings
                BuildingCategoryEnum::ManufacturingWorkshops
                | BuildingCategoryEnum::Agriculture
                | BuildingCategoryEnum::Commerce
                | BuildingCategoryEnum::Natural => "ui_vertical_separator_stone_thin",
                // Default to thin for unknown categories
                _ => "ui_vertical_separator_stone_thin",
            }
        } else {
            "ui_vertical_separator_stone_thin"
        }
    } else {
        // No building, use thin separators
        "ui_vertical_separator_stone_thin"
    };

    let path = format!("ui/{}.png", separator_name);
    (path.clone(), path)
}

/// Load the terrain background image handle
pub fn load_terrain_background(
    asset_server: &AssetServer,
    biome: BiomeTypeEnum,
) -> Handle<Image> {
    let path = get_terrain_background_path(biome);
    asset_server.load(&path)
}

/// Load the building background image handle
pub fn load_building_background(
    asset_server: &AssetServer,
    building: &BuildingData,
) -> Handle<Image> {
    let path = get_building_background_path(building);
    asset_server.load(&path)
}

/// Load separator images
pub fn load_separators(
    asset_server: &AssetServer,
    building: Option<&BuildingData>,
) -> (Handle<Image>, Handle<Image>) {
    let (left_path, right_path) = get_separator_paths(building);
    (asset_server.load(&left_path), asset_server.load(&right_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_background_path() {
        let path = get_terrain_background_path(BiomeTypeEnum::Grassland);
        assert_eq!(path, "ui/backgrounds/terrain_Grassland.jpg");
    }

    #[test]
    fn test_separator_paths_no_building() {
        let (left, right) = get_separator_paths(None);
        assert_eq!(left, "ui/ui_vertical_separator_stone_thin.png");
        assert_eq!(right, "ui/ui_vertical_separator_stone_thin.png");
    }
}
