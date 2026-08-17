//! LL-E stage B: dedicated flowing-river water render.
//!
//! Reads `rivers.json` client-side (like the debug overlay), rasterizes a **river
//! coverage** texture (R = channel coverage, 1 in the core → 0 at the banks; width
//! by Strahler order) aligned to the terrain's world-width mapping, and draws one
//! full-world quad with [`RiverMaterial`] (`shaders/river.wgsl`). The shader
//! discards outside the channel and alpha-blends the water onto the terrain, so no
//! terrain-shader change is needed. Sits above terrain/lake, below trees.
//! No-op for maps without a `.ymir` (Azgaar).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d};

use shared::rivers::RiverNetwork;

use crate::state::resources::WorldCache;
use crate::states::AppState;

/// In front of the terrain (−1000) so rivers show on land, but BEHIND the ocean
/// (−500) and lake (−100) surfaces so rivers visibly empty *into* them (the
/// standing water occludes the river mouth) rather than drawing on top.
const RIVER_Z: f32 = -600.0;

/// Coverage texture resolution relative to the heightmap grid. The heightmap is
/// one texel per Ymir cell (~500 m); ×2 gives ~250 m texels so channels can read
/// as rivers, not lakes. (2048×2 = 4096² R8 ≈ 16 MB, built once at load.)
const COVERAGE_UPSCALE: usize = 2;

#[derive(Clone, Copy, ShaderType)]
pub struct RiverParams {
    pub world_width: f32,
    pub world_height: f32,
    pub flow_speed: f32,
    pub _padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct RiverMaterial {
    #[texture(0)]
    #[sampler(1, sampler_type = "filtering")]
    pub coverage: Handle<Image>,
    #[uniform(2)]
    pub params: RiverParams,
    #[uniform(3)]
    pub shallow_color: LinearRgba,
    #[uniform(4)]
    pub deep_color: LinearRgba,
    #[uniform(5)]
    pub foam_color: LinearRgba,
}

impl Material2d for RiverMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/river.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct RiverEntity;

pub struct RiverPlugin;

impl Plugin for RiverPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<RiverMaterial>::default())
            .add_systems(Update, spawn_river.run_if(in_state(AppState::InGame)));
    }
}

/// Channel core radius in texels by Strahler order (higher = wider). A 1-texel
/// bank falloff is added on top. 1 texel ≈ one cell on the ground, so keep these
/// small — even a trunk should read as a river, not a lake.
fn strahler_core_radius(order: u32) -> i32 {
    // Aligned to the server's per-cell River width (`strahler_radius_cells`,
    // 0..=5 → 0, _ → 1) so the rendered channel tracks the River/Riverbank cells
    // and doesn't spill onto grassland. (Client texels are ~2× finer, so radius 0
    // + bank ≈ the server's 1-cell channel.)
    match order {
        0..=5 => 0,
        _ => 1,
    }
}

fn read_rivers_file(map_name: &str) -> Option<String> {
    let rel = format!("assets/maps/{map_name}.ymir/rivers.json");
    for base in ["../../", ""] {
        if let Ok(s) = std::fs::read_to_string(format!("{base}{rel}")) {
            return Some(s);
        }
    }
    None
}

