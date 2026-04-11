//! Enriched heightmap generation pipeline.
//!
//! Replaces the simple bilinear upscale with a 7-step pipeline:
//! 1. Masking (land only)
//! 2. Normalisation (coast=0, peaks=1)
//! 3. Gaussian blur (remove u8 quantisation artefacts)
//! 4. Coastal transition via SDF + cliff/plain noise
//! 5. Bilinear upscale ×4
//! 6. FBM detail (meso + micro), amplitude modulated by altitude & biome
//! 7. Quantisation to u16

use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use rayon::prelude::*;
use shared::{BiomeTypeEnum, get_biome_from_color};

// ---------------------------------------------------------------------------
// Biome terrain profile
// ---------------------------------------------------------------------------

/// Per-biome parameters controlling relief generation.
#[derive(Debug, Clone, Copy)]
pub struct BiomeTerrainProfile {
    /// 0.0 = never cliff, 1.0 = normal cliff probability
    pub cliff_allowed: f32,
    /// FBM amplitude multiplier (0 = flat, 1 = normal)
    pub fbm_factor: f32,
    /// Additive offset to coastal transition width (source pixels)
    pub coast_bias: f32,
}

impl BiomeTerrainProfile {
    const fn new(cliff_allowed: f32, fbm_factor: f32, coast_bias: f32) -> Self {
        Self {
            cliff_allowed,
            fbm_factor,
            coast_bias,
        }
    }
}

/// Returns the terrain profile for a given biome.
pub fn biome_terrain_profile(biome: BiomeTypeEnum) -> BiomeTerrainProfile {
    match biome {
        BiomeTypeEnum::Wetland => BiomeTerrainProfile::new(0.0, 0.15, 30.0),
        BiomeTypeEnum::Grassland => BiomeTerrainProfile::new(0.5, 0.6, 10.0),
        BiomeTypeEnum::Savanna => BiomeTerrainProfile::new(0.6, 0.5, 5.0),
        BiomeTypeEnum::Desert => BiomeTerrainProfile::new(0.7, 0.4, 0.0),
        BiomeTypeEnum::ColdDesert => BiomeTerrainProfile::new(0.8, 0.5, 0.0),
        BiomeTypeEnum::TropicalSeasonalForest => BiomeTerrainProfile::new(0.7, 0.8, 5.0),
        BiomeTypeEnum::TropicalRainForest => BiomeTerrainProfile::new(0.6, 0.9, 5.0),
        BiomeTypeEnum::TropicalDeciduousForest => BiomeTerrainProfile::new(0.7, 0.8, 5.0),
        BiomeTypeEnum::TemperateRainForest => BiomeTerrainProfile::new(0.8, 1.0, 0.0),
        BiomeTypeEnum::Taiga => BiomeTerrainProfile::new(0.9, 1.0, 0.0),
        BiomeTypeEnum::Tundra => BiomeTerrainProfile::new(1.0, 1.1, -5.0),
        BiomeTypeEnum::Ice => BiomeTerrainProfile::new(1.0, 1.2, -10.0),
        BiomeTypeEnum::Lake => BiomeTerrainProfile::new(0.3, 0.3, 15.0),
        BiomeTypeEnum::Ocean | BiomeTypeEnum::DeepOcean => {
            BiomeTerrainProfile::new(0.0, 0.0, 0.0)
        }
        BiomeTypeEnum::Undefined => BiomeTerrainProfile::new(0.5, 0.5, 0.0),
    }
}

// ---------------------------------------------------------------------------
// Deterministic noise functions (pure math, no crate needed)
// ---------------------------------------------------------------------------

/// Hash a 2D position to a single f32 in [0, 1).
#[inline]
fn hash21(x: f32, y: f32) -> f32 {
    let mut n = (x * 127.1 + y * 311.7).sin() * 43758.5453;
    n = n - n.floor();
    n
}

