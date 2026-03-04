use bevy::prelude::*;

#[derive(Component)]
pub struct CarouselItem {
    pub index: usize,         // L'index d'origine de l'action
}