use bevy::prelude::*;
use shared::{BiomeTypeEnum, BuildingData, BuildingTypeEnum};

/// Get the background image path for a cell
/// Currently returns placeholder paths that will need to be created
pub fn get_background_image_path(
    building: Option<&BuildingData>,
    biome: BiomeTypeEnum,
) -> String {
    if let Some(building) = building {
        // Building-specific backgrounds
        let building_type = &building.base_data.specific_type;
        format!("ui/backgrounds/building_{}.png", building_type.to_name_lowercase())
    } else {
        // Terrain/biome backgrounds
        format!("ui/backgrounds/terrain_{:?}.jpg", biome)
    }
}

/// Load the background image handle
pub fn load_background_image(
    asset_server: &AssetServer,
    building: Option<&BuildingData>,
    biome: BiomeTypeEnum,
) -> Handle<Image> {
    let path = get_background_image_path(building, biome);

    // For now, try to load the specific background, fall back to generic
    // In production, we'd check if the file exists first
    asset_server.load(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_paths() {
        // Test terrain background
        let path = get_background_image_path(None, BiomeTypeEnum::Grassland);
        assert_eq!(path, "ui/backgrounds/terrain_Grassland.png");

        // Note: Building tests would require creating a BuildingData instance
        // which is more complex, so we skip those for now
    }
}
