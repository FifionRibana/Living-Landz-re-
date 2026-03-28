use std::collections::{HashMap, HashSet};

use bevy::mesh::Indices;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use hexx::Hex;
use rand::{Rng, SeedableRng};
use shared::atlas::{BuildingAtlas, TreeAtlas};
use shared::grid::GridConfig;
use shared::{
    AgricultureData, AnimalBreedingData, BiomeChunkData, BiomeTypeEnum, BuildingCategoryEnum,
    BuildingSpecific, BuildingSpecificTypeEnum, BuildingTypeEnum, CommerceData, CultData,
    EntertainmentData, ManufacturingWorkshopData, TerrainChunkData, TerrainChunkId,
    TerrainChunkSdfData, TreeAge, TreeTypeEnum, constants, get_biome_color,
};

use super::components::{Biome, Building, Terrain};
use super::materials::TerrainMaterial;
use crate::camera::MainCamera;
use crate::networking::client::NetworkClient;
use crate::rendering::terrain::components::TreeGlobalMesh;
use crate::rendering::terrain::materials::{
    BiomeParams, ChunkInfo, HeightmapParams, LakeParams, RoadParams, SdfParams, TreeMaterial,
};
use crate::state::resources::{ConnectionStatus, WorldCache};

pub fn initialize_terrain(
    connection: Res<ConnectionStatus>,
    network_client_opt: Option<ResMut<NetworkClient>>,
    world_cache_opt: Option<ResMut<WorldCache>>,
    // terrains: Query<&Terrain>,
) {
    let Some(_network_client) = network_client_opt else {
        return;
    };
    let Some(_world_cache) = world_cache_opt else {
        return;
    };

    if !connection.is_ready() {
        return;
    }
}

pub fn request_terrain_global_data(
    mut cache: ResMut<WorldCache>,
    network_client_opt: Option<ResMut<NetworkClient>>,
) {
    let Some(mut network_client) = network_client_opt else {
        return;
    };

    if !cache.is_terrain_global_loaded() && !cache.is_terrain_global_requested() {
        info!("Requesting terrain global data from server");
        network_client.send_message(shared::protocol::ClientMessage::RequestTerrainGlobalData {
            world_name: "Gaulyia".to_string(),
        });
        cache.mark_terrain_global_requested();
    }
}

