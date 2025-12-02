// client/src/rendering/terrain/materials.rs

use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub sdf_texture: Handle<Image>,
    
    #[uniform(2)]
    pub sand_color: LinearRgba,
    
    #[uniform(3)]
    pub grass_color: LinearRgba,
    
    #[uniform(4)]
    pub sdf_params: SdfParams,

    #[uniform(5)]
    pub wave_params: WaveParams,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct SdfParams {
    pub beach_start: f32,
    pub beach_end: f32,
    pub opacity_start: f32,
    pub opacity_end: f32,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct WaveParams {
    pub time: f32,
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub foam_width: f32,
}

impl Default for TerrainMaterial {
    fn default() -> Self {
        Self {
            sdf_texture: Handle::default(),
            sand_color: LinearRgba::new(0.76, 0.70, 0.50, 1.0),
            grass_color: LinearRgba::new(0.36, 0.52, 0.28, 1.0),
            sdf_params: SdfParams {
                beach_start: -0.1,
                beach_end: 0.4,
                opacity_start: -0.4,
                opacity_end: 0.0,
            },
            wave_params: WaveParams {
                time: 0.0,
                wave_speed: 1.0,
                wave_amplitude: 0.08,
                foam_width: 0.12,
            },
        }
    }
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_sdf.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}