use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

/// Material for rendering territory borders using SDF
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerritoryBorderMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub border_sdf_texture: Handle<Image>,

    #[uniform(2)]
    pub border_params: BorderParams,

    #[uniform(3)]
    pub time: f32,
}

/// Parameters for border rendering
#[derive(Clone, Copy, ShaderType)]
pub struct BorderParams {
    /// Line width in pixels
    pub line_width: f32,
    /// Edge softness for anti-aliasing
    pub edge_softness: f32,
    /// Glow intensity
    pub glow_intensity: f32,
    /// Border color
    pub color: LinearRgba,
}

impl Default for BorderParams {
    fn default() -> Self {
        Self {
            line_width: 3.0,
            edge_softness: 1.5,
            glow_intensity: 0.2,
            color: LinearRgba::new(1.0, 0.8, 0.2, 1.0), // Gold color
        }
    }
}

impl Material2d for TerritoryBorderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/territory_border.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
