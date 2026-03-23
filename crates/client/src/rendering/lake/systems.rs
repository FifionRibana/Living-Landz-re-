use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::MeshMaterial2d;

use crate::networking::client::NetworkClient;
use crate::rendering::lake::materials::{LakeMaterial, LakeParams};
use crate::state::resources::WorldCache;

#[derive(Component)]
pub struct LakeEntity;

pub fn request_lake_data(
    mut cache: ResMut<WorldCache>,
    network_client_opt: Option<ResMut<NetworkClient>>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };

    // Request lake data only once
    if cache.is_lake_loaded() || cache.is_lake_requested() {
        return;
    }

    info!("Requesting lake data from server");
    network_client.send_message(shared::protocol::ClientMessage::RequestLakeData {
        world_name: "Gaulyia".to_string(),
    });
    cache.mark_lake_requested();
}

pub fn spawn_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lake_materials: ResMut<Assets<LakeMaterial>>,
    cache: Res<WorldCache>,
    lake_query: Query<Entity, With<LakeEntity>>,
) {
    if lake_query.iter().count() > 0 {
        return;
    }

    let Some(lake_data) = cache.get_lake() else {
        return;
    };

    let (Some(mask_handle), Some(sdf_handle)) = (
        cache.get_lake_mask_handle(),
        cache.get_lake_sdf_handle(),
    ) else {
        return;
    };

    let world_width = lake_data.world_width;
    let world_height = lake_data.world_height;

    info!(
        "🏞️ Spawning lake mesh: {}x{} (mask {}x{}, SDF {}x{})",
        world_width, world_height,
        lake_data.width, lake_data.height,
        lake_data.sdf_width, lake_data.sdf_height
    );

    let mesh = create_lake_mesh(world_width, world_height);

    commands.spawn((
        LakeEntity,
        Name::new("Lake"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(lake_materials.add(LakeMaterial {
            mask_texture: mask_handle.clone(),
            sdf_texture: sdf_handle.clone(),
            params: LakeParams {
                world_width,
                world_height,
                ..default()
            },
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));
}

pub fn create_lake_textures(
    mut cache: ResMut<WorldCache>,
    mut images: ResMut<Assets<Image>>,
) {
    // Skip if already created or no data
    if cache.has_lake_mask_handle() {
        return;
    }

    let Some(lake_data) = cache.get_lake() else {
        return;
    };

    if lake_data.sdf_values.is_empty() {
        return;
    }

    info!(
        "Creating lake textures: mask {}x{}, SDF {}x{}",
        lake_data.width, lake_data.height, lake_data.sdf_width, lake_data.sdf_height
    );

    // Mask texture
    let mut mask_image = Image::new(
        Extent3d {
            width: lake_data.width as u32,
            height: lake_data.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        lake_data.mask_values.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    mask_image.sampler = bevy::image::ImageSampler::linear();

    // SDF texture
    let mut sdf_image = Image::new(
        Extent3d {
            width: lake_data.sdf_width as u32,
            height: lake_data.sdf_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        lake_data.sdf_values.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    sdf_image.sampler = bevy::image::ImageSampler::linear();

    let mask_handle = images.add(mask_image);
    let sdf_handle = images.add(sdf_image);

    cache.set_lake_mask_handle(mask_handle);
    cache.set_lake_sdf_handle(sdf_handle);
    info!("✓ Lake textures created");
}

pub fn update_lake_time(time: Res<Time>, mut materials: ResMut<Assets<LakeMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.params.time = time.elapsed_secs();
    }
}

fn create_lake_mesh(width: f32, height: f32) -> Mesh {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [width, 0.0, 0.0],
        [width, height, 0.0],
        [0.0, height, 0.0],
    ];

    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let indices = vec![0u32, 1, 2, 0, 2, 3];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn create_lake_mask_texture(
    mut cache: ResMut<WorldCache>,
    mut images: ResMut<Assets<Image>>,
) {
    if cache.has_lake_mask_handle() {
        return;
    }

    let Some(lake_data) = cache.get_lake() else {
        return;
    };

    info!(
        "Creating lake mask texture {}x{}",
        lake_data.width, lake_data.height
    );

    let mut mask_image = Image::new(
        Extent3d {
            width: lake_data.width as u32,
            height: lake_data.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        lake_data.mask_values.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    mask_image.sampler = bevy::image::ImageSampler::linear();

    let handle = images.add(mask_image);
    cache.set_lake_mask_handle(handle);
    info!("✓ Lake mask texture created");
}