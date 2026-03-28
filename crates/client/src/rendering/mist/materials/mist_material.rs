use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

#[derive(Clone, Copy, ShaderType)]
pub struct MistParams {
    pub world_width: f32,
    pub world_height: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for MistParams {
    fn default() -> Self {
        Self {
            world_width: 192000.0,
            world_height: 100600.0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MistMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mist_texture: Handle<Image>,

    #[uniform(2)]
    pub params: MistParams,
}

impl Material2d for MistMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mist.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}