/// Value noise with bilinear interpolation, returns [0, 1].
#[inline]
fn noise2d(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;

    // Smoothstep
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);

    let v00 = hash21(ix, iy);
    let v10 = hash21(ix + 1.0, iy);
    let v01 = hash21(ix, iy + 1.0);
    let v11 = hash21(ix + 1.0, iy + 1.0);

    let a = v00 + (v10 - v00) * ux;
    let b = v01 + (v11 - v01) * ux;
    a + (b - a) * uy
}

/// FBM with inter-octave rotation (~37°: cos=0.8, sin=0.6) and ×2 frequency.
/// Returns approximately [0, 1].
#[inline]
fn fbm_rotated(mut x: f32, mut y: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;

    // Rotation matrix: [0.8, 0.6; -0.6, 0.8]
    const COS: f32 = 0.8;
    const SIN: f32 = 0.6;

    for _ in 0..octaves {
        value += amplitude * noise2d(x, y);
        // Rotate
        let rx = COS * x + SIN * y;
        let ry = -SIN * x + COS * y;
        x = rx * 2.0;
        y = ry * 2.0;
        amplitude *= 0.5;
    }
    value
}

// ---------------------------------------------------------------------------
// Smoothstep helper
// ---------------------------------------------------------------------------

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---------------------------------------------------------------------------
// Gaussian blur (separable, kernel 5×5, sigma ≈ 2)
// ---------------------------------------------------------------------------

/// 1D Gaussian kernel for sigma≈2, radius 2 (5 taps).
const GAUSS_KERNEL: [f32; 5] = [0.0625, 0.25, 0.375, 0.25, 0.0625];
const GAUSS_RADIUS: i32 = 2;

/// Separable Gaussian blur on a f32 buffer of size (w, h).
fn gaussian_blur(data: &[f32], w: usize, h: usize) -> Vec<f32> {
    // Horizontal pass
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for k in -GAUSS_RADIUS..=GAUSS_RADIUS {
                let sx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                sum += data[y * w + sx] * GAUSS_KERNEL[(k + GAUSS_RADIUS) as usize];
            }
            tmp[y * w + x] = sum;
        }
    }
    // Vertical pass
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for k in -GAUSS_RADIUS..=GAUSS_RADIUS {
                let sy = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                sum += tmp[sy * w + x] * GAUSS_KERNEL[(k + GAUSS_RADIUS) as usize];
            }
            out[y * w + x] = sum;
        }
    }
    out
}

/// Separable Gaussian blur with configurable sigma on a bool mask (as f32 0/1).
/// Returns blurred f32 values in [0, 1]. Used to smooth coastline staircases.
fn gaussian_blur_mask(mask: &[bool], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    let radius = (sigma * 2.5).ceil() as i32;
    // Build 1D kernel
    let kernel_size = (2 * radius + 1) as usize;
    let mut kernel = vec![0.0f32; kernel_size];
    let mut sum = 0.0f32;
    for i in 0..kernel_size {
        let x = (i as i32 - radius) as f32;
        let v = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel[i] = v;
        sum += v;
    }
    for v in kernel.iter_mut() {
        *v /= sum;
    }

    // Convert mask to f32
    let src: Vec<f32> = mask.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();

    // Horizontal pass
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0f32;
            for k in -radius..=radius {
                let sx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                s += src[y * w + sx] * kernel[(k + radius) as usize];
            }
            tmp[y * w + x] = s;
        }
    }
    // Vertical pass
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0f32;
            for k in -radius..=radius {
                let sy = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                s += tmp[sy * w + x] * kernel[(k + radius) as usize];
            }
            out[y * w + x] = s;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Euclidean Distance Transform (Felzenszwalb & Huttenlocher, 1D separable)
// ---------------------------------------------------------------------------

