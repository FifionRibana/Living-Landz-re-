use bevy::prelude::*;
use bincode::{Decode, Encode};
use hexx::*;
use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use rayon::prelude::*;
use shared::{
    BiomeChunkData, BiomeColor, MeshData as SharedMeshData, ShoreType, TerrainChunkId, constants, get_biome_color, grid::{CellData, GridCell}, types::{BiomeChunkId, BiomeTypeEnum, find_closest_biome}
};
use std::collections::{HashMap, HashSet};

use super::mesh_data::MeshData;
use super::terrain_mesh_data::TerrainMeshData;

use crate::utils::{algorithm, file_system};

#[derive(Default, Encode, Decode, Clone)]
pub struct BiomeChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub biome_type: BiomeTypeEnum,
    pub mesh_data: MeshData,
    pub outlines: Vec<Vec<[f64; 2]>>,
    pub generated_at: u64,
}

impl BiomeChunkMeshData {
    pub fn to_shared_biome_chunk_data(&self, name: &str, id: BiomeChunkId) -> BiomeChunkData {
        BiomeChunkData {
            name: name.to_string(),
            id,
            mesh_data: SharedMeshData {
                triangles: self.mesh_data.triangles.clone(),
                normals: self.mesh_data.normals.clone(),
                uvs: self.mesh_data.uvs.clone(),
            },
            outline: self.outlines.clone(),
            generated_at: self.generated_at,
        }
    }
}

pub struct BiomeMeshData {
    pub name: String,
    /// Largeur et hauteur du terrain
    pub width: u32,
    pub height: u32,
    /// Facteur d'échelle appliqué
    pub scale: [u32; 2],
    pub chunks: HashMap<BiomeChunkId, BiomeChunkMeshData>,
    pub generated_at: u64,
}

