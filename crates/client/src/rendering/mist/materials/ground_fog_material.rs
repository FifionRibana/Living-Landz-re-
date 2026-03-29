use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

#[derive(Clone, Copy, ShaderType)]
pub struct GroundFogParams {
    pub world_width: f32,
    pub world_height: f32,
    pub camera_x: f32,
    pub camera_y: f32,
}

impl Default for GroundFogParams {
    fn default() -> Self {
        Self {
            world_width: 192000.0,
            world_height: 100600.0,
            camera_x: 0.0,
            camera_y: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GroundFogMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mist_texture: Handle<Image>,

    #[uniform(2)]
    pub params: GroundFogParams,
}

impl Material2d for GroundFogMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ground_fog.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}