use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct BlurSettings {
    pub iterations: u32,
    pub scale: u32,
}

#[derive(Resource, Default)]
pub struct BlurredSceneTexture {
    pub handle: Option<Handle<Image>>,
}