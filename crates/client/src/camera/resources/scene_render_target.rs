use bevy::prelude::*;

/// Handle partagé vers la texture de rendu de la scène.
#[derive(Resource)]
pub struct SceneRenderTarget(pub Handle<Image>);