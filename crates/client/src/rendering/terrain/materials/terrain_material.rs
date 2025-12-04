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

    // Road bindings (matching terrain_signed_sdf.wgsl)
    #[texture(5)]
    #[sampler(6)]
    pub road_sdf_texture: Handle<Image>,

    #[uniform(7)]
    pub road_params: RoadParams,

    #[uniform(8)]
    pub road_color_light: LinearRgba,

    #[uniform(9)]
    pub road_color_dark: LinearRgba,

    #[uniform(10)]
    pub road_color_tracks: LinearRgba,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct SdfParams {
    pub beach_start: f32,
    pub beach_end: f32,
    pub has_coast: f32,  // 1.0 if terrain has coast, 0.0 otherwise
    pub _padding: f32,   // Unused, kept for vec4 alignment
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct RoadParams {
    pub has_roads: f32,
    pub edge_softness: f32,
    pub noise_frequency: f32,
    pub noise_amplitude: f32,
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
                has_coast: 0.0,
                _padding: 0.0,
            },
            road_sdf_texture: Handle::default(), // Will be replaced with dummy texture
            road_params: RoadParams {
                has_roads: 0.0,
                edge_softness: 2.0,
                noise_frequency: 0.15,
                noise_amplitude: 3.0,
            },
            road_color_light: LinearRgba::new(0.76, 0.70, 0.55, 1.0),
            road_color_dark: LinearRgba::new(0.55, 0.48, 0.38, 1.0),
            road_color_tracks: LinearRgba::new(0.40, 0.35, 0.28, 1.0),
        }
    }
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_signed_sdf.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}