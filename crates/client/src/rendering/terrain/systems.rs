use std::collections::HashSet;

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use hexx::Hex;
use rand::{Rng, SeedableRng};
use shared::atlas::TreeAtlas;
use shared::grid::GridConfig;
use shared::{
    BiomeChunkData, BiomeType, BuildingCategory, BuildingSpecific, TerrainChunkData,
    TerrainChunkId, TreeAge, TreeType, constants, get_biome_color,
};

use super::components::{Biome, Building, Terrain};
use crate::networking::client::NetworkClient;
use crate::state::resources::{ConnectionStatus, WorldCache};

pub fn initialize_terrain(
    connection: Res<ConnectionStatus>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    mut world_cache_opt: Option<ResMut<WorldCache>>,
    terrains: Query<&Terrain>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };
    let Some(mut world_cache) = world_cache_opt else {
        return;
    };

    if !connection.is_ready() {
        return;
    }
}

pub fn spawn_terrain(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    terrains: Query<&Terrain>,
    biomes: Query<&Biome>,
) {
    let Some(world_cache) = world_cache_opt else {
        return;
    };

    let spawned_terrains: HashSet<_> = terrains
        .iter()
        .map(|t| TerrainChunkData::storage_key(t.name.as_str(), t.id))
        .collect();
    let spawned_biomes: HashSet<_> = biomes
        .iter()
        .map(|b| BiomeChunkData::storage_key(b.name.as_str(), b.id))
        .collect();

    for terrain in world_cache.loaded_terrains() {
        let terrain_name = terrain.clone().name;
        if spawned_terrains.contains(&terrain.get_storage_key()) {
            continue;
        }

        info!(
            "Spawning {} triangles for chunk ({},{}).",
            terrain.mesh_data.triangles.len(),
            terrain.id.x,
            terrain.id.y
        );

        let mesh_data = terrain.mesh_data.clone();

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        commands.spawn((
            Name::new(format!("Terrain_{}", terrain_name)),
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.4, 0.6, 0.3)))),
            Transform::from_translation(world_position.extend(-1000.1)),
            Terrain {
                name: terrain_name,
                id: terrain.id,
            },
        ));
    }

    for biome in world_cache.loaded_biomes() {
        let biome_name = biome.clone().name;
        // if biome.id.biome == BiomeType::Ocean || biome.id.biome == BiomeType::DeepOcean {
        //     continue;
        // }
        // if biome.id.biome != BiomeType::Savanna {
        //     continue;
        // }
        if spawned_biomes.contains(&biome.get_storage_key()) {
            continue;
        }

        info!(
            "Spawning {} triangles for biome {:?} chunk ({},{}).",
            biome.mesh_data.triangles.len(),
            biome.id.biome,
            biome.id.x,
            biome.id.y
        );

        let mesh_data = biome.mesh_data.clone();

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let world_position = Vec2::new(
            biome.id.x as f32 * constants::CHUNK_SIZE.x,
            biome.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        // Create an atlas instead of using a new one every time
        let color = *get_biome_color(&biome.id.biome).as_color();

        commands.spawn((
            Name::new(format!("Biome_{}", biome_name)),
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(world_position.extend(-1000.0)),
            Biome {
                name: biome_name,
                id: biome.id,
            },
        ));
    }
}

pub fn spawn_building(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buildings: Query<&Building>,
    images: Res<Assets<Image>>,
    tree_atlas: Res<TreeAtlas>,
    grid_config: Res<GridConfig>,
) {
    let Some(world_cache) = world_cache_opt else {
        return;
    };

    let spawned_buildings: HashSet<_> = buildings.iter().map(|b| b.id).collect();

    for building in world_cache.loaded_buildings() {
        let building_base = &building.base_data;

        let building_id = building_base.id;
        if spawned_buildings.contains(&(building_id as i64)) {
            continue;
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(building_id);

        let mut world_position = grid_config
            .layout
            .hex_to_world_pos(Hex::new(building_base.cell.q, building_base.cell.r));

        match (
            &building_base.building_type.category,
            &building.specific_data,
        ) {
            (BuildingCategory::Natural, BuildingSpecific::Tree(tree_data)) => {
                let tree_type = TreeType::Cedar; // TODO: create assets for oak and larch
                // let tree_type = tree_data.tree_type;
                let age = TreeAge::get_tree_age(tree_data.age as u32);
                let variation = tree_data.variant;
                let density = match tree_data.density {
                    (..0.45) => 1,
                    (0.45..0.55) => 2,
                    (0.55..0.65) => 3,
                    (0.65..0.75) => 4,
                    (0.75..0.85) => 5,
                    (0.85..) => 6,
                    _ => 6,
                };

                let variant = &format!(
                    "{}_{}_{:02}{:02}",
                    tree_type.to_name(),
                    age.to_name(),
                    variation,
                    density
                );
                let image_handle = tree_atlas
                    .handles
                    .get(variant)
                    .expect(format!("Tree variation not found {}", variant).as_str());

                let image_size = images.get(&*image_handle).map(|img| {
                    let size = img.texture_descriptor.size;
                    Vec2::new(size.width as f32, size.height as f32)
                });

                let scale_var = rng.random_range(0.9..=1.1);
                let flip_x = rng.random_bool(0.5);

                let offset_x: f32 = rng.random_range(-8.0..=8.0);
                let offset_y: f32 = rng.random_range(-8.0..=8.0);

                world_position.x += offset_x;
                world_position.y += offset_y + 8.0; // shift slightly up

                let custom_size = image_size.map(|size| {
                    let width = size.x.min(256.0f32) * scale_var * 64. / 256.; // TODO: assets shall be already downsized
                    let height = width * (size.y / size.x) * scale_var; // Aspect ratio conservé
                    Vec2::new(width, height)
                });

                commands.spawn((
                    Name::new(format!("{}_{}", &variant, building_id)),
                    Sprite {
                        image: image_handle.clone(),
                        custom_size,
                        flip_x,
                        ..default()
                    },
                    Transform::from_translation(world_position.extend(-world_position.y * 0.0001)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    Building {
                        id: building_id as i64,
                    },
                ));
            }
            _ => {}
        };
    }
}
