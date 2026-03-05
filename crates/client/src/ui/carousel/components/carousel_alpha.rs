use bevy::prelude::*;

#[derive(Component)]
pub struct CarouselAlpha {
    /// L'alpha cible défini par le design (ex: 1.0 pour un texte plein, 0.5 pour un décor)
    pub base_alpha: f32,
}

impl CarouselAlpha {
    pub fn new(alpha: f32) -> Self {
        Self { base_alpha: alpha }
    }
}