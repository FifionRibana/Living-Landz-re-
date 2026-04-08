use bevy::prelude::*;
use shared::{BuildingTypeEnum, atlas::BuildingAtlas};

pub fn setup_building_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut atlas = BuildingAtlas::default();
    atlas.load();

    // Filter out trees - they use TreeAtlas, not BuildingAtlas
    let building_types: Vec<_> = BuildingTypeEnum::iter()
        .filter(|bt| bt.to_specific_type() != shared::BuildingSpecificTypeEnum::Tree)
        .collect();

    info!("Loading building atlas with {} building types", building_types.len());

    for building_type in building_types {
        let Some(sprite_variations) = atlas.get_variations(building_type) else {
            continue; // No sprite defined yet for this building type
        };
        let sprite_variations = sprite_variations.to_vec();

        info!("Loading building type {:?} with {} variations", building_type, sprite_variations.len());

        for sprite_variation in sprite_variations {
            let path = format!("sprites/buildings/{}.png", sprite_variation);
            info!("Loading sprite: {}", path);
            let handle = asset_server.load(path);
            atlas.handles.insert(sprite_variation, handle);
        }
    }

    info!("✓ Building atlas loaded with {} handles", atlas.handles.len());
    commands.insert_resource(atlas);
}