/// Compute the squared Euclidean distance transform of a binary image.
/// `is_foreground` returns true for "inside" pixels (land).
/// Returns distance from each foreground pixel to the nearest background pixel,
/// with background pixels having distance 0.
fn edt_land_to_water(land_mask: &[bool], w: usize, h: usize) -> Vec<f32> {
    let inf = (w * w + h * h) as f32;

    // Initialise: land → INF, water → 0
    let mut dist_sq: Vec<f32> = land_mask
        .iter()
        .map(|&is_land| if is_land { inf } else { 0.0 })
        .collect();

    // 1D EDT along rows
    let mut v = vec![0i32; w.max(h)];
    let mut z = vec![0.0f32; w.max(h) + 1];
    let mut d = vec![0.0f32; w.max(h)];

    for y in 0..h {
        edt_1d(&mut dist_sq[y * w..(y + 1) * w], w, &mut v, &mut z, &mut d);
    }

    // Transpose
    let mut transposed = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            transposed[x * h + y] = dist_sq[y * w + x];
        }
    }

    // 1D EDT along columns (now rows of transposed)
    for x in 0..w {
        edt_1d(&mut transposed[x * h..(x + 1) * h], h, &mut v, &mut z, &mut d);
    }

    // Transpose back and sqrt
    let mut result = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            result[y * w + x] = transposed[x * h + y].sqrt();
        }
    }
    result
}

