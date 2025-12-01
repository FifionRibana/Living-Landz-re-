use std::collections::HashMap;

use bevy::prelude::*;
use bincode::{Decode, Encode};
use i_triangle::float::{triangulatable::Triangulatable, triangulation::Triangulation};
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contours::Contour;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{MeshData as SharedMeshData, TerrainChunkData, TerrainChunkId, OceanData};
use shared::{TerrainChunkSdfData, HeightmapChunkData, constants};

use super::mesh_data::MeshData;

use crate::utils::{algorithm, file_system};
use crate::world::resources::SdfConfig;

#[derive(Default, Encode, Decode, Clone)]
pub struct TerrainChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub mesh_data: MeshData,
    pub sdf_data: Vec<TerrainChunkSdfData>,
    pub heightmap_data: Option<HeightmapChunkData>,
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
            heightmap_data: self.heightmap_data.clone(),
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
        heightmap_image: Option<&DynamicImage>,
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

        tracing::info!("Generating global SDF");
        let t_sdf = std::time::Instant::now();

        let sdf_resolution = 64usize;
        let global_sdf_width = n_chunk_x as usize * sdf_resolution;
        let global_sdf_height = n_chunk_y as usize * sdf_resolution;
        let max_distance = 50.0f32;

        let global_sdf = generate_global_sdf(
            scaled_image_ref,
            global_sdf_width,
            global_sdf_height,
            n_chunk_x as f32 * constants::CHUNK_SIZE.x,
            n_chunk_y as f32 * constants::CHUNK_SIZE.y,
            max_distance,
        );

        tracing::info!(
            "    Global SDF {}x{} generated in {:?}",
            global_sdf_width,
            global_sdf_height,
            t_sdf.elapsed()
        );

        tracing::info!("Splitting SDF into chunks");
        let t3 = std::time::Instant::now();

        let mut chunk_sdf_data: HashMap<TerrainChunkId, Vec<TerrainChunkSdfData>> = HashMap::new();

        let chunk_sdf_values = split_sdf_into_chunks(
            &global_sdf,
            global_sdf_width,
            global_sdf_height,
            sdf_resolution,
            n_chunk_x,
            n_chunk_y,
        );

        let chunk_sdf_data: HashMap<TerrainChunkId, Vec<TerrainChunkSdfData>> = chunk_sdf_values
            .into_iter()
            .map(|(id, values)| {
                let mut data = TerrainChunkSdfData::new(sdf_resolution as u8);
                data.values = values;
                (id, vec![data])
            })
            .collect();

        let chunk_sdf_data_ref = &chunk_sdf_data;

        tracing::info!("    SDF chunks created in {:?}", t3.elapsed());

        // Générer la heightmap globale et la découper en chunks (optionnel)
        let chunk_heightmap_data: HashMap<TerrainChunkId, Option<HeightmapChunkData>> = if let Some(heightmap_img) = heightmap_image {
            tracing::info!("Generating global heightmap");
            let t_heightmap = std::time::Instant::now();

            let heightmap_luma = heightmap_img.to_luma8();
            let global_heightmap = generate_global_heightmap(
                &heightmap_luma,
                global_sdf_width,
                global_sdf_height,
            );

            tracing::info!(
                "    Global heightmap {}x{} generated in {:?}",
                global_sdf_width,
                global_sdf_height,
                t_heightmap.elapsed()
            );

            tracing::info!("Splitting heightmap into chunks");
            let t_heightmap_split = std::time::Instant::now();

            let chunk_heightmap_values = split_sdf_into_chunks(
                &global_heightmap,
                global_sdf_width,
                global_sdf_height,
                sdf_resolution,
                n_chunk_x,
                n_chunk_y,
            );

            let result: HashMap<TerrainChunkId, Option<HeightmapChunkData>> = chunk_heightmap_values
                .into_iter()
                .map(|(id, values)| {
                    let data = HeightmapChunkData::from_values(sdf_resolution as u8, values);
                    (id, Some(data))
                })
                .collect();

            tracing::info!("    Heightmap chunks created in {:?}", t_heightmap_split.elapsed());

            result
        } else {
            // Pas de heightmap : None pour tous les chunks
            (0..n_chunk_y)
                .flat_map(|cy| {
                    (0..n_chunk_x).map(move |cx| {
                        (TerrainChunkId { x: cx, y: cy }, None)
                    })
                })
                .collect()
        };

        let chunk_heightmap_data_ref = &chunk_heightmap_data;

        tracing::info!("Detecting chunks outlines");
        let t4 = std::time::Instant::now();

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
            t4.elapsed()
        );

        let chunk_contours_ref = &chunk_contours;

        tracing::info!("Generating mesh faces from outlines");
        let t5 = std::time::Instant::now();

        let chunk_meshes: HashMap<_, _> = chunk_sdf_data
            .iter()
            .filter_map(|(&id, sdf_data)| {
                if let Some(sdf) = sdf_data.first() {
                    // Un chunk a besoin d'un mesh s'il contient de la terre
                    let has_land = sdf.values.iter().any(|&v| v > 128);

                    if has_land {
                        Some((id, generate_full_chunk_mesh(constants::CHUNK_SIZE)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        tracing::info!(
            "    {} meshes generated in {:?}",
            chunk_meshes.len(),
            t5.elapsed()
        );

        let t6 = std::time::Instant::now();

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
                                mesh_data: meshes.clone(),
                                outlines: chunk_contours_ref.get(&id).cloned().unwrap_or_default(),
                                sdf_data: chunk_sdf_data_ref.get(&id).cloned().unwrap_or_default(),
                                heightmap_data: chunk_heightmap_data_ref.get(&id).cloned().unwrap_or(None),
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

                let local_pos = chunk_origin
                    + Vec2::new(
                        (x as f32 + 0.5) * texel_size_x,
                        (y as f32 + 0.5) * texel_size_y,
                    );

                let dist =
                    TerrainMeshData::compute_min_distance_to_contour(local_pos, contour_points);
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

    pub fn generate_sdf_data_multi_contours(
        all_contours: &[Vec<Vec2>],
        chunk_origin: Vec2,
        config: &SdfConfig,
    ) -> Vec<u8> {
        let res = config.resolution as usize;
        let texel_size_x = config.chunk_world_size_x / config.resolution as f32;
        let texel_size_y = config.chunk_world_size_y / config.resolution as f32;

        // Tolérance en pixels pour la détection des bords
        let edge_threshold = 2.0;
        let max_x = config.chunk_world_size_x - edge_threshold;
        let max_y = config.chunk_world_size_y - edge_threshold;

        // Collecter tous les segments valides (pas alignés sur les bords)
        let valid_segments: Vec<(Vec2, Vec2)> = all_contours
            .iter()
            .flat_map(|contour| {
                if contour.len() < 2 {
                    return vec![];
                }

                (0..contour.len())
                    .filter_map(|i| {
                        let a = contour[i];
                        let b = contour[(i + 1) % contour.len()];

                        if TerrainMeshData::is_segment_on_chunk_edge(
                            a,
                            b,
                            edge_threshold,
                            max_x,
                            max_y,
                        ) {
                            None // Ignorer ce segment
                        } else {
                            Some((a, b))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if valid_segments.is_empty() {
            return vec![255u8; res * res];
        }

        (0..res * res)
            .into_par_iter()
            .map(|idx| {
                let x = idx % res;
                let y = idx / res;

                let local_pos = chunk_origin
                    + Vec2::new(
                        (x as f32 + 0.5) * texel_size_x,
                        (y as f32 + 0.5) * texel_size_y,
                    );

                let min_dist = valid_segments
                    .iter()
                    .map(|(a, b)| TerrainMeshData::distance_point_to_segment(local_pos, *a, *b))
                    .fold(f32::MAX, f32::min);

                let normalized = (min_dist / config.max_distance).clamp(0.0, 1.0);
                (normalized * 255.0) as u8
            })
            .collect()
    }

    pub fn generate_signed_sdf_data_multi_contours(
        all_contours: &[Vec<Vec2>],
        chunk_origin: Vec2,
        config: &SdfConfig,
        is_land: impl Fn(Vec2) -> bool + Sync, // Fonction pour savoir si un point est sur terre
    ) -> Vec<u8> {
        let res = config.resolution as usize;
        let texel_size_x = config.chunk_world_size_x / config.resolution as f32;
        let texel_size_y = config.chunk_world_size_y / config.resolution as f32;

        let edge_threshold = 2.0;
        let max_x = config.chunk_world_size_x - edge_threshold;
        let max_y = config.chunk_world_size_y - edge_threshold;

        let valid_segments: Vec<(Vec2, Vec2)> = all_contours
            .iter()
            .flat_map(|contour| {
                if contour.len() < 2 {
                    return vec![];
                }

                (0..contour.len())
                    .filter_map(|i| {
                        let a = contour[i];
                        let b = contour[(i + 1) % contour.len()];

                        if TerrainMeshData::is_segment_on_chunk_edge(
                            a,
                            b,
                            edge_threshold,
                            max_x,
                            max_y,
                        ) {
                            None
                        } else {
                            Some((a, b))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if valid_segments.is_empty() {
            let center = Vec2::new(
                config.chunk_world_size_x / 2.0,
                config.chunk_world_size_y / 2.0,
            );
            if is_land(center) {
                return vec![255u8; res * res]; // Tout terre
            } else {
                return vec![0u8; res * res]; // Tout eau
            }
        }

        (0..res * res)
            .into_par_iter()
            .map(|idx| {
                let x = idx % res;
                let y = idx / res;

                let local_pos = chunk_origin
                    + Vec2::new(
                        (x as f32 + 0.5) * texel_size_x,
                        (y as f32 + 0.5) * texel_size_y,
                    );

                let min_dist = valid_segments
                    .iter()
                    .map(|(a, b)| TerrainMeshData::distance_point_to_segment(local_pos, *a, *b))
                    .fold(f32::MAX, f32::min);

                // Signed : positif sur terre, négatif dans l'eau
                let signed_dist = if is_land(local_pos) {
                    min_dist
                } else {
                    -min_dist
                };

                // Encoder : 128 = sur le contour, 0 = loin dans l'eau, 255 = loin sur terre
                let normalized = (signed_dist / config.max_distance).clamp(-1.0, 1.0);
                let encoded = ((normalized + 1.0) * 0.5 * 255.0) as u8;

                encoded
            })
            .collect()
    }

    /// Vérifie si un segment est entièrement sur un bord du chunk
    fn is_segment_on_edge(a: Vec2, b: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
        // Bord gauche
        if a.x < threshold && b.x < threshold {
            return true;
        }
        // Bord droit
        if a.x > max_x && b.x > max_x {
            return true;
        }
        // Bord bas
        if a.y < threshold && b.y < threshold {
            return true;
        }
        // Bord haut
        if a.y > max_y && b.y > max_y {
            return true;
        }
        false
    }

    /// Vérifie si un segment est aligné sur un bord du chunk (côte artificielle à ignorer)
    fn is_segment_on_chunk_edge(a: Vec2, b: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
        // Tolérance pour la position sur le bord
        let pos_threshold = threshold;
        // Tolérance pour l'alignement (segment quasi horizontal ou vertical)
        let align_threshold = threshold;

        // Bord gauche : les deux points près de x=0 ET segment quasi vertical
        let on_left =
            a.x < pos_threshold && b.x < pos_threshold && (a.x - b.x).abs() < align_threshold;

        // Bord droit : les deux points près de x=max ET segment quasi vertical
        let on_right = a.x > max_x && b.x > max_x && (a.x - b.x).abs() < align_threshold;

        // Bord bas : les deux points près de y=0 ET segment quasi horizontal
        let on_bottom =
            a.y < pos_threshold && b.y < pos_threshold && (a.y - b.y).abs() < align_threshold;

        // Bord haut : les deux points près de y=max ET segment quasi horizontal
        let on_top = a.y > max_y && b.y > max_y && (a.y - b.y).abs() < align_threshold;

        on_left || on_right || on_bottom || on_top
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

// server/src/terrain/mesh_generator.rs

pub fn generate_full_chunk_mesh(chunk_size: Vec2) -> MeshData {
    let w = chunk_size.x;
    let h = chunk_size.y;

    // Deux triangles couvrant tout le chunk
    let triangles = vec![
        // Triangle 1
        [0.0, 0.0, 0.0],
        [w, 0.0, 0.0],
        [w, h, 0.0],
        // Triangle 2
        [0.0, 0.0, 0.0],
        [w, h, 0.0],
        [0.0, h, 0.0],
    ];

    let normals = vec![[0.0, 0.0, 1.0]; 6];

    // UVs correspondants (0-1 sur le chunk)
    let uvs = vec![
        // Triangle 1
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        // Triangle 2
        [0.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];

    MeshData {
        triangles,
        normals,
        uvs,
    }
}

/// Simplification Douglas-Peucker
fn simplify_contour(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Marquer les points à garder
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;

    simplify_recursive(points, 0, points.len() - 1, tolerance, &mut keep);

    points
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, &p)| p)
        .collect()
}

fn simplify_recursive(
    points: &[Vec2],
    start: usize,
    end: usize,
    tolerance: f32,
    keep: &mut [bool],
) {
    if end <= start + 1 {
        return;
    }

    let start_pt = points[start];
    let end_pt = points[end];

    let mut max_dist = 0.0;
    let mut max_idx = start;

    for i in (start + 1)..end {
        let dist = point_to_line_distance(points[i], start_pt, end_pt);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > tolerance {
        keep[max_idx] = true;
        simplify_recursive(points, start, max_idx, tolerance, keep);
        simplify_recursive(points, max_idx, end, tolerance, keep);
    }
}

fn point_to_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line = line_end - line_start;
    let len_sq = line.length_squared();

    if len_sq < f32::EPSILON {
        return (point - line_start).length();
    }

    let t = ((point - line_start).dot(line) / len_sq).clamp(0.0, 1.0);
    let projection = line_start + line * t;

    (point - projection).length()
}

/// Re-calcule les edge_info pour les points simplifiés
fn resample_edge_info(
    original: &[Vec2],
    original_edges: &[EdgeType],
    simplified: &[Vec2],
    threshold: f32,
    chunk_size: Vec2,
) -> Vec<EdgeType> {
    simplified
        .iter()
        .map(|&p| detect_point_edge(p, threshold, chunk_size.x, chunk_size.y))
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum EdgeType {
    None,
    Left,
    Right,
    Bottom,
    Top,
}

fn detect_point_edge(p: Vec2, threshold: f32, max_x: f32, max_y: f32) -> EdgeType {
    if p.x < threshold {
        EdgeType::Left
    } else if p.x > max_x - threshold {
        EdgeType::Right
    } else if p.y < threshold {
        EdgeType::Bottom
    } else if p.y > max_y - threshold {
        EdgeType::Top
    } else {
        EdgeType::None
    }
}

fn snap_edge_points(points: &[Vec2], edge_info: &[EdgeType], chunk_size: Vec2) -> Vec<Vec2> {
    points
        .iter()
        .enumerate()
        .map(
            |(i, &p)| match edge_info.get(i).unwrap_or(&EdgeType::None) {
                EdgeType::Left => Vec2::new(0.0, p.y),
                EdgeType::Right => Vec2::new(chunk_size.x, p.y),
                EdgeType::Bottom => Vec2::new(p.x, 0.0),
                EdgeType::Top => Vec2::new(p.x, chunk_size.y),
                EdgeType::None => p,
            },
        )
        .collect()
}

fn filter_coastal_points_with_edges(
    contour: &[Vec2],
    threshold: f32,
    max_x: f32,
    max_y: f32,
) -> (Vec<Vec2>, bool, Vec<EdgeType>) {
    if contour.len() < 3 {
        return (vec![], false, vec![]);
    }

    let n = contour.len();
    let mut result: Vec<Vec2> = Vec::new();
    let mut edge_info = Vec::new();
    let mut on_edge_count = 0;

    for i in 0..n {
        let a = contour[i];
        let b = contour[(i + 1) % n];

        if is_segment_on_chunk_edge(a, b, threshold, max_x, max_y) {
            on_edge_count += 1;
            continue;
        }

        if result.is_empty() || (*result.last().unwrap() - a).length() > 0.5 {
            result.push(a);
            edge_info.push(detect_point_edge(a, threshold, max_x, max_y));
        }
    }

    let is_closed = on_edge_count == 0;

    if is_closed && result.len() > 1 {
        if (*result.first().unwrap() - *result.last().unwrap()).length() < 1.0 {
            result.pop();
            edge_info.pop();
        }
    }

    (result, is_closed, edge_info)
}

fn compute_smoothed_normals(points: &[Vec2], window_size: usize, is_closed: bool) -> Vec<Vec2> {
    let n = points.len();
    let half_window = window_size / 2;

    (0..n)
        .map(|i| {
            let mut avg_normal = Vec2::ZERO;
            let mut count = 0;

            for offset in 0..window_size {
                let raw_idx = i as i32 - half_window as i32 + offset as i32;

                let idx = if is_closed {
                    ((raw_idx % n as i32) + n as i32) as usize % n
                } else {
                    if raw_idx < 0 || raw_idx >= n as i32 - 1 {
                        continue;
                    }
                    raw_idx as usize
                };

                let next_idx = if is_closed {
                    (idx + 1) % n
                } else {
                    if idx + 1 >= n {
                        continue;
                    }
                    idx + 1
                };

                let dir = (points[next_idx] - points[idx]).normalize_or_zero();
                if dir != Vec2::ZERO {
                    avg_normal += Vec2::new(-dir.y, dir.x);
                    count += 1;
                }
            }

            if count > 0 {
                avg_normal /= count as f32;
            }

            let result = avg_normal.normalize_or_zero();
            if result == Vec2::ZERO {
                // Fallback
                let prev_i = if i > 0 {
                    i - 1
                } else if is_closed {
                    n - 1
                } else {
                    0
                };
                let next_i = if i < n - 1 {
                    i + 1
                } else if is_closed {
                    0
                } else {
                    n - 1
                };
                let dir = (points[next_i] - points[prev_i]).normalize_or_zero();
                Vec2::new(-dir.y, dir.x)
            } else {
                result
            }
        })
        .collect()
}

fn compute_safe_offset_distances(
    points: &[Vec2],
    normals: &[Vec2],
    max_width: f32,
    min_factor: f32,
    is_closed: bool,
) -> Vec<f32> {
    let n = points.len();
    let min_width = max_width * min_factor;

    let mut distances = vec![max_width; n];

    for i in 0..n {
        let point = points[i];
        let normal = normals[i];

        for j in 0..n {
            let dist_idx = {
                let d = (j as i32 - i as i32).abs();
                d.min(n as i32 - d) as usize
            };

            if dist_idx < 3 {
                continue;
            }

            let j_next = if is_closed {
                (j + 1) % n
            } else if j + 1 < n {
                j + 1
            } else {
                continue;
            };

            let seg_start = points[j];
            let seg_end = points[j_next];

            if let Some(t) = ray_segment_intersection(point, normal, seg_start, seg_end) {
                if t > 0.0 && t < distances[i] {
                    distances[i] = (t * 0.4).max(min_width);
                }
            }
        }
    }

    for _ in 0..5 {
        let outer_points: Vec<Vec2> = points
            .iter()
            .enumerate()
            .map(|(i, &p)| p + normals[i] * distances[i])
            .collect();

        let mut adjusted = false;

        for i in 0..n {
            let i_next = if is_closed {
                (i + 1) % n
            } else if i + 1 < n {
                i + 1
            } else {
                continue;
            };

            let seg1_start = outer_points[i];
            let seg1_end = outer_points[i_next];

            for j in 0..n {
                let dist_idx = {
                    let d = (j as i32 - i as i32).abs();
                    d.min(n as i32 - d) as usize
                };

                if dist_idx < 3 {
                    continue;
                }

                let j_next = if is_closed {
                    (j + 1) % n
                } else if j + 1 < n {
                    j + 1
                } else {
                    continue;
                };

                let seg2_start = outer_points[j];
                let seg2_end = outer_points[j_next];

                if segments_intersect(seg1_start, seg1_end, seg2_start, seg2_end) {
                    distances[i] = (distances[i] * 0.6).max(min_width);
                    distances[i_next] = (distances[i_next] * 0.6).max(min_width);
                    distances[j] = (distances[j] * 0.6).max(min_width);
                    distances[j_next] = (distances[j_next] * 0.6).max(min_width);
                    adjusted = true;
                }
            }
        }

        if !adjusted {
            break;
        }
    }

    smooth_distances(&mut distances, is_closed);

    distances
}

fn smooth_distances(distances: &mut [f32], is_closed: bool) {
    let n = distances.len();
    if n < 3 {
        return;
    }

    let original = distances.to_vec();

    for i in 0..n {
        if !is_closed && (i == 0 || i == n - 1) {
            continue;
        }

        let prev_i = if i > 0 { i - 1 } else { n - 1 };
        let next_i = if i < n - 1 { i + 1 } else { 0 };

        let avg = (original[prev_i] + original[i] + original[next_i]) / 3.0;
        distances[i] = distances[i].min(avg);
    }
}

fn segments_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let d1 = cross_2d(b2 - b1, a1 - b1);
    let d2 = cross_2d(b2 - b1, a2 - b1);
    let d3 = cross_2d(a2 - a1, b1 - a1);
    let d4 = cross_2d(a2 - a1, b2 - a1);

    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn ray_segment_intersection(
    ray_origin: Vec2,
    ray_dir: Vec2,
    seg_a: Vec2,
    seg_b: Vec2,
) -> Option<f32> {
    let v1 = ray_origin - seg_a;
    let v2 = seg_b - seg_a;
    let v3 = Vec2::new(-ray_dir.y, ray_dir.x);

    let dot = v2.dot(v3);
    if dot.abs() < 0.0001 {
        return None;
    }

    let t1 = (v2.x * v1.y - v2.y * v1.x) / dot;
    let t2 = v1.dot(v3) / dot;

    if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
        Some(t1)
    } else {
        None
    }
}

fn is_segment_on_chunk_edge(a: Vec2, b: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
    let align_threshold = threshold;

    let on_left = a.x < threshold && b.x < threshold && (a.x - b.x).abs() < align_threshold;
    let on_right =
        a.x > max_x - threshold && b.x > max_x - threshold && (a.x - b.x).abs() < align_threshold;
    let on_bottom = a.y < threshold && b.y < threshold && (a.y - b.y).abs() < align_threshold;
    let on_top =
        a.y > max_y - threshold && b.y > max_y - threshold && (a.y - b.y).abs() < align_threshold;

    on_left || on_right || on_bottom || on_top
}

fn generate_global_sdf(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    sdf_width: usize,
    sdf_height: usize,
    world_width: f32,
    world_height: f32,
    max_distance: f32,
) -> Vec<u8> {
    let img_width = image.width() as f32;
    let img_height = image.height() as f32;

    // Ratio SDF -> image
    let sdf_to_img_x = img_width / sdf_width as f32;
    let sdf_to_img_y = img_height / sdf_height as f32;

    // Ratio monde -> image (pour max_distance)
    let world_to_img_x = img_width / world_width;
    let world_to_img_y = img_height / world_height;

    // Rayon de recherche en pixels image
    let search_radius = ((max_distance * world_to_img_x.max(world_to_img_y)) as i32).max(1);

    tracing::info!(
        "    SDF params: sdf_to_img ({:.2}, {:.2}), search_radius: {}",
        sdf_to_img_x,
        sdf_to_img_y,
        search_radius
    );

    (0..sdf_width * sdf_height)
        .into_par_iter()
        .map(|idx| {
            let sx = idx % sdf_width;
            let sy = idx / sdf_width;

            // Position dans l'image
            let img_x = ((sx as f32 + 0.5) * sdf_to_img_x) as i32;
            let img_y = ((sy as f32 + 0.5) * sdf_to_img_y) as i32;

            // Déterminer si le point est sur terre
            let current_is_land = if img_x >= 0
                && img_x < image.width() as i32
                && img_y >= 0
                && img_y < image.height() as i32
            {
                image.get_pixel(img_x as u32, img_y as u32)[0] > 30
            } else {
                false
            };

            // Chercher le pixel de type opposé le plus proche
            let mut min_dist_sq = i32::MAX;

            for dy in -search_radius..=search_radius {
                for dx in -search_radius..=search_radius {
                    let nx = img_x + dx;
                    let ny = img_y + dy;

                    if nx < 0 || nx >= image.width() as i32 || ny < 0 || ny >= image.height() as i32
                    {
                        continue;
                    }

                    let neighbor_is_land = image.get_pixel(nx as u32, ny as u32)[0] > 30;

                    if neighbor_is_land != current_is_land {
                        let dist_sq = dx * dx + dy * dy;
                        min_dist_sq = min_dist_sq.min(dist_sq);
                    }
                }
            }

            // Convertir en distance monde
            let min_dist_pixels = (min_dist_sq as f32).sqrt();
            let min_dist_world = min_dist_pixels / world_to_img_x.max(world_to_img_y);

            let min_dist = if min_dist_sq == i32::MAX {
                max_distance
            } else {
                min_dist_world.min(max_distance)
            };

            let signed_dist = if current_is_land { min_dist } else { -min_dist };

            let normalized = (signed_dist / max_distance).clamp(-1.0, 1.0);
            ((normalized + 1.0) * 0.5 * 255.0) as u8
        })
        .collect()
}

/// Génère les données d'océan globales (SDF + heightmap) pour un monde
/// À utiliser pour le rendu de l'océan autour du continent
pub fn generate_ocean_data(
    name: String,
    binary_map: &DynamicImage,
    heightmap_image: &DynamicImage,
    n_chunk_x: i32,
    n_chunk_y: i32,
    world_width: f32,
    world_height: f32,
) -> OceanData {
    let sdf_resolution = 64usize;
    let global_width = n_chunk_x as usize * sdf_resolution;
    let global_height = n_chunk_y as usize * sdf_resolution;
    let max_distance = 50.0f32;

    tracing::info!("Generating ocean data for world: {}", name);
    tracing::info!("  Resolution: {}x{}", global_width, global_height);

    // 1. Générer le SDF global
    let binary_luma = binary_map.to_luma8();
    let global_sdf = generate_global_sdf(
        &binary_luma,
        global_width,
        global_height,
        world_width,
        world_height,
        max_distance,
    );

    // 2. Générer la heightmap globale
    let heightmap_luma = heightmap_image.to_luma8();
    let global_heightmap = generate_global_heightmap(
        &heightmap_luma,
        global_width,
        global_height,
    );

    // 3. Créer OceanData
    OceanData::new(
        name,
        global_width,
        global_height,
        max_distance,
        global_sdf,
        global_heightmap,
    )
}

/// Génère une heightmap globale à partir d'une image
/// Échantillonne l'image source à la résolution souhaitée
pub fn generate_global_heightmap(
    heightmap_image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    target_width: usize,
    target_height: usize,
) -> Vec<u8> {
    let img_width = heightmap_image.width() as f32;
    let img_height = heightmap_image.height() as f32;

    // Ratio heightmap -> image
    let scale_x = img_width / target_width as f32;
    let scale_y = img_height / target_height as f32;

    tracing::info!(
        "    Heightmap params: {}x{} -> {}x{} (scale: {:.2}x{:.2})",
        heightmap_image.width(),
        heightmap_image.height(),
        target_width,
        target_height,
        scale_x,
        scale_y
    );

    (0..target_width * target_height)
        .into_par_iter()
        .map(|idx| {
            let tx = idx % target_width;
            let ty = idx / target_width;

            // Position dans l'image source avec échantillonnage bilinéaire
            let img_x = (tx as f32 + 0.5) * scale_x;
            let img_y = (ty as f32 + 0.5) * scale_y;

            // Sample bilinéaire
            sample_heightmap_bilinear(heightmap_image, img_x, img_y)
        })
        .collect()
}

/// Échantillonne une heightmap avec interpolation bilinéaire
fn sample_heightmap_bilinear(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    x: f32,
    y: f32,
) -> u8 {
    let x0 = (x.floor() as i32).clamp(0, image.width() as i32 - 1) as u32;
    let y0 = (y.floor() as i32).clamp(0, image.height() as i32 - 1) as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);

    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let v00 = image.get_pixel(x0, y0)[0] as f32;
    let v10 = image.get_pixel(x1, y0)[0] as f32;
    let v01 = image.get_pixel(x0, y1)[0] as f32;
    let v11 = image.get_pixel(x1, y1)[0] as f32;

    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;
    let v = v0 * (1.0 - fy) + v1 * fy;

    v.round().clamp(0.0, 255.0) as u8
}

/// Découpe la SDF globale en chunks avec overlap correct
fn split_sdf_into_chunks(
    global_sdf: &[u8],
    global_width: usize,
    global_height: usize,
    chunk_resolution: usize,
    n_chunk_x: i32,
    n_chunk_y: i32,
) -> HashMap<TerrainChunkId, Vec<u8>> {
    let mut result = HashMap::new();

    // Overlap en texels (0.5 de chaque côté = 1 texel total d'extension)
    let overlap = 0.5f32;

    for cy in 0..n_chunk_y {
        for cx in 0..n_chunk_x {
            let chunk_id = TerrainChunkId { x: cx, y: cy };

            let mut sdf_values = Vec::with_capacity(chunk_resolution * chunk_resolution);

            let base_x = cx as f32 * chunk_resolution as f32;
            let base_y = cy as f32 * chunk_resolution as f32;

            for sy in 0..chunk_resolution {
                for sx in 0..chunk_resolution {
                    // Mapper [0, 63] → [-0.5, 63.5] dans la SDF globale
                    let t_x = sx as f32 / (chunk_resolution - 1) as f32; // 0 à 1
                    let t_y = sy as f32 / (chunk_resolution - 1) as f32;

                    // Position avec overlap symétrique
                    let global_x =
                        base_x - overlap + t_x * (chunk_resolution as f32 - 1.0 + 2.0 * overlap);
                    let global_y =
                        base_y - overlap + t_y * (chunk_resolution as f32 - 1.0 + 2.0 * overlap);

                    let value = sample_sdf_bilinear(
                        global_sdf,
                        global_width,
                        global_height,
                        global_x,
                        global_y,
                    );

                    sdf_values.push(value);
                }
            }

            result.insert(chunk_id, sdf_values);
        }
    }

    result
}

fn sample_sdf_bilinear(sdf: &[u8], width: usize, height: usize, x: f32, y: f32) -> u8 {
    let x0 = (x.floor() as i32).clamp(0, width as i32 - 1) as usize;
    let y0 = (y.floor() as i32).clamp(0, height as i32 - 1) as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let v00 = sdf[y0 * width + x0] as f32;
    let v10 = sdf[y0 * width + x1] as f32;
    let v01 = sdf[y1 * width + x0] as f32;
    let v11 = sdf[y1 * width + x1] as f32;

    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;
    let v = v0 * (1.0 - fy) + v1 * fy;

    v.round().clamp(0.0, 255.0) as u8
}
