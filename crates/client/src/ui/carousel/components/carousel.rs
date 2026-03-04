use bevy::prelude::*;

#[derive(Component)]
pub struct Carousel {
    pub scroll_offset: f32,    // Position actuelle du défilement
    pub item_width: f32,      // Largeur d'une carte
    pub spacing: f32,         // Espace entre les cartes
    pub total_items: usize,   // Nombre total d'actions
}
