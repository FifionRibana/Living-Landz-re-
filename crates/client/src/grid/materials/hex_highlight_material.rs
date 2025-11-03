use bevy::asset::{Handle, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HexHighlightMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub time: f32,
}

impl Material2d for HexHighlightMaterial {
    fn fragment_shader() -> ShaderRef {
        eprintln!("Loading shader: shaders/hexagon_highlight.wgsl");
        "shaders/hexagon_highlight.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
