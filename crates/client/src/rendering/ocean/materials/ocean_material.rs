// client/src/rendering/ocean/materials.rs

use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

#[derive(Clone, Copy, ShaderType)]
pub struct OceanParams {
    pub time: f32,
    pub world_width: f32,
    pub world_height: f32,
    pub max_depth: f32,
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub foam_width: f32,
    pub _padding: f32,
}

impl Default for OceanParams {
    fn default() -> Self {
        Self {
            time: 0.0,
            world_width: 6000.0,  // Ajuster selon ta carte
            world_height: 5000.0,
            max_depth: 100.0,
            wave_speed: 1.0,
            wave_amplitude: 0.08,
            foam_width: 0.15,
            _padding: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct OceanMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub heightmap: Handle<Image>,
    
    #[texture(2)]
    #[sampler(3)]
    pub sdf_texture: Handle<Image>,
    
    #[uniform(4)]
    pub shallow_color: LinearRgba,
    
    #[uniform(5)]
    pub deep_color: LinearRgba,
    
    #[uniform(6)]
    pub foam_color: LinearRgba,
    
    #[uniform(7)]
    pub params: OceanParams,
}

impl Default for OceanMaterial {
    fn default() -> Self {
        Self {
            heightmap: Handle::default(),
            sdf_texture: Handle::default(), 
            shallow_color: LinearRgba::new(0.15, 0.40, 0.50, 1.0),
            deep_color: LinearRgba::new(0.02, 0.08, 0.15, 1.0),
            foam_color: LinearRgba::new(0.92, 0.95, 1.0, 1.0),
            params: OceanParams::default(),
        }
    }
}

impl Material2d for OceanMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ocean.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}