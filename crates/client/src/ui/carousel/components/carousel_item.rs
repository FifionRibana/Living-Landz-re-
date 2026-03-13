use bevy::prelude::*;

#[derive(Component)]
pub struct CarouselItem {
    pub carousel_id: u32,
    pub index: usize,         // L'index d'origine de l'action
}