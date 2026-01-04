use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Un segment du contour avec sa normale (pointant vers l'intérieur)
#[derive(Clone, Copy, Debug)]
pub struct ContourSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub normal: Vec2, // Normale unitaire pointant vers l'intérieur
}

impl ContourSegment {
    pub fn new(start: Vec2, end: Vec2, interior_side: Vec2) -> Self {
        let dir = (end - start).normalize();
        // Normale perpendiculaire
        let perp = Vec2::new(-dir.y, dir.x);

        // Choisir le sens qui pointe vers l'intérieur
        let midpoint = (start + end) * 0.5;
        let normal = if (midpoint + perp - interior_side).length()
            < (midpoint - perp - interior_side).length()
        {
            perp
        } else {
            -perp
        };

        Self { start, end, normal }
    }

    /// Créer un segment à partir d'un contour ordonné (sens horaire = intérieur à droite)
    pub fn from_contour_points(start: Vec2, end: Vec2) -> Self {
        let dir = (end - start).normalize();
        // Pour un contour sens horaire, l'intérieur est à droite
        let normal = Vec2::new(dir.y, -dir.x);
        Self { start, end, normal }
    }
}

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

// Point du contour pour le GPU
#[derive(Clone, Copy, Default, ShaderType)]
pub struct GpuContourPoint {
    pub position: Vec2,
    pub _padding: Vec2,
}

// Material pour le rendu du territoire
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerritoryMaterial {
    #[uniform(0)]
    pub settings: TerritorySettings,

    #[storage(1, read_only)]
    pub contour_points: Handle<ShaderStorageBuffer>,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct TerritorySettings {
    pub num_points: u32,
    pub border_width: f32,
    pub fade_distance: f32,
    pub _padding: f32,
    pub border_color: Vec4,
    pub fill_color: Vec4,
}

impl Material2d for TerritoryMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/territory_border_dvlp.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
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

pub fn create_territory_material(
    contour_points: &[Vec2],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerritoryMaterial>>,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    border_color: Color,
    fill_color: Color,
) -> (Handle<Mesh>, Handle<TerritoryMaterial>) {
    // Calculer la bounding box du contour
    let (min, max) = compute_bounds(contour_points);
    let padding = 50.0; // Marge pour le fondu

    let min_padded = min - Vec2::splat(padding);
    let max_padded = max + Vec2::splat(padding);
    let size = max_padded - min_padded;
    let _center = (min_padded + max_padded) * 0.5;

    // Créer un quad couvrant le territoire
    let mesh = Rectangle::new(size.x, size.y);
    let mesh_handle = meshes.add(mesh);

    // Préparer les points pour le GPU
    let gpu_points: Vec<GpuContourPoint> = contour_points
        .iter()
        .map(|p| GpuContourPoint {
            position: *p,
            _padding: Vec2::ZERO,
        })
        .collect();

    // Créer le storage buffer
    let mut buffer = ShaderStorageBuffer::default();
    buffer.set_data(gpu_points);
    let buffer_handle = buffers.add(buffer);

    // Créer le material
    let material = TerritoryMaterial {
        settings: TerritorySettings {
            num_points: contour_points.len() as u32,
            border_width: 2.0,
            fade_distance: 15.0,
            _padding: 0.0,
            border_color: border_color.to_linear().to_vec4(),
            fill_color: fill_color.to_linear().to_vec4(),
        },
        contour_points: buffer_handle,
    };
    let material_handle = materials.add(material);

    (mesh_handle, material_handle)
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

pub const CHUNK_WIDTH: f32 = 600.0;
pub const CHUNK_HEIGHT: f32 = 503.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    pub fn from_world_pos(pos: Vec2) -> Self {
        Self {
            x: (pos.x / CHUNK_WIDTH).floor() as i32,
            y: (pos.y / CHUNK_HEIGHT).floor() as i32,
        }
    }

    /// Retourne le rectangle (min, max) du chunk en coordonnées monde
    pub fn bounds(&self) -> (Vec2, Vec2) {
        let min = Vec2::new(self.x as f32 * CHUNK_WIDTH, self.y as f32 * CHUNK_HEIGHT);
        let max = min + Vec2::new(CHUNK_WIDTH, CHUNK_HEIGHT);
        (min, max)
    }
}

pub fn create_chunk_contour_material(
    chunk: ChunkCoord,
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
    let chunk_center = (chunk_min + chunk_max) * 0.5;

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