fn spawn_river(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RiverMaterial>>,
    mut images: ResMut<Assets<Image>>,
    world_cache: Option<Res<WorldCache>>,
    existing: Query<Entity, With<RiverEntity>>,
) {
    if !existing.is_empty() {
        return;
    }
    let Some(world_cache) = world_cache else {
        return;
    };
    let Some(tg) = world_cache.get_terrain_global() else {
        return;
    };
    let Some(raw) = read_rivers_file(&tg.name) else {
        return;
    };
    let network = RiverNetwork::parse(&raw);
    if network.segments.is_empty() {
        return;
    }

    let gw = tg.heightmap_width.max(1) as usize;
    let gh = tg.heightmap_height.max(1) as usize;
    let tex_w = gw * COVERAGE_UPSCALE;
    let tex_h = gh * COVERAGE_UPSCALE;
    let coverage = build_river_coverage(&network, gw, gh, tex_w, tex_h);

    let mut image = Image::new(
        Extent3d {
            width: tex_w as u32,
            height: tex_h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        coverage,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    let coverage_handle = images.add(image);

    // Size the river quad to the CELL grid extent (heightmap dims × the cell
    // scale), not the chunk-rounded world_width the biome texture uses — so the
    // rendered channel lands on the River/Riverbank hex cells instead of ~½-cell
    // off. Falls back to world_width when the scale is unknown (legacy/Azgaar).
    let upc = if tg.world_units_per_cell > 0.0 {
        tg.world_units_per_cell
    } else {
        tg.world_width / gw as f32
    };
    let river_world_w = gw as f32 * upc;
    let river_world_h = gh as f32 * upc;

    let mesh = create_river_mesh(river_world_w, river_world_h);

    commands.spawn((
        RiverEntity,
        Name::new("Rivers"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(RiverMaterial {
            coverage: coverage_handle,
            params: RiverParams {
                world_width: river_world_w,
                world_height: river_world_h,
                flow_speed: 1.0,
                _padding: 0.0,
            },
            // Match the painterly lake palette so rivers read as the same water.
            shallow_color: LinearRgba::new(0.15, 0.35, 0.38, 1.0),
            deep_color: LinearRgba::new(0.05, 0.12, 0.20, 1.0),
            foam_color: LinearRgba::new(0.88, 0.90, 0.92, 1.0),
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, RIVER_Z)),
    ));
    info!("Spawned river water from {} segments", network.segments.len());
}

/// Rasterize river coverage into a `tex_w×tex_h` R8 buffer (cell grid `gw×gh`
/// upscaled by `COVERAGE_UPSCALE`). Ymir points are cell space (y=0 south) →
/// Y-flipped to match the heightmap/biome textures (`gh-1-cy`) then scaled to
/// texels, so the water sits on the valleys. Segments are stamped along
/// interpolated steps (so thin channels stay continuous at higher resolution);
/// coverage is 1.0 within the core radius, falling to 0 across a 1-texel bank
/// margin; overlapping segments max-blend.
fn build_river_coverage(net: &RiverNetwork, gw: usize, gh: usize, tex_w: usize, tex_h: usize) -> Vec<u8> {
    let mut cov = vec![0u8; tex_w * tex_h];
    let sx = tex_w as f32 / gw as f32;
    let sy = tex_h as f32 / gh as f32;
    for seg in &net.segments {
        let core = strahler_core_radius(seg.strahler_order);
        let bank = core + 1;
        let bank2 = bank * bank;
        let span = (bank - core).max(1) as f32;
        // Segment points in texel space (cell → texel, with the heightmap Y-flip).
        let pts: Vec<(f32, f32)> = seg
            .points
            .iter()
            .map(|p| (p[0] * sx, (gh as f32 - 1.0 - p[1]) * sy))
            .collect();
        for w in pts.windows(2) {
            let (ax, ay) = w[0];
            let (bx, by) = w[1];
            let steps = ((bx - ax).hypot(by - ay).ceil() as i32).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                stamp_disc(&mut cov, tex_w, tex_h, ax + (bx - ax) * t, ay + (by - ay) * t, core, bank, bank2, span);
            }
        }
    }
    // No global blur: it spread the water outward over grassland cells. The
    // shader's 9-tap coverage sampling + shore-noise handle in-channel smoothing,
    // so the render stays within the River/Riverbank footprint.
    cov
}

#[allow(clippy::too_many_arguments)]
fn stamp_disc(
    cov: &mut [u8],
    tex_w: usize,
    tex_h: usize,
    px: f32,
    py: f32,
    core: i32,
    bank: i32,
    bank2: i32,
    span: f32,
) {
    let cx = px.round() as i32;
    let cy = py.round() as i32;
    for dy in -bank..=bank {
        for dx in -bank..=bank {
            let d2 = dx * dx + dy * dy;
            if d2 > bank2 {
                continue;
            }
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= tex_w as i32 || y >= tex_h as i32 {
                continue;
            }
            let d = (d2 as f32).sqrt();
            let v = if d <= core as f32 {
                1.0
            } else {
                (1.0 - (d - core as f32) / span).clamp(0.0, 1.0)
            };
            let vv = (v * 255.0) as u8;
            let idx = (y as usize) * tex_w + (x as usize);
            if vv > cov[idx] {
                cov[idx] = vv;
            }
        }
    }
}

/// Full-world quad `[0,world_w] × [0,world_h]`, uv 0..1.
fn create_river_mesh(world_w: f32, world_h: f32) -> Mesh {
    let positions: Vec<[f32; 3]> = vec![
        [0.0, 0.0, 0.0],
        [world_w, 0.0, 0.0],
        [world_w, world_h, 0.0],
        [0.0, world_h, 0.0],
    ];
    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 4];
    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(indices)
}
