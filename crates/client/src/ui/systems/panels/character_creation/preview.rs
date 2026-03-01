use bevy::prelude::*;

use super::components::*;
use super::resources::CharacterCreationState;

/// Updates counter texts (e.g. "3 / 8") when the creation state changes.
pub fn update_counter_texts(
    creation_state: Res<CharacterCreationState>,
    mut query: Query<(&LayerCounterText, &mut Text)>,
) {
    if !creation_state.is_changed() {
        return;
    }

    for (counter, mut text) in query.iter_mut() {
        if let Some(layer) = creation_state.layer(counter.category) {
            **text = layer.counter_text();
        }
    }
}

/// Updates layer preview images when the creation state changes.
pub fn update_preview_images(
    creation_state: Res<CharacterCreationState>,
    mut query: Query<(&LayerPreviewImage, &mut ImageNode)>,
    asset_server: Res<AssetServer>,
) {
    if !creation_state.is_changed() {
        return;
    }

    for (preview, mut image_node) in query.iter_mut() {
        if let Some(layer) = creation_state.layer(preview.category) {
            let path = layer.asset_path();
            image_node.image = asset_server.load(&path);
        }
    }
}
