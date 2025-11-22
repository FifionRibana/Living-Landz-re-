use bevy::prelude::*;
use shared::atlas::MoonAtlas;

pub fn setup_moon_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut atlas = MoonAtlas::default();
    atlas.load();

    for (index, sprite) in &atlas.sprites {
        let path = format!("sprites/{}.png", sprite);
        atlas.handles.insert(index.clone(), asset_server.load(path));
    }

    commands.insert_resource(atlas);
    info!("✓ Moon atlas loaded");
}
