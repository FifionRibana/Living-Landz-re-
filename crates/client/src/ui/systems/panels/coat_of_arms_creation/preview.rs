use bevy::prelude::*;

use super::components::*;
use super::resources::CoatOfArmsCreationState;

/// Updates counter texts when the creation state changes.
pub fn update_heraldry_counter_texts(
    creation_state: Res<CoatOfArmsCreationState>,
    mut query: Query<(&HeraldryCounterText, &mut Text)>,
) {
    if !creation_state.is_changed() {
        return;
    }

    for (counter, mut text) in query.iter_mut() {
        if let Some(layer_state) = creation_state.layer(counter.layer) {
            **text = layer_state.counter_text();
        }
    }
}

/// Updates heraldry preview images when the creation state changes.
pub fn update_heraldry_preview_images(
    creation_state: Res<CoatOfArmsCreationState>,
    mut query: Query<(&HeraldryPreviewImage, &mut ImageNode)>,
    asset_server: Res<AssetServer>,
) {
    if !creation_state.is_changed() {
        return;
    }

    for (preview, mut image_node) in query.iter_mut() {
        if let Some(layer_state) = creation_state.layer(preview.layer) {
            let path = layer_state.asset_path();
            image_node.image = asset_server.load(&path);
        }
    }
}
