use bevy::prelude::*;

#[derive(Component)]
pub struct CarouselAlpha {
    /// L'alpha cible défini par le design (ex: 1.0 pour un texte plein, 0.5 pour un décor)
    pub base_alpha: f32,
    pub has_visible_background: bool,
}

impl CarouselAlpha {
    pub fn new(alpha: f32) -> Self {
        Self { 
            base_alpha: alpha,
            has_visible_background: false
        }
    }

    pub fn with_background(mut self) -> Self {
        self.has_visible_background = true;
        self
    }
}