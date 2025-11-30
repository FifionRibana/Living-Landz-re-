use std::collections::HashSet;

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
use crate::networking::client::NetworkClient;
use crate::rendering::terrain::materials::SdfParams;
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
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    // default_terrain_material: Res<DefaultTerrainMaterial>,
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
    // let spawned_biomes: HashSet<_> = biomes
    //     .iter()
    //     .map(|b| BiomeChunkData::storage_key(b.name.as_str(), b.id))
    //     .collect();

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
        let mesh_data_ref = &mesh_data;

        // Debug des positions de la mesh
        let pos_min = mesh_data
            .triangles
            .clone()
            .iter()
            .fold([f32::MAX, f32::MAX, f32::MAX], |acc, p| {
                [acc[0].min(p[0]), acc[1].min(p[1]), acc[2].min(p[2])]
            });
        let pos_max = mesh_data
            .triangles
            .clone()
            .iter()
            .fold([f32::MIN, f32::MIN, f32::MIN], |acc, p| {
                [acc[0].max(p[0]), acc[1].max(p[1]), acc[2].max(p[2])]
            });

        info!("Mesh positions - min: {:?}, max: {:?}", pos_min, pos_max);

        // TODO : vérifier les liaisons entre chunk
        // TODO : étendre le sdf à l'extérieur du contour plutôt que s'arrêter net
        // TODO : sdf pour les biomes ou via shader pour blend avec terrain

        // Générer les UV avant d'étendre les bords
        let mut all_triangles = mesh_data_ref.triangles.clone();
        let all_normals = mesh_data_ref.normals.clone();
        let all_uvs = mesh_data_ref.uvs.clone();
        // generate_uvs_from_positions(&mesh_data_ref.triangles.clone(), constants::CHUNK_SIZE);

        // Vertex colors : terrain principal = opaque
        let all_colors: Vec<[f32; 4]> = vec![[1.0, 1.0, 1.0, 1.0]; all_triangles.len()];

        let uvs =
            generate_uvs_from_positions(&mesh_data_ref.triangles.clone(), constants::CHUNK_SIZE);

        // let mut triangles = mesh_data_ref.triangles.clone();

        // Étendre les bords de 1 pixel pour éviter les coutures
        extend_mesh_edges(&mut all_triangles, 600.0, 503.0, 1.0);

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, all_triangles)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, all_normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs.clone());
        // .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, all_colors);

        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        let mesh_handle = meshes.add(mesh);

        let material_handle = if let Some(sdf) = terrain.sdf_data.first() {
            let sdf_texture = create_sdf_texture_from_data(sdf, &mut images);

            info!("Creating material WITH SDF for chunk {:?}", terrain.id);
            MeshMaterial2d(terrain_materials.add(TerrainMaterial {
                sdf_texture,
                sdf_params: SdfParams {
                    beach_start: -0.15,
                    beach_end: 0.6,
                    opacity_start: -0.15,
                    opacity_end: 0.0,
                }, // has_coast = 1.0
                ..default()
            }))
        } else {
            info!("Creating material WITHOUT SDF for chunk {:?}", terrain.id);
            let dummy_sdf = images.add(Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![255u8], // Tout terre
                TextureFormat::R8Unorm,
                default(),
            ));

            MeshMaterial2d(terrain_materials.add(TerrainMaterial {
                sdf_texture: dummy_sdf,
                sand_color: LinearRgba::new(0.76, 0.70, 0.50, 1.0),
                grass_color: LinearRgba::new(0.36, 0.52, 0.28, 1.0),
                sdf_params: SdfParams {
                    beach_start: 0.0,
                    beach_end: 0.0,
                    opacity_start: -1.0,
                    opacity_end: -1.0,
                },
            }))
        };

        // Debug des données SDF
        if let Some(sdf) = terrain.sdf_data.first() {
            let min_val = sdf.values.iter().min().unwrap_or(&255);
            let max_val = sdf.values.iter().max().unwrap_or(&0);
            let zero_count = sdf.values.iter().filter(|&&v| v < 50).count();

            info!(
                "SDF debug - resolution: {}, min: {}, max: {}, values near 0: {}, total: {}",
                sdf.resolution,
                min_val,
                max_val,
                zero_count,
                sdf.values.len()
            );
        }

        if let Some(sdf) = terrain.sdf_data.first() {
            let min_val = sdf.values.iter().min().unwrap_or(&0);
            let max_val = sdf.values.iter().max().unwrap_or(&0);
            let mid_count = sdf.values.iter().filter(|&&v| v > 100 && v < 156).count();
            let land_count = sdf.values.iter().filter(|&&v| v > 128).count();
            let water_count = sdf.values.iter().filter(|&&v| v < 128).count();

            info!(
                "Chunk {:?} SDF - min: {}, max: {}, land: {}, water: {}, near_edge: {}, total: {}",
                terrain.id,
                min_val,
                max_val,
                land_count,
                water_count,
                mid_count,
                sdf.values.len()
            );
        }

        // Debug des UV
        let uv_min = all_uvs
            .clone()
            .iter()
            .fold(Vec2::MAX, |acc, &uv| acc.min(Vec2::new(uv[0], uv[1])));
        let uv_max = all_uvs
            .clone()
            .iter()
            .fold(Vec2::MIN, |acc, &uv| acc.max(Vec2::new(uv[0], uv[1])));
        info!("UV range: min {:?}, max {:?}", uv_min, uv_max);

        commands.spawn((
            Name::new(format!("Terrain_{}", terrain_name.clone())),
            Mesh2d(mesh_handle),
            material_handle,
            // MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.4, 0.6, 0.3)))),
            Transform::from_translation(world_position.extend(-1000.)),
            Terrain {
                name: terrain_name.clone(),
                id: terrain.id,
            },
        ));

        // if !terrain.sdf_data.is_empty() {
        //     info!(
        //         "    with {} SDF data layers.",
        //         terrain.sdf_data.len()
        //     );
        //     for sdf in terrain.sdf_data.iter() {
        //         info!(
        //             "    sdf points: {}", sdf.values.len()
        //         );
        //     };
        //     for sdf in terrain.sdf_data.iter() {
        //         let sdf_texture = create_sdf_texture_from_data(sdf, &mut images);

        //         let material_handle  =terrain_materials.add(TerrainMaterial {
        //             sdf_texture,
        //             ..default()
        //         });

        //         commands.spawn((
        //             Name::new(format!("TerrainSDF_{}_{}", terrain_name.clone(), sdf.resolution)),
        //             MeshMaterial3d(material_handle.clone()),
        //             // Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        //             // Transform::from_translation(world_position.extend(-100.)),
        //         ));
        //     }
        // }
    }

    // for biome in world_cache.loaded_biomes() {
    //     let biome_name = biome.clone().name;
    //     // if biome.id.biome == BiomeTypeEnum::Ocean || biome.id.biome == BiomeTypeEnum::DeepOcean {
    //     //     continue;
    //     // }
    //     // if biome.id.biome != BiomeTypeEnum::Savanna {
    //     //     continue;
    //     // }
    //     if spawned_biomes.contains(&biome.get_storage_key()) {
    //         continue;
    //     }

    //     info!(
    //         "Spawning {} triangles for biome {:?} chunk ({},{}).",
    //         biome.mesh_data.triangles.len(),
    //         biome.id.biome,
    //         biome.id.x,
    //         biome.id.y
    //     );

    //     let mesh_data = biome.mesh_data.clone();

    //     let mesh = Mesh::new(
    //         PrimitiveTopology::TriangleList,
    //         RenderAssetUsages::RENDER_WORLD,
    //     )
    //     .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.triangles)
    //     .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
    //     .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

    //     let world_position = Vec2::new(
    //         biome.id.x as f32 * constants::CHUNK_SIZE.x,
    //         biome.id.y as f32 * constants::CHUNK_SIZE.y,
    //     );

    //     // Create an atlas instead of using a new one every time
    //     let color = *get_biome_color(&biome.id.biome).as_color();

    //     commands.spawn((
    //         Name::new(format!("Biome_{}", biome_name)),
    //         Mesh2d(meshes.add(mesh)),
    //         MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
    //         Transform::from_translation(world_position.extend(-1000.0)),
    //         Biome {
    //             name: biome_name,
    //             id: biome.id,
    //         },
    //     ));
    // }
}

pub fn spawn_building(
    mut commands: Commands,
    world_cache_opt: Option<Res<WorldCache>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

        match (&building_base.category, &building.specific_data) {
            (BuildingCategoryEnum::Natural, BuildingSpecific::Tree(tree_data)) => {
                let tree_type = TreeTypeEnum::Cedar; // TODO: create assets for oak and larch
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
                    tree_type.to_name_lowercase(),
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
    if let Some(image_handle) = building_atlas.get_sprite(building_type, variant) {
        let image_size = images.get(&*image_handle).map(|img| {
            let size = img.texture_descriptor.size;
            Vec2::new(size.width as f32, size.height as f32)
        });

        let custom_size = image_size.map(|size| Vec2::new(48.0, 48.0 * (size.y / size.x)));

        let mut position = Vec2::new(world_position.x, world_position.y + 8.);

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

// Setup du material par défaut au démarrage
pub fn setup_default_terrain_material(
    mut commands: Commands,
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

    let default_material = materials.add(TerrainMaterial {
        sdf_texture: dummy_texture,
        sdf_params: SdfParams {
            beach_start: 0.,
            beach_end: 0.2,
            opacity_start: 0.,
            opacity_end: 0.,
        }, // has_coast = 0.0
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
