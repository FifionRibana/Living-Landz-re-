// client/src/rendering/ocean/materials.rs

use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

#[derive(Clone, Copy, ShaderType)]
pub struct OceanParams {
    pub world_width: f32,
    pub world_height: f32,
    pub max_depth: f32,
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub foam_width: f32,
    /// Normalized sea level in the terrain heightmap (Ymir: ~0.574). `> 0` selects
    /// the SDF-driven shore + real metric-depth path in the shader; `0` (Azgaar)
    /// keeps the legacy inverted-heightmap bathymetry.
    pub sea_level_norm: f32,
    pub _padding2: f32,
}

impl Default for OceanParams {
    fn default() -> Self {
        Self {
            world_width: 6000.0,  // Ajuster selon ta carte
            world_height: 5000.0,
            max_depth: 100.0,
            wave_speed: 1.0,
            wave_amplitude: 0.08,
            foam_width: 0.15,
            sea_level_norm: 0.0,
            _padding2: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct OceanMaterial {
    #[texture(0)]
    #[sampler(1, sampler_type = "filtering")]
    pub heightmap: Handle<Image>,

    #[texture(2)]
    #[sampler(3, sampler_type = "filtering")]
    pub sdf_texture: Handle<Image>,

    #[uniform(4)]
    pub shallow_color: LinearRgba,

    #[uniform(5)]
    pub deep_color: LinearRgba,

    #[uniform(6)]
    pub foam_color: LinearRgba,

    #[uniform(7)]
    pub params: OceanParams,

    /// Terrain global heightmap (enriched, R16Unorm) — used for accurate
    /// coastline discard so the ocean boundary matches the heightmap's
    /// coastal slope rather than the binary map SDF.
    #[texture(8)]
    #[sampler(9, sampler_type = "filtering")]
    pub terrain_heightmap: Handle<Image>,

    #[uniform(10)]
    pub debug_params: OceanDebugParams,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct OceanDebugParams {
    /// Shader base mode (0=off, 1-6 = debug modes). Same encoding as terrain.
    pub debug_base: f32,
    /// Overlay bit-flags (bit 0 = level lines).
    pub overlay_flags: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for OceanMaterial {
    fn default() -> Self {
        Self {
            heightmap: Handle::default(),
            sdf_texture: Handle::default(),
            shallow_color: LinearRgba::new(0.18, 0.38, 0.42, 1.0),
            deep_color: LinearRgba::new(0.04, 0.10, 0.18, 1.0),
            foam_color: LinearRgba::new(0.88, 0.90, 0.92, 1.0),
            params: OceanParams::default(),
            terrain_heightmap: Handle::default(),
            debug_params: OceanDebugParams::default(),
        }
    }
}

impl Material2d for OceanMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ocean_painterly.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}