pub fn create_terrain_global_textures(
    mut cache: ResMut<WorldCache>,
    mut images: ResMut<Assets<Image>>,
) {
    // Only create handles once, when data arrives
    if !cache.is_terrain_global_loaded() || cache.has_terrain_global_handles() {
        return;
    }

    let data = cache.get_terrain_global().unwrap();

    info!(
        "Creating global biome texture {}x{} and heightmap {}x{}",
        data.biome_width, data.biome_height, data.heightmap_width, data.heightmap_height
    );

    let mut biome_image = Image::new(
        Extent3d {
            width: data.biome_width,
            height: data.biome_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data.biome_values.clone(),
        TextureFormat::Rgba8Unorm,
        default(),
    );
    biome_image.sampler = bevy::image::ImageSampler::nearest();

    let heightmap_image = Image::new(
        Extent3d {
            width: data.heightmap_width,
            height: data.heightmap_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data.heightmap_values.clone(),
        TextureFormat::R8Unorm,
        default(),
    );

    let biome_handle = images.add(biome_image);
    let heightmap_handle = images.add(heightmap_image);

    cache.set_terrain_global_handles(biome_handle, heightmap_handle);
    info!("✓ Global terrain textures created");
}

pub fn spawn_terrain(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    terrains: Query<&Terrain>,
    camera: Query<&Transform, With<MainCamera>>,
) {
    let Some(world_cache) = world_cache_opt else {
        return;
    };

    let spawned_terrains: HashSet<_> = terrains
        .iter()
        .map(|t| TerrainChunkData::storage_key(t.name.as_str(), t.id))
        .collect();

    // Get camera position for priority sorting
    let camera_chunk = if let Ok(transform) = camera.single() {
        let pos = transform.translation.truncate();
        TerrainChunkId {
            x: (pos.x / constants::CHUNK_SIZE.x).round() as i32,
            y: (pos.y / constants::CHUNK_SIZE.y).round() as i32,
        }
    } else {
        TerrainChunkId { x: 0, y: 0 }
    };

    // Collect unspawned chunks and sort by distance to camera
    let mut to_spawn: Vec<_> = world_cache.loaded_terrains()
        .filter(|t| {
            !t.mesh_data.triangles.is_empty()
                && !spawned_terrains.contains(&t.get_storage_key())
        })
        .collect();

    to_spawn.sort_unstable_by_key(|t| {
        let dx = t.id.x - camera_chunk.x;
        let dy = t.id.y - camera_chunk.y;
        dx * dx + dy * dy
    });

    let mut spawned_this_frame = 0;
    let max_spawns_per_frame = 3;

    for terrain in to_spawn.into_iter().take(max_spawns_per_frame) {
        if spawned_terrains.contains(&terrain.get_storage_key()) {
            continue;
        }

        if terrain.mesh_data.triangles.is_empty() {
            continue;
        }

        if spawned_this_frame >= max_spawns_per_frame {
            break;
        }

        let terrain_name = &terrain.name;

        // Use data directly — no clone
        let mut all_triangles = terrain.mesh_data.triangles.clone(); // needed: extend_mesh_edges mutates
        let all_normals = terrain.mesh_data.normals.clone();
        let all_uvs = terrain.mesh_data.uvs.clone();

        extend_mesh_edges(&mut all_triangles, 600.0, 503.0, 1.0);

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, all_triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, all_normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs);

        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        let mesh_handle = meshes.add(mesh);

        // Get global textures (same as before, no changes)
        let (biome_texture, biome_params) = if let (Some(bh), Some(global)) = (
            world_cache.get_terrain_global_biome_handle(),
            world_cache.get_terrain_global(),
        ) {
            (
                bh.clone(),
                BiomeParams {
                    has_biome: 1.0,
                    world_width: global.world_width,
                    world_height: global.world_height,
                    ..default()
                },
            )
        } else {
            let dummy = images.add(Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![5 * 17, 5 * 17, 0, 255],
                TextureFormat::Rgba8Unorm,
                default(),
            ));
            (dummy, BiomeParams::default())
        };

        let (heightmap_texture, heightmap_params) =
            if let Some(hh) = world_cache.get_terrain_global_heightmap_handle() {
                (
                    hh.clone(),
                    HeightmapParams {
                        has_heightmap: 1.0,
                        ..default()
                    },
                )
            } else {
                let dummy = images.add(Image::new(
                    Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    vec![128u8],
                    TextureFormat::R8Unorm,
                    default(),
                ));
                (dummy, HeightmapParams::default())
            };

        let (lake_sdf_texture, lake_params) = if let Some(lh) = world_cache.get_lake_sdf_handle() {
            (
                lh.clone(),
                LakeParams {
                    has_lake: 1.0,
                    ..default()
                },
            )
        } else {
            let dummy = images.add(Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![255u8],
                TextureFormat::R8Unorm,
                default(),
            ));
            (dummy, LakeParams::default())
        };

        let material_handle = if let Some(sdf) = terrain.sdf_data.first() {
            let sdf_texture = create_sdf_texture_from_data(sdf, &mut images);

            let (road_texture, road_params) = if let Some(ref road_sdf) = terrain.road_sdf_data {
                let road_tex = create_road_sdf_texture(road_sdf, &mut images);
                (
                    road_tex,
                    RoadParams {
                        has_roads: 1.0,
                        ..default()
                    },
                )
            } else {
                let dummy = images.add(Image::new(
                    Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    vec![0u8],
                    TextureFormat::R8Unorm,
                    default(),
                ));
                (dummy, RoadParams::default())
            };

            MeshMaterial2d(terrain_materials.add(TerrainMaterial {
                sdf_texture,
                sdf_params: SdfParams {
                    beach_start: -0.1,
                    beach_end: 0.4,
                    has_coast: 1.0,
                    _padding: 0.0,
                },
                road_sdf_texture: road_texture,
                road_params,
                road_color_light: LinearRgba::new(0.76, 0.70, 0.55, 1.0),
                road_color_dark: LinearRgba::new(0.55, 0.48, 0.38, 1.0),
                road_color_tracks: LinearRgba::new(0.40, 0.35, 0.28, 1.0),
                chunk_info: ChunkInfo {
                    world_offset_x: world_position.x,
                    world_offset_y: world_position.y,
                    chunk_width: constants::CHUNK_SIZE.x,
                    chunk_height: constants::CHUNK_SIZE.y,
                },
                biome_texture: biome_texture.clone(),
                biome_params,
                heightmap_texture: heightmap_texture.clone(),
                heightmap_params,
                lake_sdf_texture: lake_sdf_texture.clone(),
                lake_params,
                ..default()
            }))
        } else {
            // No SDF — simple material (same structure, skip road/coast)
            let dummy_sdf = images.add(Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![128u8],
                TextureFormat::R8Unorm,
                default(),
            ));
            let dummy_road = images.add(Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0u8],
                TextureFormat::R8Unorm,
                default(),
            ));
            MeshMaterial2d(terrain_materials.add(TerrainMaterial {
                sdf_texture: dummy_sdf,
                road_sdf_texture: dummy_road,
                chunk_info: ChunkInfo {
                    world_offset_x: world_position.x,
                    world_offset_y: world_position.y,
                    chunk_width: constants::CHUNK_SIZE.x,
                    chunk_height: constants::CHUNK_SIZE.y,
                },
                biome_texture: biome_texture.clone(),
                biome_params,
                heightmap_texture: heightmap_texture.clone(),
                heightmap_params,
                lake_sdf_texture: lake_sdf_texture.clone(),
                lake_params,
                ..default()
            }))
        };

        commands.spawn((
            Name::new(format!("Terrain_{}_{}", terrain.id.x, terrain.id.y)),
            Mesh2d(mesh_handle),
            material_handle,
            Transform::from_translation(world_position.extend(-1000.)),
            Terrain {
                name: terrain_name.clone(),
                id: terrain.id,
            },
        ));

        spawned_this_frame += 1;
    }
}

