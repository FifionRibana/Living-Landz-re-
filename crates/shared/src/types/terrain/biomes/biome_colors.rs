use std::{hash::{Hash, Hasher}, ops::Deref};

use super::biome_type::BiomeType;
use bevy::color::{Color, ColorToPacked};

pub struct BiomeColor(pub Color);

impl Hash for BiomeColor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_srgba().to_u8_array().hash(state);
    }
}

impl PartialEq for BiomeColor {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_srgba().to_u8_array() == other.0.to_srgba().to_u8_array()
    }
}

impl Eq for BiomeColor {}

impl BiomeColor {
    pub fn srgb_u8(r: u8, g: u8, b: u8) -> Self {
        BiomeColor(Color::srgb_u8(r, g, b))
    }

    pub fn distance_to(&self, other: &BiomeColor) -> i32 {
        let c1 = self.0.to_srgba();
        let c2 = other.0.to_srgba();

        let dr = (c1.red - c2.red) * u8::MAX as f32;
        let dg = (c1.green - c2.green) * u8::MAX as f32;
        let db = (c1.blue - c2.blue) * u8::MAX as f32;

        (dr * dr + dg * dg + db * db).sqrt() as i32
    }

    pub fn as_color(&self) -> &Color {
        &self.0
    }

    pub fn red(&self) -> u8 {
        (self.to_srgba().red * u8::MAX as f32) as u8
    }

    pub fn green(&self) -> u8 {
        (self.to_srgba().green * u8::MAX as f32) as u8
    }

    pub fn blue(&self) -> u8 {
        (self.to_srgba().blue * u8::MAX as f32) as u8
    }
}

impl Deref for BiomeColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn get_biome_color(biome: &BiomeType) -> BiomeColor {
    match biome {
        BiomeType::Ocean => BiomeColor::srgb_u8(0, 15, 30),
        BiomeType::DeepOcean => BiomeColor::srgb_u8(0, 50, 80),
        BiomeType::Desert => BiomeColor::srgb_u8(251, 231, 159),
        BiomeType::Savanna => BiomeColor::srgb_u8(210, 208, 130),
        BiomeType::Grassland => BiomeColor::srgb_u8(200, 214, 143),
        BiomeType::TropicalSeasonalForest => BiomeColor::srgb_u8(182, 217, 93),
        BiomeType::TropicalRainForest => BiomeColor::srgb_u8(125, 203, 52),
        BiomeType::TropicalDeciduousForest => BiomeColor::srgb_u8(41, 188, 86),
        BiomeType::TemperateRainForest => BiomeColor::srgb_u8(64, 156, 67),
        BiomeType::Wetland => BiomeColor::srgb_u8(11, 145, 49),
        BiomeType::Taiga => BiomeColor::srgb_u8(75, 107, 50),
        BiomeType::Tundra => BiomeColor::srgb_u8(150, 120, 75),
        BiomeType::Lake => BiomeColor::srgb_u8(51, 115, 121),
        BiomeType::ColdDesert => BiomeColor::srgb_u8(181, 184, 135),
        BiomeType::Ice => BiomeColor::srgb_u8(213, 231, 235),
        BiomeType::Undefined => BiomeColor::srgb_u8(0, 0, 0),
    }
}

pub fn get_biome_from_color(rgba: &[u8; 4]) -> BiomeType {
    let (r, g, b) = (rgba[0], rgba[1], rgba[2]);

    match (r, g, b) {
        (0, 15, 30) => BiomeType::Ocean,
        (0, 50, 80) => BiomeType::DeepOcean,
        (251, 231, 159) => BiomeType::Desert,
        (210, 208, 130) => BiomeType::Savanna,
        (200, 214, 143) => BiomeType::Grassland,
        (182, 217, 93) => BiomeType::TropicalSeasonalForest,
        (125, 203, 52) => BiomeType::TropicalRainForest,
        (41, 188, 86) => BiomeType::TropicalDeciduousForest,
        (64, 156, 67) => BiomeType::TemperateRainForest,
        (11, 145, 49) => BiomeType::Wetland,
        (75, 107, 50) => BiomeType::Taiga,
        (150, 120, 75) => BiomeType::Tundra,
        (51, 115, 121) => BiomeType::Lake,
        (181, 184, 135) => BiomeType::ColdDesert,
        (213, 231, 235) => BiomeType::Ice,
        _ => BiomeType::Undefined,
    }
}

pub fn find_closest_biome(pixel_color: &BiomeColor) -> BiomeType {
    let known_colors: Vec<(BiomeType, BiomeColor)> = BiomeType::iter()
        .map(|b| (b, get_biome_color(&b)))
        .collect();
    if pixel_color.red() == 0 && pixel_color.green() == 0 && pixel_color.blue() == 0 {
        BiomeType::Ocean
    } else {
        let distances: Vec<(BiomeType, i32)> = known_colors
            .iter()
            .map(|(biome, color)| (*biome, pixel_color.distance_to(color)))
            .collect();

        distances
            .iter()
            .min_by_key(|&(_, dist)| dist)
            .map(|(biome, _)| *biome)
            .unwrap()
    }
}
