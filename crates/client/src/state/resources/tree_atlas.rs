use std::collections::HashMap;

use bevy::prelude::*;
use shared::{TreeType, atlas::TreeAtlas};

pub fn setup_tree_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut atlas = TreeAtlas::default();
    atlas.load();

    for tree_type in TreeType::iter() {
        if !matches!(tree_type, TreeType::Cedar) {
            continue;
        }

        let mut variations = HashMap::new();
        let sprite_variations = atlas
            .get_variations(tree_type)
            .expect(format!("No variation found for tree type {:?}", tree_type).as_str());

        for sprite_variation in sprite_variations {
            let path = format!("sprites/trees/{}.png", sprite_variation);
            variations.insert(sprite_variation.clone(), asset_server.load(path));
        }

        atlas.handles.extend(variations);
    }

    commands.insert_resource(atlas);
    info!("✓ Tree atlas loaded");
}
