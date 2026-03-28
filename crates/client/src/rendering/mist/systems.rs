use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::networking::client::NetworkClient;
use crate::rendering::mist::materials::{MistMaterial, MistParams};
use crate::state::resources::WorldCache;
use shared::constants;

#[derive(Component)]
pub struct MistEntity;

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

pub fn request_exploration_map(
    mut cache: ResMut<WorldCache>,
    network_client_opt: Option<ResMut<NetworkClient>>,
) {
    let Some(mut network_client) = network_client_opt else { return; };

    if !cache.is_exploration_loaded() && !cache.is_exploration_requested() {
        info!("Requesting exploration map from server");
        network_client.send_message(shared::protocol::ClientMessage::RequestExplorationMap {
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
    let width = exploration.width;
    let height = exploration.height;

    if width == 0 || height == 0 { return; }

    // Create mist texture from exploration data (bilinear filtering for smooth edges)
    let mut mist_image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        exploration.data.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // Enable bilinear filtering for smooth transitions
    mist_image.sampler = bevy::image::ImageSampler::linear();

    let mist_texture = images.add(mist_image);

    let world_width = width as f32 * constants::CHUNK_SIZE.x;
    let world_height = height as f32 * constants::CHUNK_SIZE.y;

    let mesh = create_mist_mesh(world_width, world_height);

    let material = mist_materials.add(MistMaterial {
        mist_texture: mist_texture.clone(),
        params: MistParams {
            world_width,
            world_height,
            ..default()
        },
    });

    info!(
        "🌫️ Spawning mist mesh: {}x{} ({}x{} chunks)",
        world_width, world_height, width, height
    );

    commands.spawn((
        Name::new("Mist"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)), // Above everything
        MistEntity,
    ));
}

pub fn update_mist_texture(
    mut world_cache: Option<ResMut<WorldCache>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(ref mut world_cache) = world_cache else { return; };

    let exploration = world_cache.exploration_cache_mut();
    if !exploration.dirty || !exploration.is_loaded() { return; }

    if let Some(ref handle) = exploration.texture_handle {
        if let Some(image) = images.get_mut(handle) {
            image.data = Some(exploration.data.clone());
            exploration.dirty = false;
        }
    }
}