impl BiomeMeshData {
    pub fn from_image(
        name: &str,
        image: &DynamicImage,
        binary_mask: &ImageBuffer<Luma<u8>, Vec<u8>>,
        _binary_masks: &HashMap<TerrainChunkId, ImageBuffer<Luma<u8>, Vec<u8>>>,
        scale: &Vec2,
        cache_directory: &str,
    ) -> Self {
        let start = std::time::Instant::now();

        let mut biome_mesh_data = HashMap::new();

        for biome_type in BiomeTypeEnum::iter() {
            info!("=== {:?} ===", biome_type);
            let load_result = file_system::load_from_disk(
                format!("{}{}_biomemap_{:?}.bin", cache_directory, name, biome_type).as_str(),
            );
            let mut scaled_image: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::default();
            let loaded = match load_result {
                Ok(image) => {
                    scaled_image = image;
                    true
                }
                _ => false,
            };

            if !loaded {
                tracing::info!(
                    "Masking and Upscaling image by: {}x{} for biome: {:?}",
                    scale.x,
                    scale.y,
                    biome_type
                );
                let t1 = std::time::Instant::now();

                let flipped_image = image::imageops::flip_vertical(&image.to_rgba8());
                let flipped_image_ref = &flipped_image;

                let mut biome_binary_map =
                    ImageBuffer::new(flipped_image_ref.width(), flipped_image_ref.height());

                // Biome binary map generation
                for (x, y, pixel) in flipped_image_ref.enumerate_pixels() {
                    let pixel_color = BiomeColor::srgb_u8(pixel[0], pixel[1], pixel[2]);
                    let closest_biome = find_closest_biome(&pixel_color);

                    let value = if closest_biome == biome_type {
                        u8::MAX
                    } else {
                        u8::MIN
                    };

                    biome_binary_map.put_pixel(x, y, Luma([value]));
                }

                // Mask
                // algorithm::smoothing::mask_luma_map(&mut biome_binary_map, binary_mask);

                // Clean up
                biome_binary_map = algorithm::smoothing::open_binary_map(&biome_binary_map, 1);

                // upscaling
                scaled_image = TerrainMeshData::resize_image(&biome_binary_map, scale, 178);
                algorithm::smoothing::mask_luma_map(&mut scaled_image, binary_mask);
                let scaled_image_ref = &scaled_image;

                tracing::info!(
                    "    image upscaled to {}x{} in {:?}",
                    scaled_image_ref.width(),
                    scaled_image_ref.height(),
                    t1.elapsed()
                );

                let _ = file_system::save_to_disk(
                    scaled_image_ref,
                    format!("{}{}_biomemap_{:?}.bin", cache_directory, name, biome_type).as_str(),
                );
            }

            tracing::info!("Spliting image into chunks");
            let scaled_image_ref = &scaled_image;
            let t2 = std::time::Instant::now();
            let scaled_width = scaled_image_ref.width();
            let scaled_height = scaled_image_ref.height();

            let n_chunk_x = (scaled_width as f32 / constants::CHUNK_SIZE.x).ceil() as i32;
            let n_chunk_y = (scaled_height as f32 / constants::CHUNK_SIZE.y).ceil() as i32;

            let mut chunks = HashMap::new();

            for cy in 0..n_chunk_y {
                for cx in 0..n_chunk_x {
                    let chunk_id = BiomeChunkId {
                        x: cx,
                        y: cy,
                        biome: biome_type,
                    };
                    let x_offset = (cx * constants::CHUNK_SIZE.x as i32) as u32;
                    let y_offset = (cy * constants::CHUNK_SIZE.y as i32) as u32;

                    let cropped = ImageBuffer::from_fn(
                        constants::CHUNK_SIZE.x as u32,
                        constants::CHUNK_SIZE.y as u32,
                        |px, py| *scaled_image.get_pixel(x_offset + px, y_offset + py),
                    );

                    // masking cropped
                    // algorithm::smoothing::mask_luma_map(
                    //     &mut cropped,
                    //     binary_masks
                    //         .get(&TerrainChunkId { x: cx, y: cy })
                    //         .expect("Chunk not found"),
                    // );

                    chunks.insert(chunk_id, cropped);
                }
            }
            tracing::info!("    {} chunks split in {:?}", chunks.len(), t2.elapsed());

            tracing::info!("Detecting chunks outlines");
            let t3 = std::time::Instant::now();

            let chunk_contours = chunks
                .into_iter()
                .map(|(id, buffer)| {
                    let buffer_ref = &buffer;
                    let mut contours = TerrainMeshData::detect_image_contour(buffer_ref);

                    if contours.len() == 0 && buffer_ref.get_pixel(10, 10)[0] > 0 {
                        let width = buffer_ref.width() as f64;
                        let height = buffer_ref.height() as f64;
                        contours = vec![vec![
                            [0.0, 0.0],
                            [0.0, height],
                            [width, height],
                            [width, 0.0],
                        ]];
                    }
                    (id, contours)
                })
                .collect::<HashMap<_, _>>();

            tracing::info!(
                "    {} outlines detected in {:?}",
                chunk_contours.len(),
                t3.elapsed()
            );

            tracing::info!("Generating mesh faces from outlines");
            let t4 = std::time::Instant::now();

            let chunk_contours_ref = &chunk_contours;

            let chunk_meshes = chunk_contours_ref
                .into_iter()
                .map(|(&id, contours)| (id, TerrainMeshData::mesh_faces_from_contour(&contours)))
                .collect::<HashMap<_, _>>();

            tracing::info!("    Meshes generated in {:?}", t4.elapsed());

            for (id, meshes) in chunk_meshes.into_iter() {
                biome_mesh_data.insert(
                    id,
                    BiomeChunkMeshData {
                        width: constants::CHUNK_SIZE.x as u32,
                        height: constants::CHUNK_SIZE.y as u32,
                        biome_type,
                        mesh_data: MeshData::from_meshes(meshes),
                        outlines: chunk_contours_ref
                            .get(&id)
                            .expect("Chunk contour not found")
                            .clone(),
                        generated_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                );
            }
        }
        tracing::info!("TerrainMeshData completed in {:?}", start.elapsed());

        Self {
            name: name.to_string(),
            width: image.width() as u32,
            height: image.height() as u32,
            scale: [scale.x as u32, scale.y as u32],
            chunks: biome_mesh_data,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn sample_biome(
        name: &str,
        biome_map: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        scale: &Vec2,
        hex_layout: &HexLayout,
        cache_directory: &str,
    ) -> Vec<CellData> {
        let load_result = file_system::load_from_disk(
            format!("{}{}_biomemap.bin", cache_directory, name).as_str(),
        );
        // let mut loaded = false;
        let mut scaled_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::default();
        let loaded = match load_result {
            Ok(image) => {
                scaled_image = image;
                true
            }
            _ => false,
        };

        if !loaded {
            let t1 = std::time::Instant::now();

            let flipped_image = image::imageops::flip_vertical(biome_map);
            let flipped_image_ref = &flipped_image;
            // upscaling
            scaled_image = image::imageops::resize(
                flipped_image_ref,
                biome_map.width() * scale.x as u32,
                biome_map.height() * scale.y as u32,
                image::imageops::FilterType::Nearest,
            );

            let scaled_image_ref = &scaled_image;

            tracing::info!(
                "    image upscaled to {}x{} in {:?}",
                scaled_image_ref.width(),
                scaled_image_ref.height(),
                t1.elapsed()
            );

            let _ = file_system::save_to_disk(
                scaled_image_ref,
                format!("{}{}_biomemap.bin", cache_directory, name).as_str(),
            );
        }

        scaled_image
            .enumerate_pixels()
            .par_bridge()
            .fold(HashMap::new, |mut acc, (px, py, pixel)| {
                let current_hex = hex_layout.world_pos_to_hex(Vec2::new(px as f32, py as f32));
                let vertices = &hex_layout
                    .hex_corners(current_hex)
                    .into_iter()
                    .map(|p| (p.x, p.y))
                    .collect::<Vec<(f32, f32)>>();
                if BiomeMeshData::point_in_polygon((px as f32, py as f32), vertices) {
                    let color = BiomeColor::srgb_u8(pixel[0], pixel[1], pixel[2]);
                    *acc.entry(current_hex)
                        .or_insert(HashMap::new())
                        .entry(color)
                        .or_insert(0) += 1;
                }
                acc
            })
            .reduce(HashMap::new, |mut acc, other| {
                for (hex, color_map) in other {
                    let entry = acc.entry(hex).or_insert_with(HashMap::new);

                    for (color, count) in color_map {
                        *entry.entry(color).or_insert(0) += count;
                    }
                }
                acc
            })
            .into_iter()
            .map(|(hex_cell, color_map)| {
                let world_pos = hex_layout.hex_to_world_pos(hex_cell);
                CellData {
                    cell: GridCell {
                        q: hex_cell.x,
                        r: hex_cell.y,
                    },
                    chunk: TerrainChunkId {
                        x: world_pos.x.div_euclid(constants::CHUNK_SIZE.x) as i32,
                        y: world_pos.y.div_euclid(constants::CHUNK_SIZE.y) as i32,
                    },
                    biome: color_map
                        .into_iter()
                        .max_by_key(|(_, count)| *count)
                        .map(|(color, _)| find_closest_biome(&color))
                        .unwrap_or(BiomeTypeEnum::DeepOcean),
                        shore_type: ShoreType::None,
                }
            })
            .collect()
    }

    pub fn sample_biome_for_chunk(
        source_biome_flipped: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        source_binary: &ImageBuffer<Luma<u8>, Vec<u8>>,
        scale: &Vec2,
        hex_layout: &HexLayout,
        chunk_id: &TerrainChunkId,
    ) -> Vec<CellData> {
        let img_w = source_biome_flipped.width();
        let img_h = source_biome_flipped.height();

        let x_min = chunk_id.x as f32 * constants::CHUNK_SIZE.x;
        let y_min = chunk_id.y as f32 * constants::CHUNK_SIZE.y;
        let x_max = (chunk_id.x + 1) as f32 * constants::CHUNK_SIZE.x;
        let y_max = (chunk_id.y + 1) as f32 * constants::CHUNK_SIZE.y;

        // Map chunk bounds to source pixel coordinates (with 1px margin)
        let src_x_min = ((x_min / scale.x).floor() as i32 - 1).max(0) as u32;
        let src_y_min = ((y_min / scale.y).floor() as i32 - 1).max(0) as u32;
        let src_x_max = ((x_max / scale.x).ceil() as u32 + 1).min(img_w - 1);
        let src_y_max = ((y_max / scale.y).ceil() as u32 + 1).min(img_h - 1);

        // Per-hex vote maps
        let mut hex_land: HashMap<hexx::Hex, HashMap<u32, usize>> = HashMap::new();
        let mut hex_ocean: HashMap<hexx::Hex, usize> = HashMap::new();
        let mut hex_lake: HashMap<hexx::Hex, usize> = HashMap::new();
        let mut hex_near_coast: HashSet<hexx::Hex> = HashSet::new(); // hexes influenced by coast

        for sy in src_y_min..=src_y_max {
            for sx in src_x_min..=src_x_max {
                let bin_val = source_binary.get_pixel(sx, sy)[0];
                let is_land = bin_val > 200;
                let is_lake = bin_val >= 100 && bin_val <= 160;
                let is_ocean = !is_land && !is_lake; // includes anti-aliased edges

                // Neighbor analysis
                let mut has_ocean_neighbor = false;
                let mut has_lake_neighbor = false;
                let mut has_land_neighbor = false;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = sx as i32 + dx;
                        let ny = sy as i32 + dy;
                        if nx >= 0 && nx < img_w as i32 && ny >= 0 && ny < img_h as i32 {
                            let nv = source_binary.get_pixel(nx as u32, ny as u32)[0];
                            if nv > 200 { has_land_neighbor = true; }
                            else if nv >= 100 && nv <= 160 { has_lake_neighbor = true; }
                            else { has_ocean_neighbor = true; }
                        }
                    }
                }

                let is_ocean_coast = is_land && has_ocean_neighbor;
                let is_lake_bank = is_land && has_lake_neighbor && !has_ocean_neighbor;
                let is_ocean_nearshore = is_ocean && has_land_neighbor; // water side of coast

                let pixel = source_biome_flipped.get_pixel(sx, sy);
                let color = BiomeColor::srgb_u8(pixel[0], pixel[1], pixel[2]);
                let biome = find_closest_biome(&color);
                let id = biome.to_id();

                // Sub-pixel sampling
                let world_x_start = sx as f32 * scale.x;
                let world_y_start = sy as f32 * scale.y;
                let steps = (scale.x / 20.0).ceil().max(1.0) as i32;
                let step_x = scale.x / steps as f32;
                let step_y = scale.y / steps as f32;

                for iy in 0..steps {
                    for ix in 0..steps {
                        let wx = world_x_start + (ix as f32 + 0.5) * step_x;
                        let wy = world_y_start + (iy as f32 + 0.5) * step_y;

                        if wx < x_min || wx >= x_max || wy < y_min || wy >= y_max {
                            continue;
                        }

                        let hex = hex_layout.world_pos_to_hex(Vec2::new(wx, wy));

                        if is_ocean_coast {
                            // Land pixel next to ocean → shore votes
                            *hex_ocean.entry(hex).or_insert(0) += 1;
                            hex_near_coast.insert(hex);
                            if (id as usize) < 16 && id > 2 && id != 13 {
                                *hex_land.entry(hex).or_insert_with(HashMap::new)
                                    .entry(id as u32).or_insert(0) += 1;
                            }
                        } else if is_lake_bank {
                            // Land pixel next to lake → lake bank votes
                            *hex_lake.entry(hex).or_insert(0) += 1;
                            if (id as usize) < 16 && id > 2 && id != 13 {
                                *hex_land.entry(hex).or_insert_with(HashMap::new)
                                    .entry(id as u32).or_insert(0) += 1;
                            }
                        } else if is_ocean_nearshore {
                            // Ocean pixel next to land → nearshore water
                            *hex_ocean.entry(hex).or_insert(0) += 1;
                            hex_near_coast.insert(hex);
                        } else if is_lake {
                            *hex_lake.entry(hex).or_insert(0) += 1;
                        } else if is_land && (id as usize) < 16 && id > 2 && id != 13 {
                            *hex_land.entry(hex).or_insert_with(HashMap::new)
                                .entry(id as u32).or_insert(0) += 1;
                        } else {
                            // Deep ocean
                            *hex_ocean.entry(hex).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Collect all hexes
        let mut all_hexes: HashSet<hexx::Hex> = HashSet::new();
        all_hexes.extend(hex_land.keys());
        all_hexes.extend(hex_ocean.keys());
        all_hexes.extend(hex_lake.keys());

        all_hexes
            .into_iter()
            .map(|hex_cell| {
                let land_total: usize = hex_land.get(&hex_cell)
                    .map(|m| m.values().sum()).unwrap_or(0);
                let ocean_total = hex_ocean.get(&hex_cell).copied().unwrap_or(0);
                let lake_total = hex_lake.get(&hex_cell).copied().unwrap_or(0);
                let total = land_total + ocean_total + lake_total;
                let ocean_ratio = ocean_total as f32 / total.max(1) as f32;
                let lake_ratio = lake_total as f32 / total.max(1) as f32;
                let is_near_coast = hex_near_coast.contains(&hex_cell);

                // Determine biome
                let biome = if land_total > 0 {
                    // Has land votes → pick best land biome
                    let id_map = hex_land.get(&hex_cell).unwrap();
                    let best_id = id_map.iter()
                        .max_by_key(|(_, count)| *count)
                        .map(|(id, _)| *id)
                        .unwrap_or(5);
                    BiomeTypeEnum::from_id(best_id as i16)
                        .unwrap_or(BiomeTypeEnum::Grassland)
                } else if lake_total > 0 {
                    BiomeTypeEnum::Lake
                } else if is_near_coast {
                    BiomeTypeEnum::Ocean  // shallow ocean near coast
                } else {
                    BiomeTypeEnum::DeepOcean  // far from coast
                };

                // Determine shore type (independent of biome)
                let shore_type = if ocean_ratio > 0.25 {
                    shared::ShoreType::Shoreline
                } else if lake_ratio > 0.25 {
                    shared::ShoreType::Lakebank
                } else {
                    shared::ShoreType::None
                };

                CellData {
                    cell: GridCell { q: hex_cell.x, r: hex_cell.y },
                    chunk: *chunk_id,
                    biome,
                    shore_type,
                }
            })
            .collect()
    }

    #[inline]
    fn point_in_polygon(point: (f32, f32), vertices: &[(f32, f32)]) -> bool {
        let (px, py) = point;
        let mut inside = false;
        let mut p1 = vertices[5];

        for &p2 in vertices {
            if (p2.1 > py) != (p1.1 > py) {
                let slope = (px - p2.0) * (p1.1 - p2.1) - (p1.0 - p2.0) * (py - p2.1);
                if (p2.1 > py) && (slope < 0.0) || (p2.1 <= py) && (slope > 0.0) {
                    inside = !inside;
                }
            }
            p1 = p2;
        }

        inside
    }

    pub fn save_png_image(name: &str, cache_directory: &str) -> Result {
        for biome_type in BiomeTypeEnum::iter() {
            info!("=== {:?} ===", biome_type);
            let input_path = format!("{}{}_biomemap_{:?}.bin", cache_directory, name, biome_type);
            info!("using: {}", input_path);
            let load_result = file_system::load_from_disk::<Luma<u8>>(input_path.as_str());
            match load_result {
                Ok(image) => {
                    let output_path = std::path::Path::new(&input_path)
                        .with_extension("upscaled.png")
                        .to_string_lossy()
                        .to_string();
                    info!("saving to: {}", output_path);
                    image.save_with_format(output_path, image::ImageFormat::Png)?;
                }
                _ => {
                    tracing::warn!("Failed to load {:?} biome {} image", biome_type, name);
                }
            }
        }
        Ok(())
    }
}
