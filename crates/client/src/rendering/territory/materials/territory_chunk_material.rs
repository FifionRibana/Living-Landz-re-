use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};
use shared::{ContourSegment, TerrainChunkId};

#[derive(Clone, Copy, Default, ShaderType)]
pub struct GpuContourSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub normal: Vec2,
    pub _padding: Vec2,
}

impl From<&ContourSegment> for GpuContourSegment {
    fn from(seg: &ContourSegment) -> Self {
        Self {
            start: seg.start,
            end: seg.end,
            normal: seg.normal,
            _padding: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct ChunkContourSettings {
    pub num_segments: u32,
    pub border_width: f32,
    pub fade_distance: f32,
    pub _padding: f32,
    pub border_color: Vec4,
    pub fill_color: Vec4,
}

// Material pour le rendu du territoire
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerritoryChunkMaterial {
    #[uniform(0)]
    pub settings: ChunkContourSettings,

    #[storage(1, read_only)]
    pub segments: Handle<ShaderStorageBuffer>,
}

impl Material2d for TerritoryChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/territory_border_single_dvlp.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub fn compute_bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);

    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }

    (min, max)
}

pub fn create_chunk_contour_material(
    chunk: TerrainChunkId,
    segments: &[ContourSegment],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerritoryChunkMaterial>>,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    border_color: Color,
    fill_color: Color,
    border_width: f32,
    fade_distance: f32,
) -> Option<(Handle<Mesh>, Handle<TerritoryChunkMaterial>)> {
    if segments.is_empty() {
        return None;
    }

    let (chunk_min, chunk_max) = chunk.bounds();
    let chunk_size = chunk_max - chunk_min;

    // Mesh couvrant le chunk entier
    let mesh = Rectangle::new(chunk_size.x, chunk_size.y);
    let mesh_handle = meshes.add(mesh);

    // Segments pour le GPU
    let gpu_segments: Vec<GpuContourSegment> = segments.iter().map(|s| s.into()).collect();

    let mut buffer = ShaderStorageBuffer::default();
    buffer.set_data(gpu_segments);
    let buffer_handle = buffers.add(buffer);

    let material = TerritoryChunkMaterial {
        settings: ChunkContourSettings {
            num_segments: segments.len() as u32,
            border_width,
            fade_distance,
            _padding: 0.0,
            border_color: border_color.to_linear().to_vec4(),
            fill_color: fill_color.to_linear().to_vec4(),
        },
        segments: buffer_handle,
    };
    let material_handle = materials.add(material);

    Some((mesh_handle, material_handle))
}