/// In-place 1D squared-distance transform (Felzenszwalb-Huttenlocher).
fn edt_1d(f: &mut [f32], n: usize, v: &mut [i32], z: &mut [f32], d: &mut [f32]) {
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    let mut k = 0usize;

    for q in 1..n {
        let q_i = q as i32;
        loop {
            let vk = v[k];
            let s = ((f[q] + (q_i * q_i) as f32) - (f[vk as usize] + (vk * vk) as f32))
                / (2 * (q_i - vk)) as f32;
            if s > z[k] {
                k += 1;
                v[k] = q_i;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
            if k == 0 {
                v[0] = q_i;
                z[0] = f32::NEG_INFINITY;
                z[1] = f32::INFINITY;
                break;
            }
            k -= 1;
        }
    }

    k = 0;
    for q in 0..n {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let vk = v[k];
        let diff = q as i32 - vk;
        d[q] = (diff * diff) as f32 + f[vk as usize];
    }
    f[..n].copy_from_slice(&d[..n]);
}

// ---------------------------------------------------------------------------
// Bilinear upscale (f32 buffer)
// ---------------------------------------------------------------------------

fn upscale_bilinear(src: &[f32], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<f32> {
    let scale_x = sw as f32 / dw as f32;
    let scale_y = sh as f32 / dh as f32;

    (0..dw * dh)
        .into_par_iter()
        .map(|idx| {
            let dx = idx % dw;
            let dy = idx / dw;

            let sx = (dx as f32 + 0.5) * scale_x - 0.5;
            let sy = (dy as f32 + 0.5) * scale_y - 0.5;

            let x0 = sx.floor().max(0.0) as usize;
            let y0 = sy.floor().max(0.0) as usize;
            let x1 = (x0 + 1).min(sw - 1);
            let y1 = (y0 + 1).min(sh - 1);

            let fx = (sx - sx.floor()).clamp(0.0, 1.0);
            let fy = (sy - sy.floor()).clamp(0.0, 1.0);

            let v00 = src[y0 * sw + x0];
            let v10 = src[y0 * sw + x1];
            let v01 = src[y1 * sw + x0];
            let v11 = src[y1 * sw + x1];

            let a = v00 + (v10 - v00) * fx;
            let b = v01 + (v11 - v01) * fx;
            a + (b - a) * fy
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Nearest-neighbour upscale for biome IDs (no interpolation)
// ---------------------------------------------------------------------------

fn upscale_nearest_biome(
    biome_ids: &[BiomeTypeEnum],
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
) -> Vec<BiomeTypeEnum> {
    let scale_x = sw as f32 / dw as f32;
    let scale_y = sh as f32 / dh as f32;

    (0..dw * dh)
        .into_par_iter()
        .map(|idx| {
            let dx = idx % dw;
            let dy = idx / dw;
            let sx = ((dx as f32 + 0.5) * scale_x) as usize;
            let sy = ((dy as f32 + 0.5) * scale_y) as usize;
            let sx = sx.min(sw - 1);
            let sy = sy.min(sh - 1);
            biome_ids[sy * sw + sx]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Nearest-neighbour upscale for bool mask
// ---------------------------------------------------------------------------

fn upscale_nearest_bool(
    mask: &[bool],
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
) -> Vec<bool> {
    let scale_x = sw as f32 / dw as f32;
    let scale_y = sh as f32 / dh as f32;

    (0..dw * dh)
        .into_par_iter()
        .map(|idx| {
            let dx = idx % dw;
            let dy = idx / dw;
            let sx = ((dx as f32 + 0.5) * scale_x) as usize;
            let sy = ((dy as f32 + 0.5) * scale_y) as usize;
            mask[sy.min(sh - 1) * sw + sx.min(sw - 1)]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Main pipeline
// ---------------------------------------------------------------------------

/// Configuration for the enriched heightmap generator.
pub struct EnrichedHeightmapConfig {
    /// Upscale factor (default 4)
    pub upscale: u32,
    /// Seed offset for deterministic noise between worlds
    pub seed_offset: (f32, f32),
}

impl Default for EnrichedHeightmapConfig {
    fn default() -> Self {
        Self {
            upscale: 4,
            seed_offset: (0.0, 0.0),
        }
    }
}

/// Result of the enriched heightmap pipeline.
pub struct EnrichedHeightmapResult {
    /// u16 heightmap values in little-endian byte pairs
    pub data: Vec<u8>,
    /// Output width
    pub width: u32,
    /// Output height
    pub height: u32,
}

/// Generate an enriched heightmap from source maps.
///
/// 7-step pipeline: mask → normalise → blur → coastal SDF transition →
/// upscale ×4 → FBM detail → quantise u16.
///
/// Returns LE u16 bytes suitable for R16Unorm texture.
pub fn generate_enriched_heightmap(
    heightmap_image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    binary_map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    lake_map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    biome_map: &DynamicImage,
    config: &EnrichedHeightmapConfig,
) -> EnrichedHeightmapResult {
    let sw = heightmap_image.width() as usize;
    let sh = heightmap_image.height() as usize;
    let total_src = sw * sh;

    tracing::info!(
        "Enriched heightmap: source {}x{}, upscale ×{}",
        sw,
        sh,
        config.upscale
    );
    let t_start = std::time::Instant::now();

    // -----------------------------------------------------------------------
    // Step 0: Build land mask and biome map at source resolution
    // -----------------------------------------------------------------------
    let land_mask: Vec<bool> = (0..total_src)
        .into_par_iter()
        .map(|idx| {
            let x = idx % sw;
            let y = idx / sw;
            let binary_val = binary_map.get_pixel(x as u32, y as u32)[0];
            let lake_val = lake_map.get_pixel(x as u32, y as u32)[0];
            // Land = binary white AND not lake
            binary_val > 128 && lake_val < 128
        })
        .collect();

    let biome_rgba = biome_map.to_rgba8();
    let biome_ids: Vec<BiomeTypeEnum> = (0..total_src)
        .into_par_iter()
        .map(|idx| {
            let x = idx % sw;
            let y = idx / sw;
            let px = biome_rgba.get_pixel(x as u32, y as u32);
            let rgba = [px[0], px[1], px[2], px[3]];
            get_biome_from_color(&rgba)
        })
        .collect();

    // -----------------------------------------------------------------------
    // Step 1: Masking — non-land pixels → 0
    // -----------------------------------------------------------------------
    let raw: Vec<f32> = (0..total_src)
        .into_par_iter()
        .map(|idx| {
            if land_mask[idx] {
                heightmap_image.as_raw()[idx] as f32
            } else {
                0.0
            }
        })
        .collect();

    // -----------------------------------------------------------------------
    // Step 2: Normalisation — find land min/max, remap to [0, 1]
    // -----------------------------------------------------------------------
    let (min_land, max_land) = {
        let mut mn = 255.0f32;
        let mut mx = 0.0f32;
        for (i, &v) in raw.iter().enumerate() {
            if land_mask[i] {
                mn = mn.min(v);
                mx = mx.max(v);
            }
        }
        if mx <= mn {
            mx = mn + 1.0;
        }
        (mn, mx)
    };

    let range = max_land - min_land;
    let mut normalised: Vec<f32> = raw
        .par_iter()
        .enumerate()
        .map(|(i, &v)| {
            if land_mask[i] {
                ((v - min_land) / range).clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .collect();

    tracing::info!(
        "  Step 1-2: masked & normalised (land range {:.0}–{:.0})",
        min_land,
        max_land
    );

    // -----------------------------------------------------------------------
    // Step 3: Gaussian blur + re-mask
    // -----------------------------------------------------------------------
    normalised = gaussian_blur(&normalised, sw, sh);
    // Re-apply mask (blur bleeds water into land)
    for (i, val) in normalised.iter_mut().enumerate() {
        if !land_mask[i] {
            *val = 0.0;
        }
    }
    tracing::info!("  Step 3: Gaussian blur applied");

    // -----------------------------------------------------------------------
    // Step 4: Coastal transition via SDF
    // Smooth the land mask before EDT to round pixel staircases into curves.
    // The sharp mask is kept for the final zero-forcing (step 5b/7).
    // -----------------------------------------------------------------------
    let t_sdf = std::time::Instant::now();
    let smoothed_blur = gaussian_blur_mask(&land_mask, sw, sh, 5.0);
    let land_mask_smoothed: Vec<bool> = smoothed_blur.iter().map(|&v| v > 0.5).collect();
    let sdf = edt_land_to_water(&land_mask_smoothed, sw, sh);
    tracing::info!("  Step 4a: EDT computed (smoothed mask, sigma=5) in {:?}", t_sdf.elapsed());

    // Apply coastal transition with cliff/plain variation
    let so = config.seed_offset;
    normalised
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, height)| {
            if !land_mask[idx] {
                return;
            }

            let x = idx % sw;
            let y = idx / sw;
            let dist = sdf[idx]; // distance to water in source pixels

            let profile = biome_terrain_profile(biome_ids[idx]);

            // Cliff noise at source scale
            let cliff_noise = fbm_rotated(
                x as f32 * 0.008 + so.0,
                y as f32 * 0.008 + so.1,
                4,
            );

            let adjusted_noise = cliff_noise + (1.0 - profile.cliff_allowed) * 0.8;

            let effective_width = if adjusted_noise < 0.38 {
                // Cliff: narrow transition (2-5 px)
                2.0 + (adjusted_noise / 0.38) * 3.0 + profile.coast_bias * 0.1
            } else {
                // Plain: wide transition (25-80 px)
                let t = (adjusted_noise - 0.38) / 0.62;
                25.0 + t * 55.0 + profile.coast_bias
            };

            let effective_width = effective_width.max(1.0);
            *height *= smoothstep(0.0, effective_width, dist);
        });

    tracing::info!("  Step 4b: coastal transition applied");

    // -----------------------------------------------------------------------
    // Step 5: Upscale ×4 (bilinear)
    // -----------------------------------------------------------------------
    let dw = sw * config.upscale as usize;
    let dh = sh * config.upscale as usize;

    let t_up = std::time::Instant::now();
    let upscaled = upscale_bilinear(&normalised, sw, sh, dw, dh);
    tracing::info!(
        "  Step 5: upscaled to {}x{} in {:?}",
        dw,
        dh,
        t_up.elapsed()
    );

    // Upscale biome IDs (nearest neighbour) and land mask (bilinear + threshold
    // so the mask boundary matches the bilinear height boundary and prevents
    // non-zero values from bleeding into ocean pixels).
    let biome_ids_up = upscale_nearest_biome(&biome_ids, sw, sh, dw, dh);
    let land_mask_f32: Vec<f32> = land_mask
        .iter()
        .map(|&b| if b { 1.0 } else { 0.0 })
        .collect();
    let land_mask_up_f32 = upscale_bilinear(&land_mask_f32, sw, sh, dw, dh);
    let land_mask_up: Vec<bool> = land_mask_up_f32
        .into_par_iter()
        .map(|v| v > 0.5)
        .collect();

    // -----------------------------------------------------------------------
    // Step 5b: Re-mask upscaled data — bilinear upscale bleeds land into
    // water at the coast boundary. Zero out any water pixels.
    // -----------------------------------------------------------------------
    let upscaled: Vec<f32> = upscaled
        .into_par_iter()
        .enumerate()
        .map(|(idx, v)| if land_mask_up[idx] { v } else { 0.0 })
        .collect();

    // -----------------------------------------------------------------------
    // Step 6: FBM detail (meso + micro), parallelised
    // -----------------------------------------------------------------------
    let t_fbm = std::time::Instant::now();
    let enriched: Vec<f32> = upscaled
        .into_par_iter()
        .enumerate()
        .map(|(idx, base_height)| {
            if !land_mask_up[idx] {
                return 0.0;
            }

            let x = idx % dw;
            let y = idx / dw;
            let px = x as f32 + so.0 * 100.0;
            let py = y as f32 + so.1 * 100.0;

            let profile = biome_terrain_profile(biome_ids_up[idx]);

            // Altitude-dependent amplitude: low near coast, full at mountains
            let altitude_factor = smoothstep(0.03, 0.40, base_height);

            let biome_fbm_factor = profile.fbm_factor;
            let amplitude_mod = altitude_factor * biome_fbm_factor;

            // Meso noise (collines): freq 0.03, 5 octaves, max amplitude 0.10
            let meso = fbm_rotated(px * 0.03, py * 0.03, 5);
            let meso_contrib = (meso - 0.5) * 2.0 * 0.10 * amplitude_mod;

            // Micro noise (texture): freq 0.10, 3 octaves, max amplitude 0.025
            let micro = fbm_rotated(
                px * 0.10 + 777.7,
                py * 0.10 + 333.3,
                3,
            );
            let micro_contrib = (micro - 0.5) * 2.0 * 0.025 * amplitude_mod;

            let result = base_height + meso_contrib + micro_contrib;
            result
        })
        .collect();

    tracing::info!("  Step 6: FBM detail added in {:?}", t_fbm.elapsed());

    // -----------------------------------------------------------------------
    // Step 7: Quantise to u16 LE bytes
    // -----------------------------------------------------------------------
    let mut out_bytes: Vec<u8> = vec![0u8; dw * dh * 2];
    enriched
        .par_iter()
        .zip(land_mask_up.par_iter())
        .enumerate()
        .for_each(|(idx, (&val, &is_land))| {
            let u16_val = if is_land {
                // Land must be > 0u16. Clamp to [1, 65535].
                let v = (val.clamp(0.0, 1.0) * 65535.0).round() as u16;
                v.max(1) // ensure land is never 0
            } else {
                0u16 // ocean is exactly 0
            };
            let bytes = u16_val.to_le_bytes();
            let offset = idx * 2;
            // Safety: each index writes to its own 2-byte slot
            unsafe {
                let ptr = out_bytes.as_ptr() as *mut u8;
                *ptr.add(offset) = bytes[0];
                *ptr.add(offset + 1) = bytes[1];
            }
        });

    // -----------------------------------------------------------------------
    // Step 8: Vertical flip — align with Bevy's Y-up coordinate system.
    // The biome texture is also flipped before being sent to the client
    // (see terrain_mesh_data.rs generate_globals). Without this flip the
    // heightmap is Y-inverted relative to the biome and the visual terrain.
    // -----------------------------------------------------------------------
    {
        let row_bytes = dw * 2;
        let half = dh / 2;
        for y in 0..half {
            let top = y * row_bytes;
            let bot = (dh - 1 - y) * row_bytes;
            for x in 0..row_bytes {
                out_bytes.swap(top + x, bot + x);
            }
        }
    }
    tracing::info!("  Step 8: vertical flip applied");

    tracing::info!(
        "Enriched heightmap {}x{} generated in {:?} ({:.1} MB)",
        dw,
        dh,
        t_start.elapsed(),
        (dw * dh * 2) as f64 / 1_048_576.0
    );

    EnrichedHeightmapResult {
        data: out_bytes,
        width: dw as u32,
        height: dh as u32,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Luma, Rgba, RgbaImage};

    /// Create a small test world: 16×16, land in center 8×8, water around edges.
    fn make_test_maps(
        size: u32,
    ) -> (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
        DynamicImage,
    ) {
        let w = size;
        let h = size;
        let mut heightmap = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h);
        let mut binary = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h);
        let lake = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h); // no lakes
        let mut biome = RgbaImage::new(w, h);

        let quarter = size / 4;
        let three_quarter = 3 * size / 4;

        for y in 0..h {
            for x in 0..w {
                if x >= quarter && x < three_quarter && y >= quarter && y < three_quarter {
                    // Land
                    binary.put_pixel(x, y, Luma([255]));
                    // Height varies: higher in center
                    let cx = (x as f32 - w as f32 / 2.0).abs();
                    let cy = (y as f32 - h as f32 / 2.0).abs();
                    let dist = (cx.max(cy)) / (size as f32 / 4.0);
                    let h_val = ((1.0 - dist) * 200.0 + 30.0) as u8;
                    heightmap.put_pixel(x, y, Luma([h_val]));
                    // Grassland biome
                    biome.put_pixel(x, y, Rgba([200, 214, 143, 255]));
                } else {
                    // Water
                    binary.put_pixel(x, y, Luma([0]));
                    heightmap.put_pixel(x, y, Luma([0]));
                    biome.put_pixel(x, y, Rgba([0, 15, 30, 255])); // Ocean
                }
            }
        }

        (heightmap, binary, lake, DynamicImage::ImageRgba8(biome))
    }

    #[test]
    fn ocean_pixels_stay_zero() {
        let (hm, bin, lake, biome) = make_test_maps(32);
        let config = EnrichedHeightmapConfig {
            upscale: 2,
            seed_offset: (0.0, 0.0),
        };
        let result = generate_enriched_heightmap(&hm, &bin, &lake, &biome, &config);

        let dw = result.width as usize;
        let dh = result.height as usize;
        assert_eq!(result.data.len(), dw * dh * 2);

        // Check water pixels (edges) are 0
        for y in 0..4 {
            for x in 0..dw {
                let idx = (y * dw + x) * 2;
                let val = u16::from_le_bytes([result.data[idx], result.data[idx + 1]]);
                assert_eq!(val, 0, "Water pixel at ({}, {}) should be 0", x, y);
            }
        }
    }

    #[test]
    fn noise_is_deterministic() {
        let (hm, bin, lake, biome) = make_test_maps(16);
        let config = EnrichedHeightmapConfig {
            upscale: 2,
            seed_offset: (42.0, 7.0),
        };
        let r1 = generate_enriched_heightmap(&hm, &bin, &lake, &biome, &config);
        let r2 = generate_enriched_heightmap(&hm, &bin, &lake, &biome, &config);
        assert_eq!(r1.data, r2.data, "Same seed must produce identical output");
    }

    #[test]
    fn land_pixels_never_zero() {
        let (hm, bin, lake, biome) = make_test_maps(32);
        let config = EnrichedHeightmapConfig {
            upscale: 2,
            seed_offset: (0.0, 0.0),
        };
        let result = generate_enriched_heightmap(&hm, &bin, &lake, &biome, &config);

        let dw = result.width as usize;
        let mid = dw / 2;

        // Check a well-interior land pixel
        let idx = (mid * dw + mid) * 2;
        let val = u16::from_le_bytes([result.data[idx], result.data[idx + 1]]);
        assert!(val > 0, "Interior land pixel should be > 0, got {}", val);
    }

    #[test]
    fn wetland_lower_variance_than_taiga() {
        // Use a large enough map so interior pixels are far from the coast
        // (SDF > 80 px), ensuring the coastal transition doesn't flatten everything
        let size = 256u32;
        let w = size;
        let h = size;
        let mut heightmap = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h);
        let mut binary = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h);
        let lake = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h);

        // All land, varying height (mountain in center)
        for y in 0..h {
            for x in 0..w {
                binary.put_pixel(x, y, Luma([255]));
                let cx = (x as f32 - w as f32 / 2.0).abs() / (w as f32 / 2.0);
                let cy = (y as f32 - h as f32 / 2.0).abs() / (h as f32 / 2.0);
                let d = (cx * cx + cy * cy).sqrt().min(1.0);
                let h_val = ((1.0 - d) * 200.0 + 30.0) as u8;
                heightmap.put_pixel(x, y, Luma([h_val]));
            }
        }

        // Wetland biome
        let mut biome_wetland = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                biome_wetland.put_pixel(x, y, Rgba([11, 145, 49, 255]));
            }
        }

        // Taiga biome
        let mut biome_taiga = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                biome_taiga.put_pixel(x, y, Rgba([75, 107, 50, 255]));
            }
        }

        let config = EnrichedHeightmapConfig {
            upscale: 2,
            seed_offset: (0.0, 0.0),
        };

        let r_wetland = generate_enriched_heightmap(
            &heightmap,
            &binary,
            &lake,
            &DynamicImage::ImageRgba8(biome_wetland),
            &config,
        );
        let r_taiga = generate_enriched_heightmap(
            &heightmap,
            &binary,
            &lake,
            &DynamicImage::ImageRgba8(biome_taiga),
            &config,
        );

        // Compute variance only on interior pixels (skip edge 25% to avoid coast effects)
        fn interior_variance(data: &[u8], width: usize, height: usize) -> f64 {
            let margin_x = width / 4;
            let margin_y = height / 4;
            let mut vals = Vec::new();
            for y in margin_y..(height - margin_y) {
                for x in margin_x..(width - margin_x) {
                    let idx = (y * width + x) * 2;
                    let v = u16::from_le_bytes([data[idx], data[idx + 1]]) as f64 / 65535.0;
                    vals.push(v);
                }
            }
            let n = vals.len() as f64;
            let mean = vals.iter().sum::<f64>() / n;
            vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
        }

        let var_w = interior_variance(
            &r_wetland.data,
            r_wetland.width as usize,
            r_wetland.height as usize,
        );
        let var_t = interior_variance(
            &r_taiga.data,
            r_taiga.width as usize,
            r_taiga.height as usize,
        );
        assert!(
            var_w < var_t,
            "Wetland variance ({:.6}) should be < Taiga variance ({:.6})",
            var_w,
            var_t
        );
    }

    #[test]
    fn enriched_has_more_variance_than_bilinear() {
        let (hm, bin, lake, biome) = make_test_maps(128);
        let config = EnrichedHeightmapConfig {
            upscale: 2,
            seed_offset: (0.0, 0.0),
        };
        let enriched = generate_enriched_heightmap(&hm, &bin, &lake, &biome, &config);

        let dw = enriched.width as usize;
        let dh = enriched.height as usize;

        // Compare interior variance only (skip coast effects)
        fn interior_var_u16(data: &[u8], w: usize, h: usize) -> f64 {
            let margin_x = w / 4;
            let margin_y = h / 4;
            let mut vals = Vec::new();
            for y in margin_y..(h - margin_y) {
                for x in margin_x..(w - margin_x) {
                    let idx = (y * w + x) * 2;
                    let v = u16::from_le_bytes([data[idx], data[idx + 1]]) as f64 / 65535.0;
                    vals.push(v);
                }
            }
            let n = vals.len() as f64;
            let mean = vals.iter().sum::<f64>() / n;
            vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
        }

        let var_enriched = interior_var_u16(&enriched.data, dw, dh);

        // The enriched map must have non-trivial variance in the interior
        // (FBM adds detail beyond the smooth source)
        assert!(
            var_enriched > 1e-6,
            "Enriched interior variance ({:.8}) should be non-trivial",
            var_enriched
        );
    }
}
