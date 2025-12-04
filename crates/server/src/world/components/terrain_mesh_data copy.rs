use std::collections::HashMap;

use bevy::prelude::*;
use bincode::{Decode, Encode};
use i_triangle::float::{triangulatable::Triangulatable, triangulation::Triangulation};
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contours::Contour;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{CoastalSkirtData, MeshData as SharedMeshData, TerrainChunkData, TerrainChunkId};
use shared::{TerrainChunkSdfData, constants};

use super::mesh_data::MeshData;

use crate::utils::{algorithm, file_system};
use crate::world::resources::{SdfConfig, SkirtConfig};

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

        // === 1. GÉNÉRER LA SDF GLOBALE ===
        tracing::info!("Generating global SDF");
        let t_sdf = std::time::Instant::now();

        let sdf_scale = 64.0 / constants::CHUNK_SIZE.x; // Ratio SDF/monde
        let global_sdf_width = (scaled_width as f32 * sdf_scale).ceil() as u32;
        let global_sdf_height = (scaled_height as f32 * sdf_scale).ceil() as u32;

        let global_sdf = generate_global_sdf(
            scaled_image_ref,
            global_sdf_width,
            global_sdf_height,
            50.0, // max_distance en pixels monde
        );

        tracing::info!(
            "    Global SDF {}x{} generated in {:?}",
            global_sdf_width,
            global_sdf_height,
            t_sdf.elapsed()
        );

        let n_chunk_x = (scaled_width as f32 / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height as f32 / constants::CHUNK_SIZE.y).ceil() as i32;

        let mut chunks = HashMap::new();
        let mut chunk_sdf_data = HashMap::new();

        let sdf_chunk_size = 64u32; // Résolution SDF par chunk

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

                // Découper la SDF
                let sdf_x_offset = (cx as u32) * sdf_chunk_size;
                let sdf_y_offset = (cy as u32) * sdf_chunk_size;

                let mut sdf_values = Vec::with_capacity((sdf_chunk_size * sdf_chunk_size) as usize);

                for sy in 0..sdf_chunk_size {
                    for sx in 0..sdf_chunk_size {
                        let global_x = sdf_x_offset + sx;
                        let global_y = sdf_y_offset + sy;

                        let value = if global_x < global_sdf_width && global_y < global_sdf_height {
                            global_sdf[(global_y * global_sdf_width + global_x) as usize]
                        } else {
                            0u8 // Eau par défaut hors limites
                        };

                        sdf_values.push(value);
                    }
                }

                let mut data = TerrainChunkSdfData::new(sdf_chunk_size as u8);
                data.values = sdf_values;
                chunk_sdf_data.insert(chunk_id, vec![data]);
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

        tracing::info!("Generating mesh faces from outlines");
        let t5 = std::time::Instant::now();

        let chunk_meshes: HashMap<_, _> = chunk_contours_ref
            .iter()
            .filter_map(|(&id, contours)| {
                let buffer = chunks.get(&id).expect("Chunk buffer not found");

                // Vérifier si c'est un chunk 100% eau
                let is_water_chunk = buffer.get_pixel(10, 10)[0] <= 30
                    && buffer.get_pixel(buffer.width() / 2, buffer.height() / 2)[0] <= 30
                    && buffer.get_pixel(buffer.width() - 10, buffer.height() - 10)[0] <= 30;

                if is_water_chunk && contours.is_empty() {
                    None // Pas de mesh pour l'eau pure
                } else {
                    Some((id, generate_full_chunk_mesh(constants::CHUNK_SIZE)))
                }
            })
            .collect();

        tracing::info!("    Meshes generated in {:?}", t5.elapsed());

        // tracing::info!("Generating skirt from outlines");
        let t6 = std::time::Instant::now();

        // let chunk_skirt_data: HashMap<TerrainChunkId, Vec<CoastalSkirtData>> = chunk_contours_ref
        //     .par_iter()
        //     .map(|(id, contours)| {
        //         let skirts: Vec<CoastalSkirtData> = contours
        //             .iter()
        //             .filter_map(|contour| {
        //                 let contour_vec2: Vec<Vec2> = contour
        //                     .iter()
        //                     .map(|p| Vec2::new(p[0] as f32, p[1] as f32))
        //                     .collect();

        //                 generate_skirt_data(
        //                     &contour_vec2,
        //                     Vec2::new(600.0, 503.0),
        //                     &SkirtConfig::default(),
        //                     2.0,
        //                 )
        //             })
        //             .collect();

        //         (*id, skirts)
        //     })
        //     .collect();
        // let chunk_skirt_data_ref = &chunk_skirt_data;

        // tracing::info!("    Skirt generated in {:?}", t6.elapsed());

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
                                sdf_data: chunk_sdf_data.get(&id).cloned().unwrap_or_default(),
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

            let dist = distance_point_to_segment(point, a, b);
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

                        if is_segment_on_chunk_edge(a, b, edge_threshold, max_x, max_y) {
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
                    .map(|(a, b)| distance_point_to_segment(local_pos, *a, *b))
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
        is_land: impl Fn(Vec2) -> bool + Sync,
    ) -> Vec<u8> {
        let res = config.resolution as usize;
        let texel_size_x = config.chunk_world_size_x / config.resolution as f32;
        let texel_size_y = config.chunk_world_size_y / config.resolution as f32;

        let edge_threshold = 2.0;
        let max_x = config.chunk_world_size_x;
        let max_y = config.chunk_world_size_y;
        let extend_distance = config.max_distance * 2.0;

        // D'abord, corriger les contours pour avoir des intersections exactes aux bords
        let corrected_contours: Vec<Vec<Vec2>> = all_contours
            .iter()
            .map(|contour| snap_contour_to_edges(contour, edge_threshold, max_x, max_y))
            .collect();

        let segments: Vec<(Vec2, Vec2)> = corrected_contours
            .iter()
            .flat_map(|contour| {
                if contour.len() < 2 {
                    return vec![];
                }

                (0..contour.len())
                    .filter_map(|i| {
                        let a = contour[i];
                        let b = contour[(i + 1) % contour.len()];

                        // Ignorer les segments alignés sur les bords
                        if is_segment_aligned_on_edge(a, b, edge_threshold, max_x, max_y) {
                            return None;
                        }

                        // Prolonger perpendiculairement aux bords
                        let (extended_a, extended_b) = extend_segment_at_edges(
                            a,
                            b,
                            edge_threshold,
                            max_x,
                            max_y,
                            extend_distance,
                        );

                        Some((extended_a, extended_b))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if segments.is_empty() {
            let center = Vec2::new(
                config.chunk_world_size_x / 2.0,
                config.chunk_world_size_y / 2.0,
            );
            if is_land(center) {
                return vec![255u8; res * res];
            } else {
                return vec![0u8; res * res];
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

                let min_dist = segments
                    .iter()
                    .map(|(a, b)| distance_point_to_segment(local_pos, *a, *b))
                    .fold(f32::MAX, f32::min);

                let signed_dist = if is_land(local_pos) {
                    min_dist
                } else {
                    -min_dist
                };

                let normalized = (signed_dist / config.max_distance).clamp(-1.0, 1.0);
                ((normalized + 1.0) * 0.5 * 255.0) as u8
            })
            .collect()
    }
}

fn snap_contour_to_edges(contour: &[Vec2], threshold: f32, max_x: f32, max_y: f32) -> Vec<Vec2> {
    if contour.len() < 2 {
        return contour.to_vec();
    }

    let mut result: Vec<Vec2> = Vec::with_capacity(contour.len() * 2);

    for i in 0..contour.len() {
        let a = contour[i];
        let b = contour[(i + 1) % contour.len()];

        // Snap le point A sur le bord s'il est proche
        let snapped_a = snap_point_to_edge(a, threshold, max_x, max_y);

        // Ajouter le point A (snappé)
        if result.is_empty() || (*result.last().unwrap() - snapped_a).length() > 0.1 {
            result.push(snapped_a);
        }

        // Vérifier si le segment traverse un bord du chunk
        // Si oui, ajouter le point d'intersection
        if let Some(intersection) = find_edge_intersection(a, b, threshold, max_x, max_y) {
            // Ne pas ajouter si c'est trop proche du point précédent ou suivant
            let dist_to_a = (intersection - snapped_a).length();
            let snapped_b = snap_point_to_edge(b, threshold, max_x, max_y);
            let dist_to_b = (intersection - snapped_b).length();

            if dist_to_a > threshold && dist_to_b > threshold {
                result.push(intersection);
            }
        }
    }

    result
}

/// Snap un point sur le bord le plus proche s'il est dans le threshold
fn snap_point_to_edge(p: Vec2, threshold: f32, max_x: f32, max_y: f32) -> Vec2 {
    let mut snapped = p;

    // Priorité aux coins
    if p.x < threshold {
        snapped.x = 0.0;
    } else if p.x > max_x - threshold {
        snapped.x = max_x;
    }

    if p.y < threshold {
        snapped.y = 0.0;
    } else if p.y > max_y - threshold {
        snapped.y = max_y;
    }

    snapped
}

/// Trouve le point d'intersection entre un segment et les bords du chunk
fn find_edge_intersection(
    a: Vec2,
    b: Vec2,
    threshold: f32,
    max_x: f32,
    max_y: f32,
) -> Option<Vec2> {
    let a_inside = a.x >= threshold
        && a.x <= max_x - threshold
        && a.y >= threshold
        && a.y <= max_y - threshold;
    let b_inside = b.x >= threshold
        && b.x <= max_x - threshold
        && b.y >= threshold
        && b.y <= max_y - threshold;

    // Si les deux sont à l'intérieur ou les deux à l'extérieur, pas d'intersection pertinente
    if a_inside == b_inside {
        return None;
    }

    // Tester l'intersection avec chaque bord
    let edges = [
        (Vec2::new(0.0, 0.0), Vec2::new(0.0, max_y)), // Gauche
        (Vec2::new(max_x, 0.0), Vec2::new(max_x, max_y)), // Droite
        (Vec2::new(0.0, 0.0), Vec2::new(max_x, 0.0)), // Bas
        (Vec2::new(0.0, max_y), Vec2::new(max_x, max_y)), // Haut
    ];

    for (e1, e2) in edges {
        if let Some(intersection) = line_segment_intersection(a, b, e1, e2) {
            // Vérifier que l'intersection est sur le bord du chunk
            let on_edge = intersection.x <= threshold
                || intersection.x >= max_x - threshold
                || intersection.y <= threshold
                || intersection.y >= max_y - threshold;

            if on_edge {
                return Some(snap_point_to_edge(intersection, threshold, max_x, max_y));
            }
        }
    }

    None
}

/// Calcule l'intersection entre deux segments
fn line_segment_intersection(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;

    let cross = d1.x * d2.y - d1.y * d2.x;

    if cross.abs() < 0.0001 {
        return None; // Parallèles
    }

    let d = b1 - a1;
    let t = (d.x * d2.y - d.y * d2.x) / cross;
    let u = (d.x * d1.y - d.y * d1.x) / cross;

    // Vérifier que l'intersection est dans les deux segments
    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some(a1 + d1 * t)
    } else {
        None
    }
}

fn is_point_on_chunk_edge(p: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
    p.x < threshold || p.x > max_x - threshold || p.y < threshold || p.y > max_y - threshold
}

fn is_segment_aligned_on_edge(a: Vec2, b: Vec2, threshold: f32, max_x: f32, max_y: f32) -> bool {
    let align_threshold = threshold;

    let on_left = a.x < threshold && b.x < threshold && (a.x - b.x).abs() < align_threshold;
    let on_right =
        a.x > max_x - threshold && b.x > max_x - threshold && (a.x - b.x).abs() < align_threshold;
    let on_bottom = a.y < threshold && b.y < threshold && (a.y - b.y).abs() < align_threshold;
    let on_top =
        a.y > max_y - threshold && b.y > max_y - threshold && (a.y - b.y).abs() < align_threshold;

    on_left || on_right || on_bottom || on_top
}

/// Calcule la distance à un segment, en ignorant les extrémités sur les bords
fn distance_point_to_segment_edge_aware(
    p: Vec2,
    a: Vec2,
    b: Vec2,
    a_on_edge: bool,
    b_on_edge: bool,
) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_squared();

    if ab_len_sq < f32::EPSILON {
        // Segment dégénéré - ignorer si sur bord
        if a_on_edge {
            return f32::MAX;
        }
        return ap.length();
    }

    let t = ap.dot(ab) / ab_len_sq;

    // Clamper t mais en tenant compte des bords
    let t_clamped = if a_on_edge && b_on_edge {
        // Les deux extrémités sur bord - utiliser seulement la partie centrale
        t.clamp(0.1, 0.9)
    } else if a_on_edge {
        // Extrémité A sur bord - ne pas clamper vers A
        if t < 0.0 {
            return f32::MAX; // Ignorer cette distance
        }
        t.min(1.0)
    } else if b_on_edge {
        // Extrémité B sur bord - ne pas clamper vers B
        if t > 1.0 {
            return f32::MAX; // Ignorer cette distance
        }
        t.max(0.0)
    } else {
        // Aucune extrémité sur bord - comportement normal
        t.clamp(0.0, 1.0)
    };

    let closest = a + ab * t_clamped;
    (p - closest).length()
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

pub fn generate_skirt_data(
    contour_points: &[Vec2],
    chunk_size: Vec2,
    config: &SkirtConfig,
    edge_threshold: f32,
) -> Option<CoastalSkirtData> {
    let (coastal_points, is_closed, edge_info) = filter_coastal_points_with_edges(
        contour_points,
        edge_threshold,
        chunk_size.x,
        chunk_size.y,
    );

    if coastal_points.len() < 2 {
        return None;
    }

    // Simplifier le contour pour éviter les micro-segments
    let simplified = simplify_contour(&coastal_points, 2.0);
    let simplified_edges = resample_edge_info(
        &coastal_points,
        &edge_info,
        &simplified,
        edge_threshold,
        chunk_size,
    );

    if simplified.len() < 2 {
        return None;
    }

    let n = simplified.len();

    // Calculer les normales avec un lissage plus fort
    let normals = compute_smoothed_normals(&simplified, config.smoothing_window.max(9), is_closed);

    // Calculer les distances sûres
    let distances = compute_safe_offset_distances(
        &simplified,
        &normals,
        config.width,
        config.min_width_factor,
        is_closed,
    );

    // Générer les points extérieurs
    let outer_contour: Vec<Vec2> = simplified
        .iter()
        .enumerate()
        .map(|(i, &p)| p + normals[i] * distances[i])
        .collect();

    // Snap les points de bord
    let snapped_inner = snap_edge_points(&simplified, &simplified_edges, chunk_size);
    let snapped_outer = snap_edge_points(&outer_contour, &simplified_edges, chunk_size);

    let inner_points: Vec<[f32; 2]> = snapped_inner.iter().map(|p| [p.x, p.y]).collect();
    let outer_points: Vec<[f32; 2]> = snapped_outer.iter().map(|p| [p.x, p.y]).collect();

    let loop_end = if is_closed { n } else { n - 1 };
    let mut indices = Vec::with_capacity(loop_end * 6);

    for i in 0..loop_end {
        let next_i = (i + 1) % n;

        let i0 = i as u32;
        let i1 = next_i as u32;
        let o0 = (n + i) as u32;
        let o1 = (n + next_i) as u32;

        indices.push(i0);
        indices.push(o0);
        indices.push(i1);

        indices.push(i1);
        indices.push(o0);
        indices.push(o1);
    }

    Some(CoastalSkirtData {
        inner_points,
        outer_points,
        indices,
    })
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
fn extend_segment_at_edges(
    a: Vec2,
    b: Vec2,
    edge_threshold: f32,
    max_x: f32,
    max_y: f32,
    extend_distance: f32,
) -> (Vec2, Vec2) {
    let a_edge = get_edge_type(a, edge_threshold, max_x, max_y);
    let b_edge = get_edge_type(b, edge_threshold, max_x, max_y);

    let extended_a = match a_edge {
        Some(EdgeType::Left) => Vec2::new(a.x - extend_distance, a.y),
        Some(EdgeType::Right) => Vec2::new(a.x + extend_distance, a.y),
        Some(EdgeType::Bottom) => Vec2::new(a.x, a.y - extend_distance),
        Some(EdgeType::Top) => Vec2::new(a.x, a.y + extend_distance),
        Some(EdgeType::None) => a,
        _ => a,
    };

    let extended_b = match b_edge {
        Some(EdgeType::Left) => Vec2::new(b.x - extend_distance, b.y),
        Some(EdgeType::Right) => Vec2::new(b.x + extend_distance, b.y),
        Some(EdgeType::Bottom) => Vec2::new(b.x, b.y - extend_distance),
        Some(EdgeType::Top) => Vec2::new(b.x, b.y + extend_distance),
        Some(EdgeType::None) => b,
        _ => b,
    };

    (extended_a, extended_b)
}

#[derive(Clone, Copy, PartialEq)]
enum EdgeType {
    None,
    Left,
    Right,
    Bottom,
    Top,
}

fn get_edge_type(p: Vec2, threshold: f32, max_x: f32, max_y: f32) -> Option<EdgeType> {
    if p.x < threshold {
        Some(EdgeType::Left)
    } else if p.x > max_x - threshold {
        Some(EdgeType::Right)
    } else if p.y < threshold {
        Some(EdgeType::Bottom)
    } else if p.y > max_y - threshold {
        Some(EdgeType::Top)
    } else {
        None
    }
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

pub fn generate_signed_sdf_from_image(
    buffer: &ImageBuffer<Luma<u8>, Vec<u8>>,
    config: &SdfConfig,
) -> Vec<u8> {
    let res = config.resolution as usize;
    let texel_size_x = config.chunk_world_size_x / config.resolution as f32;
    let texel_size_y = config.chunk_world_size_y / config.resolution as f32;

    let img_width = buffer.width() as f32;
    let img_height = buffer.height() as f32;

    // Ratio entre espace monde et pixels image
    let scale_x = img_width / config.chunk_world_size_x;
    let scale_y = img_height / config.chunk_world_size_y;

    (0..res * res)
        .into_par_iter()
        .map(|idx| {
            let x = idx % res;
            let y = idx / res;

            // Position dans l'espace monde
            let world_x = (x as f32 + 0.5) * texel_size_x;
            let world_y = (y as f32 + 0.5) * texel_size_y;

            // Position dans l'image
            let img_x = (world_x * scale_x) as i32;
            let img_y = (world_y * scale_y) as i32;

            // Déterminer si le point courant est sur terre
            let current_is_land = if img_x >= 0
                && img_x < buffer.width() as i32
                && img_y >= 0
                && img_y < buffer.height() as i32
            {
                buffer.get_pixel(img_x as u32, img_y as u32)[0] > 30
            } else {
                false
            };

            // Chercher la distance au pixel le plus proche de type opposé
            let search_radius = (config.max_distance * scale_x.max(scale_y)) as i32 + 1;
            let mut min_dist_sq = f32::MAX;

            for dy in -search_radius..=search_radius {
                for dx in -search_radius..=search_radius {
                    let nx = img_x + dx;
                    let ny = img_y + dy;

                    if nx < 0
                        || nx >= buffer.width() as i32
                        || ny < 0
                        || ny >= buffer.height() as i32
                    {
                        continue;
                    }

                    let neighbor_is_land = buffer.get_pixel(nx as u32, ny as u32)[0] > 30;

                    // Si le voisin est de type opposé, calculer la distance
                    if neighbor_is_land != current_is_land {
                        let dist_sq = (dx as f32 / scale_x).powi(2) + (dy as f32 / scale_y).powi(2);
                        min_dist_sq = min_dist_sq.min(dist_sq);
                    }
                }
            }

            let min_dist = min_dist_sq.sqrt();

            let signed_dist = if current_is_land { min_dist } else { -min_dist };

            let normalized = (signed_dist / config.max_distance).clamp(-1.0, 1.0);
            ((normalized + 1.0) * 0.5 * 255.0) as u8
        })
        .collect()
}

fn generate_global_sdf(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    sdf_width: u32,
    sdf_height: u32,
    max_distance: f32,
) -> Vec<u8> {
    let img_width = image.width() as f32;
    let img_height = image.height() as f32;

    let scale_x = img_width / sdf_width as f32;
    let scale_y = img_height / sdf_height as f32;

    let search_radius = (max_distance / scale_x.min(scale_y)).ceil() as i32 + 1;

    (0..(sdf_width * sdf_height) as usize)
        .into_par_iter()
        .map(|idx| {
            let x = (idx % sdf_width as usize) as i32;
            let y = (idx / sdf_width as usize) as i32;

            // Position dans l'image
            let img_x = ((x as f32 + 0.5) * scale_x) as i32;
            let img_y = ((y as f32 + 0.5) * scale_y) as i32;

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
            let mut min_dist_sq = f32::MAX;

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
                        // Distance en unités monde
                        let world_dx = dx as f32 * scale_x;
                        let world_dy = dy as f32 * scale_y;
                        let dist_sq = world_dx * world_dx + world_dy * world_dy;
                        min_dist_sq = min_dist_sq.min(dist_sq);
                    }
                }
            }

            let min_dist = if min_dist_sq == f32::MAX {
                max_distance
            } else {
                min_dist_sq.sqrt()
            };

            let signed_dist = if current_is_land { min_dist } else { -min_dist };

            let normalized = (signed_dist / max_distance).clamp(-1.0, 1.0);
            ((normalized + 1.0) * 0.5 * 255.0) as u8
        })
        .collect()
}
