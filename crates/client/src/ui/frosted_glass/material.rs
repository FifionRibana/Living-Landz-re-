// src/ui/frosted_glass/material.rs

use bevy::{prelude::*, render::render_resource::*, shader::ShaderRef};

/// Configuration pour un panneau en verre dépoli
#[derive(Clone, Debug)]
pub struct FrostedGlassConfig {
    pub color_top: Color,
    /// Couleur en haut du panneau
    pub color_bottom: Color,
    /// Couleur en bas du panneau
    pub opacity_top: f32,
    /// Opacité en haut (0.0 - 1.0)
    pub opacity_bottom: f32,
    /// Opacité en bas (0.0 - 1.0)
    pub edge_fade: f32,
    /// Fade horizontal pour effet carousel
    pub blur_strength: f32,
    /// Intensité du blur (0.0 - 1.0)
    pub border_radius: f32,
    /// Rayon des coins arrondis
    pub background_image: Option<Handle<Image>>,
}

impl Default for FrostedGlassConfig {
    fn default() -> Self {
        Self {
            color_top: Color::srgba(1.0, 1.0, 1.0, 0.95), // Blanc
            color_bottom: Color::srgba(0.92, 0.88, 0.82, 0.95), // Beige
            opacity_top: 0.3,
            opacity_bottom: 0.85,
            edge_fade: 0.0,
            blur_strength: 1.0,
            border_radius: 8.0,
            background_image: None,
        }
    }
}

impl FrostedGlassConfig {
    /// Preset pour les cartes (comme sur l'image de référence)
    pub fn card() -> Self {
        Self {
            color_top: Color::srgba(1.0, 1.0, 1.0, 1.0), // Blanc pur
            color_bottom: Color::srgba(0.92, 0.89, 0.83, 1.0), // Beige/crème
            opacity_top: 0.25,
            opacity_bottom: 0.95,
            edge_fade: 0.0,
            blur_strength: 1.0,
            border_radius: 8.0,
            background_image: None,
        }
    }

    /// Carte avec fade sur un bord (carousel)
    pub fn card_fading(direction: FadeDirection) -> Self {
        Self {
            edge_fade: match direction {
                FadeDirection::Left => -0.35,
                FadeDirection::Right => 0.35,
                FadeDirection::None => 0.0,
            },
            ..Self::card()
        }
    }

    /// Barre de ressources / header
    pub fn top_bar() -> Self {
        Self {
            color_top: Color::srgba(0.95, 0.93, 0.90, 1.0),
            color_bottom: Color::srgba(0.90, 0.87, 0.82, 1.0),
            opacity_top: 0.7,
            opacity_bottom: 0.8,
            edge_fade: 0.0,
            blur_strength: 0.8,
            border_radius: 4.0,
            background_image: None,
        }
    }

    /// Panel de dialogue / popup
    pub fn dialog() -> Self {
        Self {
            color_top: Color::srgba(1.0, 1.0, 1.0, 1.0),
            color_bottom: Color::srgba(0.95, 0.92, 0.88, 1.0),
            opacity_top: 0.25,
            opacity_bottom: 1.,
            edge_fade: 0.0,
            blur_strength: 3.0,
            border_radius: 8.0,
            background_image: None,
        }
    }

    /// Builder: définir les couleurs
    pub fn with_colors(mut self, top: Color, bottom: Color) -> Self {
        self.color_top = top;
        self.color_bottom = bottom;
        self
    }

    /// Builder: définir les opacités
    pub fn with_opacity(mut self, top: f32, bottom: f32) -> Self {
        self.opacity_top = top;
        self.opacity_bottom = bottom;
        self
    }

    /// Builder: définir le border radius
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_background(mut self, image: Handle<Image>) -> Self {
        self.background_image = Some(image);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FadeDirection {
    #[default]
    None,
    Left,
    Right,
}

// ============ MATERIAL ============

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FrostedGlassMaterial {
    #[uniform(0)]
    pub(crate) uniforms: FrostedGlassUniforms,

    #[texture(1)]
    #[sampler(2)]
    pub scene_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub background_image: Option<Handle<Image>>,
}

#[derive(Clone, Debug, Default, ShaderType)]
pub struct FrostedGlassUniforms {
    pub color_top: LinearRgba,
    pub color_bottom: LinearRgba,
    pub opacity_top: f32,
    pub opacity_bottom: f32,
    pub edge_fade: f32,
    pub blur_strength: f32,
    pub border_radius: f32,
    pub size: Vec2,
    pub screen_size: Vec2,
    pub use_background_image: u32,  // 0 = scene_texture, 1 = background_image
    pub _padding: Vec2, // Alignement 16 bytes
}

impl From<FrostedGlassConfig> for FrostedGlassMaterial {
    fn from(config: FrostedGlassConfig) -> Self {
        let use_background = config.background_image.is_some();
        Self {
            uniforms: FrostedGlassUniforms {
                color_top: config.color_top.into(),
                color_bottom: config.color_bottom.into(),
                opacity_top: config.opacity_top,
                opacity_bottom: config.opacity_bottom,
                edge_fade: config.edge_fade,
                blur_strength: config.blur_strength,
                border_radius: config.border_radius,
                size: Vec2::ZERO, // Sera mis à jour par le système
                screen_size: Vec2::ZERO,
                use_background_image: if use_background { 1 } else { 0 },
                _padding: Vec2::ZERO,
            },
            scene_texture: None, // Sera injecté par le plugin
            background_image: config.background_image,
        }
    }
}

impl UiMaterial for FrostedGlassMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/frosted_glass.wgsl".into()
    }
}
