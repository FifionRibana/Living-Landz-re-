use std::collections::HashMap;

use bevy::prelude::*;
use bincode::{Decode, Encode};
use i_triangle::float::{triangulatable::Triangulatable, triangulation::Triangulation};
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contours::Contour;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{BiomeType, constants};
use shared::{MeshData as SharedMeshData, TerrainChunkData, TerrainChunkId};

use super::mesh_data::MeshData;

use crate::utils::{algorithm, file_system};

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct TerrainChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub mesh_data: MeshData,
    pub outlines: Vec<Vec<[f64; 2]>>,
    pub generated_at: u64,
}

impl TerrainChunkMeshData {
    pub fn to_shared_terrain_chunk_data(&self, name: &str, id: TerrainChunkId) -> TerrainChunkData {
        TerrainChunkData {
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

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct TerrainMeshData {
    pub name: String,
    /// Largeur et hauteur du terrain
    pub width: u32,
    pub height: u32,
    /// Facteur d'échelle appliqué
    pub scale: [u32; 2],
    pub chunks: HashMap<TerrainChunkId, TerrainChunkMeshData>,
    pub generated_at: u64,
}

impl TerrainMeshData {
    pub fn from_image(name: &str, image: &DynamicImage, scale: &Vec2, cache_path: &str) -> Self {
        let start = std::time::Instant::now();
        let load_result = file_system::load_from_disk(cache_path);
        let mut loaded = false;
        let mut scaled_image: ImageBuffer<image::Luma<u8>, Vec<u8>> = ImageBuffer::default();
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
            tracing::info!("Upscaling image by: {}x{}", scale.x, scale.y);
            let t1 = std::time::Instant::now();

            let flipped_image = image::imageops::flip_vertical(&image.to_luma8());

            scaled_image = TerrainMeshData::resize_image(&flipped_image, scale, 178);
            let scaled_image_ref = &scaled_image;
            tracing::info!(
                "    image upscaled to {}x{} in {:?}",
                scaled_image_ref.width(),
                scaled_image_ref.height(),
                t1.elapsed()
            );

            file_system::save_to_disk(scaled_image_ref, cache_path);
        }

        let scaled_image_ref = &scaled_image;

        tracing::info!("Spliting image into chunks");
        let t2 = std::time::Instant::now();
        let scaled_width = scaled_image_ref.width();
        let scaled_height = scaled_image_ref.height();

        let n_chunk_x = (scaled_width as f32 / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height as f32 / constants::CHUNK_SIZE.y).ceil() as i32;

        let mut chunks = HashMap::new();

        for cy in 0..n_chunk_y {
            for cx in 0..n_chunk_x {
                let chunk_id = TerrainChunkId { x: cx, y: cy };
                let x_offset = (cx * constants::CHUNK_SIZE.x as i32) as u32;
                let y_offset = (cy * constants::CHUNK_SIZE.y as i32) as u32;

                let cropped = ImageBuffer::from_fn(
                    constants::CHUNK_SIZE.x as u32,
                    constants::CHUNK_SIZE.y as u32,
                    |px, py| *scaled_image.get_pixel(x_offset + px, y_offset + py),
                );

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

        tracing::info!("TerrainMeshData completed in {:?}", start.elapsed());

        Self {
            name: name.to_string(),
            width: image.width() as u32,
            height: image.height() as u32,
            scale: [scale.x as u32, scale.y as u32],
            chunks: chunk_meshes
                .into_iter()
                .map(|(id, meshes)| {
                    (
                        id,
                        TerrainChunkMeshData {
                            width: constants::CHUNK_SIZE.x as u32,
                            height: constants::CHUNK_SIZE.y as u32,
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
                    )
                })
                .collect::<HashMap<_, _>>(),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn resize_image<P>(
        image: &ImageBuffer<P, Vec<u8>>,
        scale: &Vec2,
        threshold: u8,
    ) -> ImageBuffer<P, Vec<u8>>
    where
        P: image::Pixel<Subpixel = u8> + 'static,
    {
        let mut scaled_image = image::imageops::resize(
            image,
            image.width() * scale.x as u32,
            image.height() * scale.y as u32,
            image::imageops::FilterType::Lanczos3,
        );

        scaled_image.par_iter_mut().for_each(|pixel| {
            *pixel = if *pixel > threshold { u8::MAX } else { u8::MIN };
        });

        scaled_image
    }

    pub fn detect_image_contour(
        image_buffer: &ImageBuffer<Luma<u8>, Vec<u8>>,
    ) -> Vec<Vec<[f64; 2]>> {
        let contours: Vec<Contour<u64>> = imageproc::contours::find_contours(image_buffer);

        let mut shape = Vec::new();
        for contour in &contours {
            let mut contour_points: Vec<[f64; 2]> = contour
                .points
                .par_iter()
                .map(|&p| [p.x as f64, p.y as f64])
                .collect();

            let smooth_iterations: u32 = 5;
            contour_points =
                algorithm::smoothing::smooth_contour_chaikin(&contour_points, smooth_iterations);

            let first_point = contour_points.first().expect("contour is empty");
            let last_point = contour_points.last().expect("contour is empty");
            if contour_points.len() > 1 && first_point != last_point {
                contour_points.push(*first_point);
            }
            shape.push(contour_points);
        }

        shape
    }

    pub fn mesh_faces_from_contour(contour: &Vec<Vec<[f64; 2]>>) -> Triangulation<[f64; 2], u32> {
        contour
            .triangulate()
            .into_delaunay()
            .to_triangulation::<u32>()
    }

    pub fn save_png_image(name: &str, cache_path: &str) -> Result {
        info!("using: {}", cache_path);
        let load_result = file_system::load_from_disk::<Luma<u8>>(cache_path);
        let mut loaded = false;
        match load_result {
            Ok(image) => {
                let output_path = std::path::Path::new(cache_path)
                    .with_extension("upscaled.png")
                    .to_string_lossy()
                    .to_string();
                info!("saving to: {}", output_path);
                image.save_with_format(output_path, image::ImageFormat::Png)?;
            }
            _ => {
                tracing::warn!("Failed to load {} image", name);
            }
        }

        Ok(())
    }
}
