use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TreeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for TreeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tree_atlas.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque // GPU traite comme opaque — le shader fait le discard
    }
}