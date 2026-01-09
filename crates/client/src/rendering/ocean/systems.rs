use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::networking::client::NetworkClient;
use crate::rendering::ocean::materials::{OceanMaterial, OceanParams};
use crate::state::resources::WorldCache;
use shared::constants;

#[derive(Component)]
pub struct OceanEntity;

/// Crée un mesh 2D rectangulaire avec UVs correctement définis
fn create_ocean_mesh(width: f32, height: f32) -> Mesh {
    let _half_width = width / 2.0;
    let _half_height = height / 2.0;

    // Vertices du rectangle (4 coins)
    // Format: [x, y, z]
    let vertices = vec![
        [0.0, 0.0, 0.0],      // Bottom-left
        [width, 0.0, 0.0],    // Bottom-right
        [width, height, 0.0], // Top-right
        [0.0, height, 0.0],   // Top-left
    ];

    // UVs: (0,0) en bas à gauche, (1,1) en haut à droite
    let uvs = vec![
        [0.0, 0.0], // Bottom-left
        [1.0, 0.0], // Bottom-right
        [1.0, 1.0], // Top-right
        [0.0, 1.0], // Top-left
    ];

    // Normales (toutes pointent vers +Z)
    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    // Indices pour 2 triangles
    let indices = vec![
        0, 1, 2, // Premier triangle
        0, 2, 3, // Deuxième triangle
    ];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn request_ocean_data(
    mut cache: ResMut<WorldCache>,
    network_client_opt: Option<ResMut<NetworkClient>>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };

    // Request ocean data only once
    if !cache.is_ocean_loaded() && !cache.is_ocean_requested() {
        info!("Requesting ocean data from server");
        network_client.send_message(shared::protocol::ClientMessage::RequestOceanData {
            world_name: "Gaulyia".to_string(),
        });
        cache.mark_ocean_requested();
    }
}

pub fn spawn_ocean(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ocean_materials: ResMut<Assets<OceanMaterial>>,
    mut _materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    cache: Res<WorldCache>,
    ocean_query: Query<Entity, With<OceanEntity>>,
) {
    // Only spawn if ocean data is loaded and ocean doesn't exist yet
    if ocean_query.iter().count() > 0 {
        return;
    }

    let Some(ocean_data) = cache.get_ocean() else {
        return;
    };

    // Calculate world dimensions
    let world_width = ocean_data.width as f32 * (constants::CHUNK_SIZE.x / 64.0);
    let world_height = ocean_data.height as f32 * (constants::CHUNK_SIZE.y / 64.0);

    let mesh = create_ocean_mesh(world_width, world_height);

    // Create SDF texture from ocean data
    let sdf_image = Image::new(
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

    // Create heightmap texture from ocean data
    let heightmap_image = Image::new(
        Extent3d {
            width: ocean_data.width as u32,
            height: ocean_data.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        ocean_data.heightmap_values.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    let sdf_texture = images.add(sdf_image);
    let heightmap = images.add(heightmap_image);

    commands.spawn((
        OceanEntity,
        Name::new("Ocean"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(ocean_materials.add(OceanMaterial {
            heightmap,
            sdf_texture,
            params: OceanParams {
                world_width,
                world_height,
                max_depth: ocean_data.max_distance,
                ..default()
            },
            // shallow_color: LinearRgba::new(0.352, 0.415, 0.459, 1.0),
            // deep_color: LinearRgba::new(0.227, 0.29, 0.352, 1.0),
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, -500.0)),
    ));
}

pub fn update_ocean_time(time: Res<Time>, mut materials: ResMut<Assets<OceanMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.params.time = time.elapsed_secs();
    }
}
