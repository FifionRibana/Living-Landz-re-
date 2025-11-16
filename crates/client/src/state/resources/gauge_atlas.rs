use std::collections::HashMap;

use bevy::prelude::*;
use shared::atlas::GaugeAtlas;

pub fn setup_gauge_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut atlas = GaugeAtlas::default();
    atlas.load();

    for (name, sprite) in &atlas.sprites {
        let path = format!("ui/{}.png", sprite);
        atlas.handles.insert(name.clone(), asset_server.load(path));
    }

    commands.insert_resource(atlas);
    info!("✓ Gauge atlas loaded");
}
