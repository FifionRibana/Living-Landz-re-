use bevy::prelude::*;

/// Handle partagé vers la texture de rendu de la scène.
#[derive(Resource)]
pub struct SceneRenderTarget(pub Handle<Image>);

/// Handle vers la texture de rendu de la scène cellule.
/// None quand pas encore initialisé ou hors cell view.
#[derive(Resource, Default)]
pub struct CellSceneRenderTarget {
    pub handle: Option<Handle<Image>>,
}