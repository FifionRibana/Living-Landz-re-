use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

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
