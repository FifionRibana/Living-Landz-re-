use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

#[derive(Clone, Copy, ShaderType)]
pub struct LakeParams {
    pub world_width: f32,
    pub world_height: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for LakeParams {
    fn default() -> Self {
        Self {
            world_width: 9600.0,
            world_height: 5030.0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct LakeMaterial {
    #[texture(0)]
    #[sampler(1, sampler_type = "filtering")]
    pub mask_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3, sampler_type = "filtering")]
    pub sdf_texture: Handle<Image>,

    #[uniform(4)]
    pub shallow_color: LinearRgba,

    #[uniform(5)]
    pub deep_color: LinearRgba,

    #[uniform(6)]
    pub params: LakeParams,
}

impl Default for LakeMaterial {
    fn default() -> Self {
        Self {
            mask_texture: Handle::default(),
            sdf_texture: Handle::default(),
            shallow_color: LinearRgba::new(0.15, 0.35, 0.38, 1.0),
            deep_color: LinearRgba::new(0.05, 0.12, 0.20, 1.0),
            params: LakeParams::default(),
        }
    }
}

impl Material2d for LakeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lake.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}