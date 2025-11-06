use bevy::prelude::*;
use hexx::{HexLayout, HexOrientation};

/// Resource wrapper pour la configuration hexagonale
#[derive(Resource, Clone)]
pub struct GridConfig {
    pub layout: HexLayout,
    pub hex_radius: f32,
    pub chunk_size: u8,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self::new(48.0, HexOrientation::Flat, Vec2::splat(1.0), 8)
    }
}

impl GridConfig {
    /// Crée une configuration avec un rayon donné
    pub fn new(radius: f32, orientation: HexOrientation, ratio: Vec2, chunk_size: u8) -> Self {
        let mut layout;
        match orientation {
            HexOrientation::Flat => {
                layout = HexLayout::flat();
            }
            HexOrientation::Pointy => {
                layout = HexLayout::pointy();
            }
        }
        layout = layout
            .with_hex_size(radius)
            .with_scale(Vec2::new(ratio[0] * radius, ratio[1] * radius));

        Self {
            layout,
            hex_radius: radius,
            chunk_size,
        }
    }
}
