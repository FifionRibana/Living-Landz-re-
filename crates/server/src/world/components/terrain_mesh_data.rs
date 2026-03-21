use std::collections::HashMap;

use super::mesh_data::MeshData;
use bevy::prelude::*;
use bincode::{Decode, Encode};
use i_triangle::float::{triangulatable::Triangulatable, triangulation::Triangulation};
use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use imageproc::contours::Contour;
use rayon::prelude::*;
use shared::{BiomeColor, find_closest_biome, get_biome_color};
use shared::{MeshData as SharedMeshData, OceanData, TerrainChunkData, TerrainChunkId};
use shared::{RoadChunkSdfData, TerrainChunkSdfData, constants};

use crate::utils::{algorithm, file_system};
use crate::world::resources::{SdfConfig, WorldGlobalState};

#[derive(Default, Encode, Decode, Clone)]
pub struct TerrainChunkMeshData {
    pub width: u32,
    pub height: u32,
    pub mesh_data: MeshData,
    pub sdf_data: Vec<TerrainChunkSdfData>,
    pub road_sdf_data: Option<RoadChunkSdfData>,
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
            road_sdf_data: self.road_sdf_data.clone(),
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
    /// Generate global data only (SDF, biome, heightmap). No per-chunk meshes.
    /// Returns the WorldGlobalState (held in memory) and the TerrainGlobalData (sent to client).
    pub fn generate_globals(
        name: &str,
        image: &DynamicImage,
        heightmap_image: Option<&DynamicImage>,
        biome_map: Option<&DynamicImage>,
        scale: &Vec2,
    ) -> (
        crate::world::resources::WorldGlobalState,
        Option<shared::TerrainGlobalData>,
    ) {
        let start = std::time::Instant::now();

        // Compute chunk dimensions from source + scale (NO global upscale)
        let source_w = image.width() as f32;
        let source_h = image.height() as f32;
        let scaled_width = source_w * scale.x;
        let scaled_height = source_h * scale.y;
        let n_chunk_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;

        tracing::info!(
            "    Source {}x{}, scale {}x → {}x{} world ({} chunks)",
            image.width(), image.height(), scale.x,
            scaled_width, scaled_height, n_chunk_x * n_chunk_y
        );

        // Prepare flipped source binary (for per-chunk SDF)
        let source_binary_flipped = image::imageops::flip_vertical(&image.to_luma8());

        // Generate global biome + heightmap textures
        let terrain_global_data = if let Some(biome_img) = biome_map {
            tracing::info!("Generating global biome texture");
            let t_biome = std::time::Instant::now();

            let biome_rgba = image::imageops::flip_vertical(&biome_img.to_rgba8());
            let biome_base_resolution = 4096usize;
            let biome_aspect = biome_img.height() as f32 / biome_img.width() as f32;
            let global_biome_width = biome_base_resolution;
            let global_biome_height =
                (biome_base_resolution as f32 * biome_aspect).round() as usize;

            let global_biome =
                generate_global_biome_texture(&biome_rgba, global_biome_width, global_biome_height);

            tracing::info!(
                "    Global biome texture {}x{} generated in {:?}",
                global_biome_width,
                global_biome_height,
                t_biome.elapsed()
            );

            tracing::info!("Generating global heightmap");
            let t_hm = std::time::Instant::now();
            let hm_base_resolution = 2048usize;
            let source_aspect = if let Some(hm_img) = heightmap_image {
                hm_img.height() as f32 / hm_img.width() as f32
            } else {
                0.5
            };
            let hm_width = hm_base_resolution;
            let hm_height = (hm_base_resolution as f32 * source_aspect).round() as usize;

            let global_heightmap = if let Some(hm_img) = heightmap_image {
                let heightmap_luma = hm_img.to_luma8();
                generate_global_heightmap(&heightmap_luma, hm_width, hm_height)
            } else {
                vec![128u8; hm_width * hm_height]
            };
            tracing::info!(
                "    Global heightmap {}x{} generated in {:?}",
                hm_width,
                hm_height,
                t_hm.elapsed()
            );

            let world_width = n_chunk_x as f32 * constants::CHUNK_SIZE.x;
            let world_height = n_chunk_y as f32 * constants::CHUNK_SIZE.y;

            Some(shared::TerrainGlobalData {
                name: name.to_string(),
                biome_width: global_biome_width as u32,
                biome_height: global_biome_height as u32,
                biome_values: global_biome,
                heightmap_width: hm_width as u32,
                heightmap_height: hm_height as u32,
                heightmap_values: global_heightmap,
                world_width,
                world_height,
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        } else {
            None
        };

        tracing::info!("Globals generated in {:?}", start.elapsed());

        let global_state = WorldGlobalState {
            map_name: name.to_string(),
            maps: None,
            source_binary_flipped,
            n_chunk_x,
            n_chunk_y,
            scale: *scale,
            sdf_resolution: 64,
            max_distance: 150.0,
            grid_config: None,
            source_biome_flipped_rgba: None,
        };

        (global_state, terrain_global_data)
    }

    /// Generate a single chunk's terrain data from pre-computed globals.
    pub fn generate_single_chunk(
        chunk_id: TerrainChunkId,
        global: &crate::world::resources::WorldGlobalState,
    ) -> Option<TerrainChunkMeshData> {
        // Extract SDF for this chunk
        let (sdf_data, chunk_mask) = global.generate_chunk_sdf_and_mask(&chunk_id);

        // Check if chunk has any land
        let has_land = sdf_data
            .first()
            .map(|s| s.values.iter().any(|&v| v > 128))
            .unwrap_or(false);

        if !has_land {
            return None;
        }

        // Detect contours from binary mask
        let mut contours = TerrainMeshData::detect_image_contour(&chunk_mask);

        if contours.is_empty() && chunk_mask.get_pixel(10, 10)[0] > 0 {
            let width = chunk_mask.width() as f64;
            let height = chunk_mask.height() as f64;
            contours = vec![vec![
                [0.0, 0.0],
                [0.0, height],
                [width, height],
                [width, 0.0],
            ]];
        }

        // Generate mesh
        let mesh = generate_full_chunk_mesh(constants::CHUNK_SIZE);

        Some(TerrainChunkMeshData {
            width: constants::CHUNK_SIZE.x as u32,
            height: constants::CHUNK_SIZE.y as u32,
            mesh_data: mesh,
            outlines: contours,
            sdf_data,
            road_sdf_data: None,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
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

    let total_pixels = sdf_width * sdf_height;
    tracing::info!(
        "    SDF generation: image {}x{}, target {}x{}",
        image.width(),
        image.height(),
        sdf_width,
        sdf_height
    );
    tracing::info!(
        "    SDF params: sdf_to_img ({:.2}, {:.2}), search_radius: {}",
        sdf_to_img_x,
        sdf_to_img_y,
        search_radius
    );
    tracing::info!(
        "    Total pixels to process: {} ({:.2} MB)",
        total_pixels,
        total_pixels as f64 / 1_000_000.0
    );

    (0..total_pixels)
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
    tracing::info!("=== OCEAN DATA GENERATION START ===");
    tracing::info!("World name: {}", name);
    tracing::info!(
        "Input binary_map: {}x{}",
        binary_map.width(),
        binary_map.height()
    );
    tracing::info!(
        "Input heightmap: {}x{}",
        heightmap_image.width(),
        heightmap_image.height()
    );
    tracing::info!(
        "Chunks: {}x{} (total: {})",
        n_chunk_x,
        n_chunk_y,
        n_chunk_x * n_chunk_y
    );
    tracing::info!("World dimensions: {}x{}", world_width, world_height);

  // Cap ocean SDF resolution to avoid huge textures at large scales
    let max_ocean_sdf_dim = 4096usize;
    let raw_width = n_chunk_x as usize * 64;
    let raw_height = n_chunk_y as usize * 64;

    let (global_width, global_height) = if raw_width > max_ocean_sdf_dim || raw_height > max_ocean_sdf_dim {
        let ratio = raw_height as f32 / raw_width as f32;
        if raw_width >= raw_height {
            (max_ocean_sdf_dim, (max_ocean_sdf_dim as f32 * ratio).round() as usize)
        } else {
            ((max_ocean_sdf_dim as f32 / ratio).round() as usize, max_ocean_sdf_dim)
        }
    } else {
        (raw_width, raw_height)
    };
    let max_distance = 50.0f32;

    tracing::info!("Target SDF resolution: {}x{}", global_width, global_height);

    let expected_bytes_sdf = global_width * global_height;
    let expected_bytes_heightmap = global_width * global_height;
    let expected_mb = (expected_bytes_sdf + expected_bytes_heightmap) as f64 / 1_000_000.0;

    tracing::info!("Expected memory allocation:");
    tracing::info!(
        "  - SDF: {} bytes ({:.2} MB)",
        expected_bytes_sdf,
        expected_bytes_sdf as f64 / 1_000_000.0
    );
    tracing::info!(
        "  - Heightmap: {} bytes ({:.2} MB)",
        expected_bytes_heightmap,
        expected_bytes_heightmap as f64 / 1_000_000.0
    );
    tracing::info!("  - Total: {:.2} MB", expected_mb);

    if expected_mb > 1000.0 {
        tracing::error!(
            "❌ ALLOCATION TOO LARGE! Expected {:.2} MB > 1000 MB",
            expected_mb
        );
        tracing::error!("Parameters seem wrong:");
        tracing::error!("  n_chunk_x = {}, n_chunk_y = {}", n_chunk_x, n_chunk_y);
        tracing::error!(
            "  global_width = {}, global_height = {}",
            global_width,
            global_height
        );
        panic!("Ocean data generation would allocate too much memory");
    }

    // 1. Générer le SDF global
    tracing::info!("Converting binary_map to luma8...");
    let binary_luma = binary_map.to_luma8();
    tracing::info!(
        "✓ Binary luma ready: {}x{}",
        binary_luma.width(),
        binary_luma.height()
    );

    tracing::info!("Generating global SDF...");
    let global_sdf = generate_global_sdf(
        &binary_luma,
        global_width,
        global_height,
        world_width,
        world_height,
        max_distance,
    );
    tracing::info!("✓ Global SDF generated: {} bytes", global_sdf.len());

    // 2. Générer la heightmap globale
    tracing::info!("Converting heightmap to luma8...");
    let heightmap_luma = heightmap_image.to_luma8();
    tracing::info!(
        "✓ Heightmap luma ready: {}x{}",
        heightmap_luma.width(),
        heightmap_luma.height()
    );

    tracing::info!("Generating global heightmap...");
    let global_heightmap = generate_global_heightmap(&heightmap_luma, global_width, global_height);
    tracing::info!(
        "✓ Global heightmap generated: {} bytes",
        global_heightmap.len()
    );

    // 3. Créer OceanData
    tracing::info!("Creating OceanData structure...");
    let ocean_data = OceanData::new(
        name,
        global_width,
        global_height,
        max_distance,
        global_sdf,
        global_heightmap,
        world_width,
        world_height,
    );
    tracing::info!("✓ OceanData created successfully");
    tracing::info!("=== OCEAN DATA GENERATION END ===");

    ocean_data
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

    let total_pixels = target_width * target_height;
    tracing::info!(
        "    Heightmap generation: {}x{} -> {}x{} (scale: {:.2}x{:.2})",
        heightmap_image.width(),
        heightmap_image.height(),
        target_width,
        target_height,
        scale_x,
        scale_y
    );
    tracing::info!(
        "    Total pixels to process: {} ({:.2} MB)",
        total_pixels,
        total_pixels as f64 / 1_000_000.0
    );

    (0..total_pixels)
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
fn sample_heightmap_bilinear(image: &ImageBuffer<Luma<u8>, Vec<u8>>, x: f32, y: f32) -> u8 {
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

/// Generates a global biome blend texture from the RGBA biome map.
/// For each target pixel, samples a window at SOURCE resolution and takes
/// a majority vote — this eliminates anti-aliased boundary pixels before
/// they ever become biome IDs.
///
/// Output: Vec<u8> in RGBA8 format. For each pixel:
///   R = primary biome ID * 17 (scaled to 0-255)
///   G = secondary biome ID * 17
///   B = blend factor (0 = 100% primary, 255 = 100% secondary)
///   A = 255
fn generate_global_biome_texture(
    biome_map: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    target_width: usize,
    target_height: usize,
) -> Vec<u8> {
    use rayon::prelude::*;

    let img_w = biome_map.width() as usize;
    let img_h = biome_map.height() as usize;

    tracing::info!(
        "    Biome blend texture: source {}x{} -> target {}x{}",
        img_w,
        img_h,
        target_width,
        target_height
    );

    let is_water_biome = |id: u8| -> bool { id <= 2 || id == 13 };

    // =========================================================================
    // Step 1: Pre-compute biome IDs at SOURCE resolution (fast, ~1.9M pixels)
    // =========================================================================
    let t1 = std::time::Instant::now();

    let source_ids: Vec<u8> = (0..img_w * img_h)
        .into_par_iter()
        .map(|idx| {
            let x = idx % img_w;
            let y = idx / img_w;
            let pixel = biome_map.get_pixel(x as u32, y as u32);
            let color = BiomeColor::srgb_u8(pixel[0], pixel[1], pixel[2]);
            let biome = find_closest_biome(&color);
            let id = biome.to_id() as u8;

            // Check purity — only accept if close to reference color
            let ref_color = get_biome_color(&biome);
            let dist = color.distance_to(&ref_color);
            if dist < 5 && !is_water_biome(id) {
                id
            } else {
                255u8 // mark as "impure" for dilation
            }
        })
        .collect();

    tracing::info!("    Step 1: Source IDs computed in {:?}", t1.elapsed());

    // =========================================================================
    // Step 2: Dilate impure/water pixels at SOURCE resolution (small, fast)
    // =========================================================================
    let t2 = std::time::Instant::now();

    let mut ids = source_ids;
    for _ in 0..16 {
        let snapshot = ids.clone();
        for y in 0..img_h {
            for x in 0..img_w {
                let idx = y * img_w + x;
                if snapshot[idx] != 255 && !is_water_biome(snapshot[idx]) {
                    continue;
                }
                let mut found = None;
                'search: for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i32 + dx).clamp(0, img_w as i32 - 1) as usize;
                        let ny = (y as i32 + dy).clamp(0, img_h as i32 - 1) as usize;
                        let n = snapshot[ny * img_w + nx];
                        if n != 255 && !is_water_biome(n) {
                            found = Some(n);
                            break 'search;
                        }
                    }
                }
                if let Some(b) = found {
                    ids[idx] = b;
                }
            }
        }
    }

    // Final fallback — any remaining impure/water → Grassland
    for pixel in ids.iter_mut() {
        if *pixel == 255 || is_water_biome(*pixel) {
            *pixel = 5; // Grassland
        }
    }

    tracing::info!("    Step 2: Dilation at source res in {:?}", t2.elapsed());

    // =========================================================================
    // Step 3: Compute blend at intermediate resolution (2× source) for smooth transitions
    // =========================================================================
    let t3 = std::time::Instant::now();
    let total_pixels = target_width * target_height;

    let blend_scale = 2usize; // 2× source resolution
    let blend_w = img_w * blend_scale;
    let blend_h = img_h * blend_scale;
    let blend_total = blend_w * blend_h;

    let transition_radius: i32 = 16;
    let transition_width: f32 = 28.0;

    // Upscale IDs to blend resolution (nearest)
    let blend_ids: Vec<u8> = (0..blend_total)
        .into_par_iter()
        .map(|idx| {
            let bx = idx % blend_w;
            let by = idx / blend_w;
            let sx = bx / blend_scale;
            let sy = by / blend_scale;
            ids[sy * img_w + sx]
        })
        .collect();

    // Compute secondary + blend at blend resolution
    let mut blend_data = vec![(0u8, 0u8); blend_total];

    blend_data
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, result)| {
            let x = (idx % blend_w) as i32;
            let y = (idx / blend_w) as i32;
            let primary = blend_ids[idx];

            let mut nearest_dist_sq = i32::MAX;
            let mut secondary = primary;

            for dy in -transition_radius..=transition_radius {
                let ny = y + dy;
                if ny < 0 || ny >= blend_h as i32 {
                    continue;
                }
                for dx in -transition_radius..=transition_radius {
                    let nx = x + dx;
                    if nx < 0 || nx >= blend_w as i32 {
                        continue;
                    }
                    let nid = blend_ids[(ny as usize) * blend_w + nx as usize];
                    if nid != primary {
                        let d = dx * dx + dy * dy;
                        if d < nearest_dist_sq {
                            nearest_dist_sq = d;
                            secondary = nid;
                        }
                    }
                }
            }

            let blend = if nearest_dist_sq == i32::MAX {
                0u8
            } else {
                let dist = (nearest_dist_sq as f32).sqrt();
                let smooth = 1.0 - (dist / transition_width).clamp(0.0, 1.0);
                (smooth * 128.0) as u8
            };

            *result = (secondary, blend);
        });

    tracing::info!(
        "    Step 3a: Blend at {}x{} in {:?}",
        blend_w,
        blend_h,
        t3.elapsed()
    );

    // Step 4b: Upscale to target RGBA — nearest for IDs, bilinear for blend
    let t3b = std::time::Instant::now();

    let blend_to_target_x = blend_w as f32 / target_width as f32;
    let blend_to_target_y = blend_h as f32 / target_height as f32;

    let mut rgba = vec![0u8; total_pixels * 4];

    rgba.par_chunks_mut(4).enumerate().for_each(|(idx, pixel)| {
        let tx = idx % target_width;
        let ty = idx / target_width;

        // Nearest for IDs
        let bx = ((tx as f32 + 0.5) * blend_to_target_x) as usize;
        let by = ((ty as f32 + 0.5) * blend_to_target_y) as usize;
        let bx = bx.min(blend_w - 1);
        let by = by.min(blend_h - 1);
        let b_idx = by * blend_w + bx;

        let primary = blend_ids[b_idx];
        let (secondary, _) = blend_data[b_idx];

        // Bilinear for blend factor
        let fx = (tx as f32 + 0.5) * blend_to_target_x - 0.5;
        let fy = (ty as f32 + 0.5) * blend_to_target_y - 0.5;
        let x0 = (fx.floor() as i32).clamp(0, blend_w as i32 - 1) as usize;
        let y0 = (fy.floor() as i32).clamp(0, blend_h as i32 - 1) as usize;
        let x1 = (x0 + 1).min(blend_w - 1);
        let y1 = (y0 + 1).min(blend_h - 1);
        let frac_x = (fx - fx.floor()).clamp(0.0, 1.0);
        let frac_y = (fy - fy.floor()).clamp(0.0, 1.0);

        let b00 = blend_data[y0 * blend_w + x0].1 as f32;
        let b10 = blend_data[y0 * blend_w + x1].1 as f32;
        let b01 = blend_data[y1 * blend_w + x0].1 as f32;
        let b11 = blend_data[y1 * blend_w + x1].1 as f32;

        let blend = (b00 * (1.0 - frac_x) * (1.0 - frac_y)
            + b10 * frac_x * (1.0 - frac_y)
            + b01 * (1.0 - frac_x) * frac_y
            + b11 * frac_x * frac_y) as u8;

        pixel[0] = primary * 17;
        pixel[1] = secondary * 17;
        pixel[2] = blend;
        pixel[3] = 255;
    });

    tracing::info!("    Step 3b: Upscaled to RGBA in {:?}", t3b.elapsed());

    rgba
}
