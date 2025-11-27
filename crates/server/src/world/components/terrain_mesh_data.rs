use std::collections::HashMap;

use bevy::prelude::*;
use bincode::{Decode, Encode};
use i_triangle::float::{triangulatable::Triangulatable, triangulation::Triangulation};
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contours::Contour;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{MeshData as SharedMeshData, TerrainChunkData, TerrainChunkId};
use shared::{TerrainChunkSdfData, constants};

use super::mesh_data::MeshData;

use crate::utils::{algorithm, file_system};
use crate::world::resources::SdfConfig;

#[derive(Default, Encode, Decode, Clone)]
pub struct TerrainChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub mesh_data: MeshData,
    pub sdf_data: Vec<TerrainChunkSdfData>,
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
            sdf_data: self.sdf_data.clone(),
            outline: self.outlines.clone(),
            generated_at: self.generated_at,
        }
    }
}

#[derive(Default, Encode, Decode, Clone)]
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
    pub fn from_image(
        name: &str,
        image: &DynamicImage,
        scale: &Vec2,
        cache_path: &str,
    ) -> (
        Self,
        HashMap<TerrainChunkId, ImageBuffer<image::Luma<u8>, Vec<u8>>>,
        ImageBuffer<image::Luma<u8>, Vec<u8>>,
    ) {
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

        let scaled_image_output = scaled_image.clone();
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

        let chunk_contours = (&chunks)
            .into_iter()
            .map(|(&id, buffer)| {
                // let buffer_ref = &buffer;
                let mut contours = TerrainMeshData::detect_image_contour(buffer);

                if contours.len() == 0 && buffer.get_pixel(10, 10)[0] > 0 {
                    let width = buffer.width() as f64;
                    let height = buffer.height() as f64;
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

        let chunk_contours_ref = &chunk_contours;

        tracing::info!("Generating sdf points from outlines");
        let t4 = std::time::Instant::now();

        // TODO: transform into parallel iterator
        let chunk_sdf_data_ref: HashMap<_, _> = chunk_contours_ref
            .par_iter() // Paralléliser sur les chunks
            .map(|(id, contours)| {
                let chunk_origin = Vec2::new(
                    id.x as f32 * constants::CHUNK_SIZE.x,
                    id.y as f32 * constants::CHUNK_SIZE.y,
                );

                let sdf_data: Vec<TerrainChunkSdfData> = contours
                    .iter()
                    .map(|contour| {
                        let image_width = 64.0;
                        let image_height = 64.0;

                        let scale_x = constants::CHUNK_SIZE.x / image_width;
                        let scale_y = constants::CHUNK_SIZE.y / image_height;

                        let world_contour: Vec<Vec2> = contour
                            .iter()
                            .map(|&point| {
                                chunk_origin
                                    + Vec2::new(
                                        point[0] as f32 * scale_x,
                                        point[1] as f32 * scale_y,
                                    )
                            })
                            .collect();

                        tracing::info!(
                            "Chunk {:?} - origin: {:?}, contour len: {}, image bounds: {:?} to {:?}, world bounds: {:?} to {:?}",
                            id,
                            chunk_origin,
                            contour.len(),
                            contour.iter().fold([f64::MAX, f64::MAX], |acc, p| [acc[0].min(p[0]), acc[1].min(p[1])]),
                            contour.iter().fold([f64::MIN, f64::MIN], |acc, p| [acc[0].max(p[0]), acc[1].max(p[1])]),
                            world_contour.iter().fold(Vec2::MAX, |acc, &p| acc.min(p)),
                            world_contour.iter().fold(Vec2::MIN, |acc, &p| acc.max(p)),
                        );

                        // Utiliser les dimensions réelles de l'image, pas un carré
                        let image_width = 600.0;
                        let image_height = 503.0;  // Ou récupérer depuis buffer.height()
                        
                        let mut data = TerrainChunkSdfData::new(64);
                        let local_contour: Vec<Vec2> = contour
                            .iter()
                            .map(|&point| Vec2::new(point[0] as f32, point[1] as f32))
                            .collect();

                        data.values = TerrainMeshData::generate_sdf_data(
                            &local_contour,
                            Vec2::ZERO,
                            &SdfConfig {
                                resolution: 64,
                                chunk_world_size_x: 600.0,  // Largeur de l'image/mesh en pixels
                                chunk_world_size_y: 503.0,  // Hauteur de l'image/mesh en pixels
                                max_distance: 30.0,       // 30 pixels de transition pour la plage
                            },
                        );
                        data
                    })
                    .collect();

                (*id, sdf_data)
            })
            .collect();

        tracing::info!("    SDF points generated in {:?}", t4.elapsed());

        tracing::info!("Generating mesh faces from outlines");
        let t5 = std::time::Instant::now();

        let chunk_meshes = chunk_contours_ref
            .into_iter()
            .map(|(&id, contours)| (id, TerrainMeshData::mesh_faces_from_contour(&contours)))
            .collect::<HashMap<_, _>>();

        tracing::info!("    Meshes generated in {:?}", t5.elapsed());

        tracing::info!("TerrainMeshData completed in {:?}", start.elapsed());

        (
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
                                sdf_data: chunk_sdf_data_ref
                                    .get(&id)
                                    .expect("Chunk SDF data not found")
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
            },
            chunks,
            scaled_image_output,
        )
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

    // Dans generate_sdf_data
    pub fn generate_sdf_data(
        contour_points: &[Vec2],
        chunk_origin: Vec2,
        config: &SdfConfig,
    ) -> Vec<u8> {
        let res = config.resolution as usize;
        let texel_size_x = config.chunk_world_size_x / config.resolution as f32;
        let texel_size_y = config.chunk_world_size_y / config.resolution as f32;
        
        (0..res * res)
            .into_par_iter()
            .map(|idx| {
                let x = idx % res;
                let y = idx / res;
                
                let local_pos = chunk_origin + Vec2::new(
                    (x as f32 + 0.5) * texel_size_x,
                    (y as f32 + 0.5) * texel_size_y,
                );
                
                let dist = TerrainMeshData::compute_min_distance_to_contour(local_pos, contour_points);
                let normalized = (dist / config.max_distance).clamp(0.0, 1.0);
                
                (normalized * 255.0) as u8
            })
            .collect()
    }

    fn compute_min_distance_to_contour(point: Vec2, contour: &[Vec2]) -> f32 {
        if contour.len() < 2 {
            return f32::MAX;
        }

        let mut min_dist = f32::MAX;

        for i in 0..contour.len() {
            let a = contour[i];
            let b = contour[(i + 1) % contour.len()];

            let dist = TerrainMeshData::distance_point_to_segment(point, a, b);
            min_dist = min_dist.min(dist);
        }

        min_dist
    }

    fn distance_point_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
        let ab = b - a;
        let ap = p - a;

        let ab_len_sq = ab.length_squared();

        if ab_len_sq < f32::EPSILON {
            return ap.length();
        }

        let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);

        let closest = a + ab * t;

        (p - closest).length()
    }
}
