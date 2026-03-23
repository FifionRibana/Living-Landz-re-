// client/src/rendering/terrain/materials.rs

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

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

    /// Position et taille du chunk dans l'espace monde
    /// Permet au shader de calculer des coordonnées globales continues
    #[uniform(11)]
    pub chunk_info: ChunkInfo,

    // Biome texture (R8: biome ID per pixel)
    #[texture(12)]
    #[sampler(13)]
    pub biome_texture: Handle<Image>,

    #[uniform(14)]
    pub biome_params: BiomeParams,

    // Heightmap texture (R8: elevation 0-255)
    #[texture(15)]
    #[sampler(16)]
    pub heightmap_texture: Handle<Image>,

    #[uniform(17)]
    pub heightmap_params: HeightmapParams,

    // Lake SDF texture (R8: 0=deep lake, 128=shore, 255=deep land)
    #[texture(18)]
    #[sampler(19, sampler_type = "filtering")]
    pub lake_sdf_texture: Handle<Image>,

    #[uniform(20)]
    pub lake_params: LakeParams,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct SdfParams {
    pub beach_start: f32,
    pub beach_end: f32,
    pub has_coast: f32, // 1.0 if terrain has coast, 0.0 otherwise
    pub _padding: f32,  // Unused, kept for vec4 alignment
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct RoadParams {
    pub has_roads: f32,
    pub edge_softness: f32,
    pub noise_frequency: f32,
    pub noise_amplitude: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct BiomeParams {
    /// 1.0 if biome texture is present, 0.0 otherwise
    pub has_biome: f32,
    pub _padding: f32,
    /// Total world width in pixels (for global UV computation)
    pub world_width: f32,
    /// Total world height in pixels (for global UV computation)
    pub world_height: f32,
}

impl Default for BiomeParams {
    fn default() -> Self {
        Self {
            has_biome: 0.0,
            _padding: 0.0,
            world_width: 1.0,
            world_height: 1.0,
        }
    }
}

#[derive(Clone, Copy, ShaderType)]
pub struct HeightmapParams {
    /// 1.0 if heightmap is present, 0.0 otherwise
    pub has_heightmap: f32,
    /// Light direction angle in radians (azimuth, 0 = east, pi/2 = north)
    pub light_azimuth: f32,
    /// Light elevation angle (0 = horizon, pi/2 = zenith)
    pub light_altitude: f32,
    /// Hillshade intensity (0 = none, 1 = full)
    pub hillshade_strength: f32,
}

impl Default for HeightmapParams {
    fn default() -> Self {
        Self {
            has_heightmap: 0.0,
            light_azimuth: 5.5,   // ~315° = northwest (classic cartography)
            light_altitude: 0.75, // ~43° above horizon
            hillshade_strength: 0.6,
        }
    }
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct LakeParams {
    pub has_lake: f32,
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
}

/// Informations de positionnement du chunk dans le monde
/// Passées au shader pour que le bruit soit continu entre chunks
#[derive(Clone, Copy, ShaderType)]
pub struct ChunkInfo {
    /// Offset X du chunk dans l'espace monde
    pub world_offset_x: f32,
    /// Offset Y du chunk dans l'espace monde
    pub world_offset_y: f32,
    /// Largeur du chunk en pixels monde
    pub chunk_width: f32,
    /// Hauteur du chunk en pixels monde
    pub chunk_height: f32,
}

impl Default for ChunkInfo {
    fn default() -> Self {
        Self {
            world_offset_x: 0.0,
            world_offset_y: 0.0,
            chunk_width: 600.0,
            chunk_height: 503.0,
        }
    }
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
            road_sdf_texture: Handle::default(),
            road_params: RoadParams {
                has_roads: 0.0,
                edge_softness: 2.0,
                noise_frequency: 0.15,
                noise_amplitude: 3.0,
            },
            road_color_light: LinearRgba::new(0.76, 0.70, 0.55, 1.0),
            road_color_dark: LinearRgba::new(0.55, 0.48, 0.38, 1.0),
            road_color_tracks: LinearRgba::new(0.40, 0.35, 0.28, 1.0),
            chunk_info: ChunkInfo::default(),
            biome_texture: Handle::default(),
            biome_params: BiomeParams::default(),
            heightmap_texture: Handle::default(),
            heightmap_params: HeightmapParams::default(),
            lake_sdf_texture: Handle::default(),
            lake_params: LakeParams::default(),
        }
    }
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_signed_sdf_painterly.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