pub fn spawn_building(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    buildings: Query<&Building>,
    images: Res<Assets<Image>>,
    tree_atlas: Res<TreeAtlas>,
    building_atlas: Res<BuildingAtlas>,
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

        // info!(
        //     "BUILDING A CATEGORY: {:?} on cell {:?}",
        //     building_base.category, building_base.cell
        // );

        match (&building_base.category, &building.specific_data) {
            (BuildingCategoryEnum::Natural, BuildingSpecific::Tree(_tree_data)) => {
                // Trees are rendered via instanced mesh merging in spawn_tree_meshes
                continue;
            }
            (
                BuildingCategoryEnum::ManufacturingWorkshops,
                BuildingSpecific::ManufacturingWorkshop(data),
            ) => {
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.workshop_type.to_building_type(),
                    0,
                    building_id,
                    world_position,
                    "Workshop",
                );
            }
            (BuildingCategoryEnum::Agriculture, BuildingSpecific::Agriculture(data)) => {
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.agriculture_type.to_building_type(),
                    0,
                    building_id,
                    world_position,
                    "Farm",
                );
            }
            (BuildingCategoryEnum::AnimalBreeding, BuildingSpecific::AnimalBreeding(data)) => {
                let variant = 0; // Use first variant, could be randomized
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.animal_type.to_building_type(),
                    variant,
                    building_id,
                    world_position,
                    "AnimalBreeding",
                );
            }
            (BuildingCategoryEnum::Entertainment, BuildingSpecific::Entertainment(data)) => {
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.entertainment_type.to_building_type(),
                    0,
                    building_id,
                    world_position,
                    "Entertainment",
                );
            }
            (BuildingCategoryEnum::Cult, BuildingSpecific::Cult(data)) => {
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.cult_type.to_building_type(),
                    0,
                    building_id,
                    world_position,
                    "Cult",
                );
            }
            (BuildingCategoryEnum::Commerce, BuildingSpecific::Commerce(data)) => {
                spawn_building_sprite(
                    &mut commands,
                    &building_atlas,
                    &images,
                    data.commerce_type.to_building_type(),
                    0,
                    building_id,
                    world_position,
                    "Commerce",
                );
            }
            _ => {
                // Fallback pour les types inconnus
                info!(
                    "  Unknown building from category {:?} on cell: {:?}",
                    building_base.category, building_base.cell
                );
                let color = Color::srgba(0.5, 0.5, 0.5, 1.0);
                let size = Vec2::new(32.0, 32.0);

                commands.spawn((
                    Name::new(format!("Building_{}", building_id)),
                    Sprite {
                        color,
                        custom_size: Some(size),
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
        };
    }
}

fn spawn_building_sprite(
    commands: &mut Commands,
    building_atlas: &BuildingAtlas,
    images: &Assets<Image>,
    building_type: BuildingTypeEnum,
    variant: usize,
    building_id: u64,
    world_position: Vec2,
    category_name: &str,
) {
    if building_type == BuildingTypeEnum::Market {
        info!("BUILDING TYPE {:?} SPAWN REQUEST", building_type);
    }
    if let Some(image_handle) = building_atlas.get_sprite(building_type, variant) {
        let image_size = images.get(image_handle).map(|img| {
            let size = img.texture_descriptor.size;
            Vec2::new(size.width as f32, size.height as f32)
        });

        let custom_size = image_size.map(|size| Vec2::new(48.0, 48.0 * (size.y / size.x)));

        let position = Vec2::new(world_position.x, world_position.y + 8.);

        commands.spawn((
            Name::new(format!("{}_{}", category_name, building_id)),
            Sprite {
                image: image_handle.clone(),
                custom_size,
                ..default()
            },
            Transform::from_translation(position.extend(-world_position.y * 0.0001)),
            GlobalTransform::default(),
            Visibility::default(),
            Building {
                id: building_id as i64,
            },
        ));
    } else {
        // Fallback: colored square if sprite not found
        warn!(
            "Sprite '{:?}' variant {} not found in building atlas",
            building_type, variant
        );
        let color = Color::srgba(0.6, 0.4, 0.2, 1.0);
        let size = Vec2::new(32.0, 32.0);

        commands.spawn((
            Name::new(format!("{}_{}", category_name, building_id)),
            Sprite {
                color,
                custom_size: Some(size),
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
}

// Converts sdf data into a bevy Image texture
pub fn create_sdf_texture_from_data(
    sdf_data: &TerrainChunkSdfData,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    let res = sdf_data.resolution as u32;

    let image = Image::new(
        Extent3d {
            width: res,
            height: res,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        sdf_data.values.clone(),
        TextureFormat::R8Unorm,
        default(),
    );

    images.add(image)
}

/// Creates an RG16 texture from road SDF data
/// R channel: distance field (0-65535)
/// G channel: metadata (importance, tracks, intersection flags)
pub fn create_road_sdf_texture(
    road_sdf_data: &shared::RoadChunkSdfData,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    let width = road_sdf_data.resolution_x as u32;
    let height = road_sdf_data.resolution_y as u32;

    // Debug: Analyze road SDF data
    if road_sdf_data.data.len() >= 4 {
        let total_pixels = (width as usize) * (height as usize);
        let mut min_dist = u16::MAX;
        let mut max_dist = u16::MIN;
        let mut non_max_count = 0;

        // Sample every 4 bytes (RG16 = 2 bytes R + 2 bytes G)
        for i in 0..total_pixels {
            let offset = i * 4;
            if offset + 1 < road_sdf_data.data.len() {
                // Read R channel (distance) as little-endian u16
                let dist = u16::from_le_bytes([
                    road_sdf_data.data[offset],
                    road_sdf_data.data[offset + 1],
                ]);
                min_dist = min_dist.min(dist);
                max_dist = max_dist.max(dist);
                if dist < u16::MAX {
                    non_max_count += 1;
                }
            }
        }

        info!(
            "Road SDF texture: {}x{}, min_dist: {}, max_dist: {}, non_max_pixels: {}/{}, data_len: {}",
            width,
            height,
            min_dist,
            max_dist,
            non_max_count,
            total_pixels,
            road_sdf_data.data.len()
        );
    }

    // Data is already in RG16 format (4 bytes per pixel)
    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        road_sdf_data.data.clone(),
        TextureFormat::Rg16Unorm,
        default(),
    );

    images.add(image)
}

// Setup du material par défaut au démarrage
pub fn setup_default_terrain_material(
    _commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Texture 1x1 blanche (dummy pour les chunks sans côte)
    let dummy_texture = images.add(Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255u8],
        TextureFormat::R8Unorm,
        default(),
    ));

    let _default_material = materials.add(TerrainMaterial {
        sdf_texture: dummy_texture,
        sdf_params: SdfParams {
            beach_start: 0.,
            beach_end: 0.2,
            has_coast: 0.0,
            _padding: 0.0,
        },
        ..default()
    });

    // commands.insert_resource(DefaultTerrainMaterial(default_material));
}

fn extend_mesh_edges(positions: &mut [[f32; 3]], image_width: f32, image_height: f32, extend: f32) {
    let edge_threshold = 1.0;

    for pos in positions.iter_mut() {
        // Bord gauche
        if pos[0] < edge_threshold {
            pos[0] -= extend;
        }
        // Bord droit
        if pos[0] > image_width - edge_threshold {
            pos[0] += extend;
        }
        // Bord bas
        if pos[1] < edge_threshold {
            pos[1] -= extend;
        }
        // Bord haut
        if pos[1] > image_height - edge_threshold {
            pos[1] += extend;
        }
    }
}

fn generate_uvs_from_positions(positions: &[[f32; 3]], chunk_world_size: Vec2) -> Vec<[f32; 2]> {
    // Utiliser les dimensions fixes de l'image, pas les bounds de la mesh
    let image_width = chunk_world_size.x; //600.0;
    let image_height = chunk_world_size.y; //503.0;

    positions
        .iter()
        .map(|pos| {
            [
                pos[0] / image_width,
                (pos[1] / image_height), // Y inversé
            ]
        })
        .collect()
}

fn filter_coastal_segments(contour: &[Vec2], threshold: f32, max_x: f32, max_y: f32) -> Vec<Vec2> {
    if contour.len() < 3 {
        return vec![];
    }

    let mut result: Vec<Vec2> = Vec::new();
    let n = contour.len();

    for i in 0..n {
        let a = contour[i];
        let b = contour[(i + 1) % n];

        // Ignorer les segments alignés sur les bords du chunk
        if is_segment_on_chunk_edge(a, b, threshold, max_x, max_y) {
            continue;
        }

        // Ajouter le point a s'il n'est pas déjà présent
        if result.is_empty() || (*result.last().unwrap() - a).length() > 0.01 {
            result.push(a);
        }
    }

    // Supprimer le dernier point s'il est identique au premier (contour fermé)
    if result.len() > 1 && (*result.first().unwrap() - *result.last().unwrap()).length() < 0.01 {
        result.pop();
    }

    result
}

/// Vérifie si un segment est aligné sur un bord du chunk (côte artificielle à ignorer)
fn is_segment_on_chunk_edge(a: Vec2, b: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
    // Tolérance pour la position sur le bord
    let pos_threshold = threshold;
    // Tolérance pour l'alignement (segment quasi horizontal ou vertical)
    let align_threshold = threshold;

    // Bord gauche : les deux points près de x=0 ET segment quasi vertical
    let on_left = a.x < pos_threshold && b.x < pos_threshold && (a.x - b.x).abs() < align_threshold;

    // Bord droit : les deux points près de x=max ET segment quasi vertical
    let on_right = a.x > max_x && b.x > max_x && (a.x - b.x).abs() < align_threshold;

    // Bord bas : les deux points près de y=0 ET segment quasi horizontal
    let on_bottom =
        a.y < pos_threshold && b.y < pos_threshold && (a.y - b.y).abs() < align_threshold;

    // Bord haut : les deux points près de y=max ET segment quasi horizontal
    let on_top = a.y > max_y && b.y > max_y && (a.y - b.y).abs() < align_threshold;

    on_left || on_right || on_bottom || on_top
}

/// Pack individual tree sprites into a single texture atlas
pub fn build_tree_atlas(
    mut tree_atlas: ResMut<TreeAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut built: Local<bool>,
) {
    if *built || tree_atlas.atlas_image.is_some() {
        return;
    }

    let variant_names: Vec<String> = tree_atlas
        .sprites
        .values()
        .flat_map(|v| v.iter().cloned())
        .collect();

    if variant_names.is_empty() {
        return;
    }

    let all_loaded = variant_names.iter().all(|name| {
        tree_atlas
            .handles
            .get(name.as_str())
            .and_then(|h| images.get(h))
            .is_some()
    });

    if !all_loaded {
        return;
    }

    let sprite_size = 256u32;
    let cols = 5u32;
    let rows = (variant_names.len() as u32 + cols - 1) / cols;
    let atlas_width = cols * sprite_size;
    let atlas_height = rows * sprite_size;

    info!(
        "Building tree atlas: {}x{} ({} variants)",
        atlas_width,
        atlas_height,
        variant_names.len()
    );

    let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    for (idx, name) in variant_names.iter().enumerate() {
        let handle = tree_atlas.handles.get(name.as_str()).unwrap();
        let image = images.get(handle).unwrap();
        let src_data = image.data.as_ref().unwrap();
        let src_w = image.texture_descriptor.size.width;
        let src_h = image.texture_descriptor.size.height;

        let col = (idx as u32) % cols;
        let row = (idx as u32) / cols;
        let dst_x = col * sprite_size;
        let dst_y = row * sprite_size;

        for y in 0..src_h.min(sprite_size) {
            for x in 0..src_w.min(sprite_size) {
                let src_idx = ((y * src_w + x) * 4) as usize;
                let dst_idx = (((dst_y + y) * atlas_width + dst_x + x) * 4) as usize;
                if src_idx + 3 < src_data.len() && dst_idx + 3 < atlas_data.len() {
                    atlas_data[dst_idx] = src_data[src_idx];
                    atlas_data[dst_idx + 1] = src_data[src_idx + 1];
                    atlas_data[dst_idx + 2] = src_data[src_idx + 2];
                    atlas_data[dst_idx + 3] = src_data[src_idx + 3];
                }
            }
        }

        tree_atlas.variants.insert(name.clone(), idx);
    }

    let atlas_image = Image::new(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let layout =
        TextureAtlasLayout::from_grid(UVec2::new(sprite_size, sprite_size), cols, rows, None, None);

    tree_atlas.sprite_size = sprite_size;
    tree_atlas.atlas_cols = cols;
    tree_atlas.atlas_rows = rows;
    tree_atlas.atlas_image = Some(images.add(atlas_image));
    tree_atlas.atlas_layout = Some(texture_atlas_layouts.add(layout));

    info!(
        "✓ Tree atlas built: {}x{} ({} variants)",
        atlas_width,
        atlas_height,
        variant_names.len()
    );
    *built = true;
}

/// Single merged mesh for ALL visible trees. Rebuilt when building cache changes.
pub fn rebuild_tree_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tree_materials: ResMut<Assets<TreeMaterial>>,
    mut world_cache: Option<ResMut<WorldCache>>,
    tree_atlas: Res<TreeAtlas>,
    grid_config: Res<GridConfig>,
    existing: Query<Entity, With<TreeGlobalMesh>>,
    mut shared_material: Local<Option<MeshMaterial2d<TreeMaterial>>>,
    time: Res<Time>,
    mut pending_rebuild: Local<f64>,
) {
    let Some(ref mut world_cache) = world_cache else {
        return;
    };
    let Some(atlas_image) = &tree_atlas.atlas_image else {
        return;
    };

    // Track dirty → set rebuild timer
    if world_cache.is_buildings_dirty() {
        *pending_rebuild = time.elapsed_secs_f64();
        world_cache.clear_buildings_dirty();
    }

    // Nothing pending
    if *pending_rebuild == 0.0 && existing.iter().count() > 0 {
        return;
    }

    // Still waiting for debounce (0.5s after last dirty)
    if *pending_rebuild > 0.0 && time.elapsed_secs_f64() - *pending_rebuild < 0.5 {
        return;
    }

    // First load: no existing mesh and no pending → need initial dirty
    if *pending_rebuild == 0.0 && existing.iter().count() == 0 {
        return;
    }

    *pending_rebuild = 0.0;

    // Despawn old
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    let material = shared_material
        .get_or_insert_with(|| {
            MeshMaterial2d(tree_materials.add(TreeMaterial {
                texture: atlas_image.clone(),
            }))
        })
        .clone();

    // Collect ALL tree quads
    let mut quads: Vec<TreeQuad> = Vec::with_capacity(100000);

    for building in world_cache.loaded_buildings() {
        let shared::BuildingSpecific::Tree(tree_data) = &building.specific_data else {
            continue;
        };

        let age = shared::TreeAge::get_tree_age(tree_data.age as u32);
        let age_idx = age.to_index();
        let variation = tree_data.variant;
        let building_id = building.base_data.id;

        // Fast numeric lookup — no format!, no HashMap<String>
        let Some(uvs) = tree_atlas
            .get_atlas_index_fast(age_idx, variation)
            .and_then(|idx| tree_atlas.get_atlas_uvs_by_index(idx))
        else {
            continue;
        };

        let cell_pos = grid_config.layout.hex_to_world_pos(hexx::Hex::new(
            building.base_data.cell.q,
            building.base_data.cell.r,
        ));

        let tree_count = match tree_data.density {
            d if d < 0.35 => 1,
            d if d < 0.50 => 2,
            d if d < 0.60 => 3,
            d if d < 0.70 => 4,
            d if d < 0.80 => 5,
            d if d < 0.90 => 6,
            d if d < 0.95 => 7,
            _ => 8,
        };

        for tree_i in 0..tree_count {
            let sub_seed = building_id.wrapping_mul(31).wrapping_add(tree_i as u64);
            let mut sub_rng = rand::rngs::StdRng::seed_from_u64(sub_seed);

            let offset_x: f32 = sub_rng.random_range(-18.0..=18.0);
            let offset_y: f32 = sub_rng.random_range(-18.0..=18.0);
            let scale_var: f32 = sub_rng.random_range(0.85..=1.15);
            let flip_x: bool = sub_rng.random_bool(0.5);

            // Sub-tree variant — fast numeric path
            let sub_uvs = if tree_i > 0 {
                let alt_var = sub_rng.random_range(1..=3i32);
                let alt_age_raw =
                    (tree_data.age as i32 + sub_rng.random_range(-30..=30)).max(0) as u32;
                let alt_age_idx = shared::TreeAge::get_tree_age(alt_age_raw).to_index();
                tree_atlas
                    .get_atlas_index_fast(alt_age_idx, alt_var)
                    .and_then(|idx| tree_atlas.get_atlas_uvs_by_index(idx))
                    .unwrap_or(uvs)
            } else {
                uvs
            };

            let pos = Vec2::new(cell_pos.x + offset_x, cell_pos.y + offset_y + 8.0);

            quads.push(TreeQuad {
                pos,
                size: 48.0 * scale_var,
                uvs: sub_uvs,
                flip_x,
            });
        }
    }

    if quads.is_empty() {
        return;
    }

    // Global sort back-to-front: higher Y drawn first
    quads.sort_unstable_by_key(|q| (-q.pos.y * 100.0) as i32);

    // Build single mesh
    let num_quads = quads.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_quads * 4);
    let mut uvs_out: Vec<[f32; 2]> = Vec::with_capacity(num_quads * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(num_quads * 6);

    for (qi, quad) in quads.iter().enumerate() {
        let half = quad.size / 2.0;
        let base_idx = (qi * 4) as u32;

        positions.push([quad.pos.x - half, quad.pos.y - half, 0.0]);
        positions.push([quad.pos.x + half, quad.pos.y - half, 0.0]);
        positions.push([quad.pos.x + half, quad.pos.y + half, 0.0]);
        positions.push([quad.pos.x - half, quad.pos.y + half, 0.0]);

        let [u_min, v_min, u_max, v_max] = quad.uvs;
        let (ul, ur) = if quad.flip_x {
            (u_max, u_min)
        } else {
            (u_min, u_max)
        };

        uvs_out.push([ul, v_max]);
        uvs_out.push([ur, v_max]);
        uvs_out.push([ur, v_min]);
        uvs_out.push([ul, v_min]);

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs_out)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices));

    info!(
        "✓ Tree mesh rebuilt: {} quads {} vertices",
        quads.len(),
        quads.len() * 4
    );

    commands.spawn((
        Name::new("TreeGlobalMesh"),
        Mesh2d(meshes.add(mesh)),
        material,
        Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
        TreeGlobalMesh,
    ));
}

struct TreeQuad {
    pos: Vec2,
    size: f32,
    uvs: [f32; 4], // u_min, v_min, u_max, v_max
    flip_x: bool,
}
