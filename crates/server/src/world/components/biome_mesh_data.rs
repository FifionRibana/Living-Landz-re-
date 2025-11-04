use bevy::prelude::*;
use bincode::{Decode, Encode};
use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use serde::{Deserialize, Serialize};
use shared::{
    BiomeChunkData, BiomeColor, MeshData as SharedMeshData, TerrainChunkId, constants,
    types::{BiomeChunkId, BiomeType, find_closest_biome},
};
use std::collections::HashMap;

use super::mesh_data::MeshData;
use super::terrain_mesh_data::TerrainMeshData;

use crate::utils::{algorithm, file_system};

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BiomeChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub biome_type: BiomeType,
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
        binary_masks: &HashMap<TerrainChunkId, ImageBuffer<Luma<u8>, Vec<u8>>>,
        scale: &Vec2,
        cache_directory: &str,
    ) -> Self {
        let start = std::time::Instant::now();

        let mut biome_mesh_data = HashMap::new();

        for biome_type in BiomeType::iter() {
            info!("=== {:?} ===", biome_type);
            let load_result = file_system::load_from_disk(
                format!("{}{}_biomemap_{:?}.bin", cache_directory, name, biome_type).as_str(),
            );
            let mut loaded = false;
            let mut scaled_image: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::default();
            match load_result {
                Ok(image) => {
                    scaled_image = image;
                    loaded = true;
                }
                _ => {
                    loaded = false;
                }
            }

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

                file_system::save_to_disk(
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

                    let mut cropped = ImageBuffer::from_fn(
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

    pub fn save_png_image(name: &str, cache_directory: &str) -> Result {
        for biome_type in BiomeType::iter() {
            info!("=== {:?} ===", biome_type);
            let input_path = format!("{}{}_biomemap_{:?}.bin", cache_directory, name, biome_type);
            info!("using: {}", input_path);
            let load_result = file_system::load_from_disk::<Luma<u8>>(input_path.as_str());
            let mut loaded = false;
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
