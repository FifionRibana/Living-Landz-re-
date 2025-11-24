use std::collections::HashMap;

use bevy::prelude::*;
use shared::{BuildingTypeEnum, atlas::BuildingAtlas};

pub fn setup_building_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut atlas = BuildingAtlas::default();
    atlas.load();

    info!("Loading building atlas with {} building types", BuildingTypeEnum::iter().count());

    for building_type in BuildingTypeEnum::iter() {
        let sprite_variations = atlas
            .get_variations(building_type)
            .expect(format!("No variation found for building type {:?}", building_type).as_str())
            .to_vec();

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
