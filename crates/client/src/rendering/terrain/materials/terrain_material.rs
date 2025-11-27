use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup};
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

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
    pub params: Vec4,
}

impl Default for TerrainMaterial {
    fn default() -> Self {
        Self {
            sdf_texture: Handle::default(),
            sand_color: LinearRgba::new(0.76, 0.7, 0.5, 1.0),
            grass_color: LinearRgba::new(0.36, 0.52, 0.28, 1.0),
            params: Vec4::new(0.0, 0.2, 1.0, 0.0), // has_coast = 1.0 by default
        }
    }
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        eprintln!("Loading shader: shaders/terrain_sdf.wgsl");
        "shaders/terrain_sdf.wgsl".into()
    }
}

/// Resource contenant le material partagé pour les chunks sans côte
#[derive(Resource)]
pub struct DefaultTerrainMaterial(pub Handle<TerrainMaterial>);