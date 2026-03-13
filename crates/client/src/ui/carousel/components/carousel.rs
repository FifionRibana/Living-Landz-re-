use bevy::prelude::*;

#[derive(Component)]
pub struct Carousel {
    pub id: u32,
    pub enabled: bool, 
    pub item_width: f32,      // Largeur d'une carte
    pub spacing: f32,         // Espace entre les cartes
    pub total_items: usize,   // Nombre total d'actions
    pub current_scroll: f32, // Ce qui est affiché
    pub target_scroll: f32,  // La destination souhaitée
    pub lerp_speed: f32,     // Vitesse de rattrapage (ex: 10.0)
    pub snap_timer: f32, // Temps écoulé depuis le dernier input
}

// TODO: Create a default
