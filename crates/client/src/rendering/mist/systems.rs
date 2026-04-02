use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::camera::MainCamera;
use crate::networking::client::lightyear_client::SendRequestExplorationMap;
use crate::rendering::mist::materials::{
    GroundFogMaterial, GroundFogParams, MistMaterial, MistParams,
};
use crate::state::resources::WorldCache;
use shared::constants;

#[derive(Component)]
pub struct MistEntity;

/// Tracks whether the real ocean SDF has been injected into the mist material.
#[derive(Component)]
pub struct MistSdfLoaded;

#[derive(Component)]
pub struct GroundFogEntity;

#[derive(Component)]
pub struct GroundFogSdfLoaded;

fn create_mist_mesh(width: f32, height: f32) -> Mesh {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [width, 0.0, 0.0],
        [width, height, 0.0],
        [0.0, height, 0.0],
    ];
    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let indices = vec![0u32, 1, 2, 0, 2, 3];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

/// Creates a 1×1 fallback SDF texture (value = 0xFF = deep inland).
fn create_fallback_sdf(images: &mut Assets<Image>) -> Handle<Image> {
    let mut img = Image::new(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![255u8],
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = bevy::image::ImageSampler::linear();
    images.add(img)
}

pub fn request_exploration_map(
    mut cache: ResMut<WorldCache>,
    mut events: MessageWriter<SendRequestExplorationMap>,
) {
    if !cache.is_exploration_loaded() && !cache.is_exploration_requested() {
        info!("Requesting exploration map from server via lightyear");
        events.write(SendRequestExplorationMap {
            terrain_name: "Gaulyia".to_string(),
        });
        cache.mark_exploration_requested();
    }
}

pub fn spawn_mist(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut mist_materials: ResMut<Assets<MistMaterial>>,
    world_cache: Option<Res<WorldCache>>,
    existing: Query<Entity, With<MistEntity>>,
) {
    let Some(world_cache) = world_cache else { return; };
    if !world_cache.is_exploration_loaded() { return; }
    if existing.iter().count() > 0 { return; }

    let exploration = world_cache.exploration_cache();
    let tex_width = exploration.width;
    let tex_height = exploration.height;
    let n_chunk_x = exploration.n_chunk_x;
    let n_chunk_y = exploration.n_chunk_y;

    if tex_width == 0 || tex_height == 0 { return; }

    // Use raw exploration data — bilinear GPU filtering + shader smoothstep handles transition
    let mut mist_image = Image::new(
        Extent3d {
            width: tex_width as u32,
            height: tex_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        exploration.data.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    mist_image.sampler = bevy::image::ImageSampler::linear();
    let mist_texture = images.add(mist_image);

    // World dimensions derived from chunk grid, not texture resolution
    let world_width = n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let world_height = n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    let mesh = create_mist_mesh(world_width, world_height);

    // Use real ocean SDF if available, otherwise fallback (no coast masking yet)
    let sdf_texture = if let Some(ocean_data) = world_cache.get_ocean() {
        let mut sdf_image = Image::new(
            Extent3d {
                width: ocean_data.width as u32,
                height: ocean_data.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            ocean_data.sdf_values.clone(),
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        sdf_image.sampler = bevy::image::ImageSampler::linear();
        images.add(sdf_image)
    } else {
        create_fallback_sdf(&mut images)
    };

    let material = mist_materials.add(MistMaterial {
        mist_texture: mist_texture.clone(),
        params: MistParams {
            world_width,
            world_height,
            ..default()
        },
        sdf_texture,
    });

    info!(
        "🌫️ Spawning mist mesh: {}×{} world, {}×{} texture ({}×{} chunks, sdf={})",
        world_width, world_height, tex_width, tex_height, n_chunk_x, n_chunk_y,
        if world_cache.get_ocean().is_some() { "real" } else { "fallback" }
    );

    let mut entity = commands.spawn((
        Name::new("Mist"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
        MistEntity,
    ));

    if world_cache.get_ocean().is_some() {
        entity.insert(MistSdfLoaded);
    }
}

/// When ocean data arrives after the mist has already spawned,
/// replace the fallback SDF texture with the real one.
pub fn update_mist_sdf(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut mist_materials: ResMut<Assets<MistMaterial>>,
    mut gf_materials: ResMut<Assets<GroundFogMaterial>>,
    world_cache: Option<Res<WorldCache>>,
    mist_query: Query<(Entity, &MeshMaterial2d<MistMaterial>), (With<MistEntity>, Without<MistSdfLoaded>)>,
    gf_query: Query<(Entity, &MeshMaterial2d<GroundFogMaterial>), (With<GroundFogEntity>, Without<GroundFogSdfLoaded>)>,
) {
    let Some(world_cache) = world_cache else { return; };
    let Some(ocean_data) = world_cache.get_ocean() else { return; };

    // Update mist SDF
    for (entity, mat_handle) in mist_query.iter() {
        let Some(material) = mist_materials.get_mut(&mat_handle.0) else { continue; };

        let mut sdf_image = Image::new(
            Extent3d {
                width: ocean_data.width as u32,
                height: ocean_data.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            ocean_data.sdf_values.clone(),
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        sdf_image.sampler = bevy::image::ImageSampler::linear();

        material.sdf_texture = images.add(sdf_image);
        commands.entity(entity).insert(MistSdfLoaded);

        info!("🌫️ Mist SDF updated with real ocean data");
    }

    // Update ground fog SDF
    for (entity, mat_handle) in gf_query.iter() {
        let Some(material) = gf_materials.get_mut(&mat_handle.0) else { continue; };

        let mut sdf_image = Image::new(
            Extent3d {
                width: ocean_data.width as u32,
                height: ocean_data.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            ocean_data.sdf_values.clone(),
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        sdf_image.sampler = bevy::image::ImageSampler::linear();

        material.sdf_texture = images.add(sdf_image);
        commands.entity(entity).insert(GroundFogSdfLoaded);

        info!("🌁 Ground fog SDF updated with real ocean data");
    }
}

/// When exploration data changes (ExplorationPatch received), update the GPU textures
/// for both the main mist and ground fog materials.
pub fn update_mist_texture(
    mut world_cache: Option<ResMut<WorldCache>>,
    mut images: ResMut<Assets<Image>>,
    mist_materials: Res<Assets<MistMaterial>>,
    gf_materials: Res<Assets<GroundFogMaterial>>,
    mist_query: Query<&MeshMaterial2d<MistMaterial>, With<MistEntity>>,
    gf_query: Query<&MeshMaterial2d<GroundFogMaterial>, With<GroundFogEntity>>,
) {
    let Some(ref mut world_cache) = world_cache else { return; };

    let exploration = world_cache.exploration_cache_mut();
    if !exploration.dirty || !exploration.is_loaded() { return; }

    // Update main mist texture
    for mat_handle in mist_query.iter() {
        if let Some(material) = mist_materials.get(&mat_handle.0) {
            if let Some(image) = images.get_mut(&material.mist_texture) {
                image.data = Some(exploration.data.clone());
            }
        }
    }

    // Update ground fog texture
    for mat_handle in gf_query.iter() {
        if let Some(material) = gf_materials.get(&mat_handle.0) {
            if let Some(image) = images.get_mut(&material.mist_texture) {
                image.data = Some(exploration.data.clone());
            }
        }
    }

    exploration.dirty = false;
}

/// Push camera world position into the mist material each frame for parallax.
pub fn update_mist_camera(
    camera: Query<&Transform, With<MainCamera>>,
    mut mist_materials: ResMut<Assets<MistMaterial>>,
    mut gf_materials: ResMut<Assets<GroundFogMaterial>>,
    mist_query: Query<&MeshMaterial2d<MistMaterial>, With<MistEntity>>,
    gf_query: Query<&MeshMaterial2d<GroundFogMaterial>, With<GroundFogEntity>>,
) {
    let Ok(cam_tf) = camera.single() else { return; };
    let cam_pos = cam_tf.translation.truncate();

    for mat_handle in mist_query.iter() {
        if let Some(material) = mist_materials.get_mut(&mat_handle.0) {
            material.params.camera_x = cam_pos.x;
            material.params.camera_y = cam_pos.y;
        }
    }

    for mat_handle in gf_query.iter() {
        if let Some(material) = gf_materials.get_mut(&mat_handle.0) {
            material.params.camera_x = cam_pos.x;
            material.params.camera_y = cam_pos.y;
        }
    }
}

/// Spawn a low-lying ground fog layer below trees, above terrain.
/// Shares the exploration texture with the main mist.
pub fn spawn_ground_fog(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut gf_materials: ResMut<Assets<GroundFogMaterial>>,
    world_cache: Option<Res<WorldCache>>,
    existing: Query<Entity, With<GroundFogEntity>>,
) {
    let Some(world_cache) = world_cache else { return; };
    if !world_cache.is_exploration_loaded() { return; }
    if existing.iter().count() > 0 { return; }

    let exploration = world_cache.exploration_cache();
    let tex_width = exploration.width;
    let tex_height = exploration.height;
    if tex_width == 0 || tex_height == 0 { return; }

    let mut mist_image = Image::new(
        Extent3d {
            width: tex_width as u32,
            height: tex_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        exploration.data.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    mist_image.sampler = bevy::image::ImageSampler::linear();
    let mist_texture = images.add(mist_image);

    let world_width = exploration.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let world_height = exploration.n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    let mesh = create_mist_mesh(world_width, world_height);

    // Use real ocean SDF if available, otherwise fallback
    let sdf_texture = if let Some(ocean_data) = world_cache.get_ocean() {
        let mut sdf_image = Image::new(
            Extent3d {
                width: ocean_data.width as u32,
                height: ocean_data.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            ocean_data.sdf_values.clone(),
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        sdf_image.sampler = bevy::image::ImageSampler::linear();
        images.add(sdf_image)
    } else {
        create_fallback_sdf(&mut images)
    };

    let material = gf_materials.add(GroundFogMaterial {
        mist_texture,
        params: GroundFogParams {
            world_width,
            world_height,
            ..default()
        },
        sdf_texture,
    });

    info!("🌁 Spawning ground fog layer");

    let mut entity = commands.spawn((
        Name::new("GroundFog"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, -0.499995)),
        GroundFogEntity,
    ));

    if world_cache.get_ocean().is_some() {
        entity.insert(GroundFogSdfLoaded);
    }
}

// Ground fog texture update is handled by update_mist_texture (unified).