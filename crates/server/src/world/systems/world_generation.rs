use crate::database::client::DatabaseTables;
use crate::database::tables;
use crate::world;
use image::{ImageBuffer, Luma};
use crate::world::components::BiomeMeshData;
use crate::world::components::NaturalBuildingGenerator;
use crate::world::components::TerrainMeshData;
use crate::world::components::generate_global_sdf;
use crate::world::components::generate_ocean_data;
use crate::world::resources::{
    WorldConfig, WorldGlobalState, WorldMaps, WorldSource, WorldSourceKind, YmirMap,
};
use bevy::prelude::*;
use hexx::HexOrientation;
use crate::world::components::{biome_blend_rgba_from_ids, resolve_biome};
use image::{DynamicImage, Rgba};
use shared::BiomeTypeEnum;
use shared::get_biome_color;
use shared::BuildingData;
use shared::GameState;
use shared::TerrainChunkId;
use shared::constants;
use shared::grid::GridConfig;
use sqlx::Row;

pub fn setup_grid_config() -> GridConfig {
    let radius = constants::HEX_SIZE;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(constants::HEX_RATIO.x, constants::HEX_RATIO.y);
    let chunk_size = 3u8;
    let grid_config = GridConfig::new(radius, orientation, ratio, chunk_size);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
    grid_config
}

/// Generate global data only (SDF, biome, heightmap, ocean).
/// Returns WorldGlobalState to keep in memory for on-demand chunk generation.
pub async fn generate_world_globals(
    map_name: &str,
    db_tables: &DatabaseTables,
    source_kind: WorldSourceKind,
) -> WorldGlobalState {
    tracing::info!(
        "=== GENERATING WORLD GLOBALS : {} (source: {}) ===",
        map_name,
        source_kind.as_str()
    );
    let start = std::time::Instant::now();

    let grid_config = setup_grid_config();
    let scale = Vec2::splat(100.);

    let source = WorldSource::load(source_kind, map_name, 12345)
        .unwrap_or_else(|e| panic!("Failed to load world source ({}): {e}", source_kind.as_str()));

    let (mut global_state, terrain_global_data) = match source {
        WorldSource::AzgaarPng(maps) => {
            let (mut global_state, terrain_global_data) = TerrainMeshData::generate_globals(
                map_name,
                &maps.binary_map,
                &maps.lake_map,
                Some(&maps.heightmap),
                Some(&maps.biome_map),
                &scale,
            );

            // Cache source biome RGBA for per-chunk cell sampling
            let source_biome_flipped = image::imageops::flip_vertical(&maps.biome_map.to_rgba8());
            tracing::info!(
                "✓ Source biome RGBA prepared ({}x{})",
                source_biome_flipped.width(),
                source_biome_flipped.height()
            );

            let source_lake_flipped = image::imageops::flip_vertical(&maps.lake_map);
            tracing::info!(
                "✓ Source lake map prepared ({}x{})",
                source_lake_flipped.width(),
                source_lake_flipped.height()
            );
            global_state.source_lake_flipped = source_lake_flipped;

            global_state.source_biome_flipped_rgba = Some(source_biome_flipped);
            global_state.maps = Some(maps);
            global_state.grid_config = Some(grid_config);
            global_state.build_effective_binary(true);
            (global_state, terrain_global_data)
        }
        WorldSource::Ymir(ymir) => build_ymir_globals(map_name, &ymir, grid_config, scale),
    };

    // Save terrain global data (biome + heightmap textures for client)
    if let Some(ref global_data) = terrain_global_data {
        db_tables
            .terrain_global_data
            .save_terrain_global_data(global_data.clone())
            .await
            .expect("Failed to save terrain global data");
        tracing::info!(
            "✓ Terrain global data saved (biome {}x{}, heightmap {}x{})",
            global_data.biome_width,
            global_data.biome_height,
            global_data.heightmap_width,
            global_data.heightmap_height
        );
    }

    // Generate ocean data from source at capped resolution
    tracing::info!("=== PREPARING OCEAN DATA ===");
    let global_maps = global_state.maps.as_ref().unwrap();

    let ocean_max_dim = 4096u32;
    let src_w = global_maps.binary_map.width();
    let src_h = global_maps.binary_map.height();
    let ocean_scale = if (src_w as f32 * global_state.scale.x) as u32 > ocean_max_dim {
        ocean_max_dim as f32 / src_w.max(src_h) as f32
    } else {
        global_state.scale.x
    };

    tracing::info!(
        "Ocean source: {}x{}, scale: {:.1} (capped from {:.1})",
        src_w,
        src_h,
        ocean_scale,
        global_state.scale.x
    );

    let ocean_src = global_state.effective_binary_smoothed.as_ref()
        .or(global_state.effective_binary.as_ref());
    let ocean_binary = if let Some(eff) = ocean_src {
        // Effective binary is already clean 0/255 and Y-flipped.
        // Resize to target ocean resolution.
        let target_w = eff.width().min(ocean_max_dim);
        let ratio = eff.height() as f32 / eff.width() as f32;
        let target_h = (target_w as f32 * ratio).round() as u32;
        let mut resized = image::imageops::resize(
            eff,
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        );
        // Re-threshold after resize (Lanczos creates intermediate values)
        resized.iter_mut().for_each(|p| {
            *p = if *p > 128 { 255 } else { 0 };
        });
        tracing::info!(
            "Ocean binary from effective_binary: {}x{} → {}x{}",
            eff.width(), eff.height(), target_w, target_h
        );
        resized
    } else {
        // Fallback: original binary map pipeline
        let ocean_binary_flipped = image::imageops::flip_vertical(&global_maps.binary_map);
        let mut resized = image::imageops::resize(
            &ocean_binary_flipped,
            (src_w as f32 * ocean_scale) as u32,
            (src_h as f32 * ocean_scale) as u32,
            image::imageops::FilterType::Lanczos3,
        );
        resized.iter_mut().for_each(|p| {
            *p = if *p > 178 { 255 } else { 0 };
        });
        resized
    };

    let ocean_heightmap = image::imageops::resize(
        &global_maps.heightmap,
        ocean_binary.width(),
        ocean_binary.height(),
        image::imageops::FilterType::Lanczos3,
    );

    let ocean_binary_w = ocean_binary.width();
    let ocean_binary_h = ocean_binary.height();

    // Pass ocean image dimensions as "world" for correct SDF search_radius.
    // The actual mesh size is computed from n_chunk_x * 64 in OceanData,
    // so the client mesh covers the real world regardless.
    let mut ocean_data = generate_ocean_data(
        map_name.to_string(),
        &ocean_binary,
        &ocean_heightmap,
        global_state.n_chunk_x,
        global_state.n_chunk_y,
        ocean_binary_w as f32,
        ocean_binary_h as f32,
    );

    // Override world dimensions with real world size (not capped ocean image size)
    ocean_data.world_width = global_state.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    ocean_data.world_height = global_state.n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    // LL-B (Ymir path): replace the pixel-mask ocean SDF with a signed distance
    // field derived from the real vector coastline, at the same resolution and
    // max_distance so the shader beach_start/beach_end thresholds still line up.
    if !global_state.coastline_cells.is_empty() {
        if let Some(ref eff) = global_state.effective_binary {
            let coastal = world::components::coastal_signed_distance_field(
                &global_state.coastline_cells,
                global_state.enriched_heightmap_width,
                global_state.enriched_heightmap_height,
                ocean_data.width,
                ocean_data.height,
                ocean_data.max_distance,
                eff,
            );
            ocean_data.sdf_values = coastal;
            tracing::info!(
                "✓ Ocean SDF from Ymir vector coastline ({} polylines → {}x{} SDF, max_distance={})",
                global_state.coastline_cells.len(),
                ocean_data.width,
                ocean_data.height,
                ocean_data.max_distance,
            );
        }
    }

    tracing::info!(
        "🌊 Saving ocean : (texture {}x{}, world_width={}, world_height={})",
        ocean_data.width,
        ocean_data.height,
        ocean_data.world_width,
        ocean_data.world_height
    );

    db_tables
        .ocean_data
        .save_ocean_data(ocean_data)
        .await
        .expect("Failed to save ocean data");

    // Generate lake data from lakemap (mask + SDF)
    tracing::info!("=== PREPARING LAKE DATA ===");
    let lake_src = &global_state.maps.as_ref().unwrap().lake_map;
    let lake_w = lake_src.width();
    let lake_h = lake_src.height();

    let lake_max_dim = 2048u32;
    let (lake_target_w, lake_target_h) = if lake_w > lake_max_dim || lake_h > lake_max_dim {
        let ratio = lake_h as f32 / lake_w as f32;
        if lake_w >= lake_h {
            (lake_max_dim, (lake_max_dim as f32 * ratio).round() as u32)
        } else {
            ((lake_max_dim as f32 / ratio).round() as u32, lake_max_dim)
        }
    } else {
        (lake_w, lake_h)
    };

    // Build effective lake mask: water pixels (from enriched heightmap) that are
    // also in the lake source map. This aligns the lake boundary with the
    // enriched heightmap's coastal slope.
    // Lake pipeline — symmetric with ocean:
    // 1. Build lake binary from effective_binary_smoothed + lake_src
    // 2. Resize to capped resolution (4096 for binary, 2048 for SDF)
    // 3. Pass image dims as "world" for correct SDF search_radius
    // 4. Derive mask from SDF (no separate mask computation)
    let lake_eff_src = global_state.effective_binary_smoothed.as_ref()
        .or(global_state.effective_binary.as_ref());
    // Ymir: inland water (effective class 2 = lake_mask or enclosed below-sea) is
    // the authoritative lake source, independent of effective_binary (which marks
    // class 2 as land so the ocean never covers it). Its grid == enriched dims ==
    // effective_binary dims, same display orientation.
    let wc_flipped = global_state.water_class_flipped.as_deref();
    let lake_binary = if let Some(eff) = lake_eff_src {
        let bw = eff.width();
        let bh = eff.height();

        // Build lake binary (SDF convention: lake=0 water, land=255).
        let eff_lake = ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(bw, bh, |x, y| {
            let is_lake = if let Some(wc) = wc_flipped {
                let idx = (y as usize) * (bw as usize) + (x as usize);
                wc.get(idx).copied() == Some(2)
            } else {
                // Azgaar: water pixels that are also in the lake source map.
                let is_water = eff.get_pixel(x, y)[0] == 0;
                let wx = x as f32 / bw as f32;
                let wy = y as f32 / bh as f32;
                let lx = (wx * lake_w as f32).min(lake_w as f32 - 1.0) as u32;
                let ly = ((1.0 - wy) * (lake_h as f32 - 1.0)) as u32;
                is_water && lake_src.get_pixel(lx, ly)[0] > 128
            };
            Luma([if is_lake { 0u8 } else { 255u8 }])
        });

        // Resize to capped resolution (same as ocean binary cap)
        let lake_cap = 4096u32;
        let ratio = bh as f32 / bw as f32;
        let (target_w, target_h) = if bw >= bh {
            (lake_cap.min(bw), (lake_cap.min(bw) as f32 * ratio).round() as u32)
        } else {
            ((lake_cap.min(bh) as f32 / ratio).round() as u32, lake_cap.min(bh))
        };
        let mut resized = image::imageops::resize(
            &eff_lake, target_w, target_h, image::imageops::FilterType::Lanczos3,
        );
        resized.iter_mut().for_each(|p| { *p = if *p > 128 { 255 } else { 0 }; });

        tracing::info!(
            "Lake binary from effective_binary: {}x{} → {}x{}",
            bw, bh, target_w, target_h
        );
        resized
    } else {
        // Fallback: original lake pipeline
        let lake_upscale_dim = 4096u32;
        let lake_ratio = lake_h as f32 / lake_w as f32;
        let (luw, luh) = if lake_w >= lake_h {
            (lake_upscale_dim, (lake_upscale_dim as f32 * lake_ratio).round() as u32)
        } else {
            ((lake_upscale_dim as f32 / lake_ratio).round() as u32, lake_upscale_dim)
        };
        let lake_upscaled = image::imageops::resize(
            lake_src, luw, luh, image::imageops::FilterType::Lanczos3,
        );
        let lake_flipped = image::imageops::flip_vertical(&lake_upscaled);
        let mut lake_inv = lake_flipped;
        lake_inv.iter_mut().for_each(|p| { *p = if *p > 178 { 0 } else { 255 }; });
        lake_inv
    };

    // SDF at capped resolution (2048 max, same as ocean)
    let lake_binary_w = lake_binary.width();
    let lake_binary_h = lake_binary.height();
    let max_lake_sdf_dim = 2048usize;
    let (lake_sdf_w, lake_sdf_h) = if lake_binary_w as usize > max_lake_sdf_dim
        || lake_binary_h as usize > max_lake_sdf_dim
    {
        let ratio = lake_binary_h as f32 / lake_binary_w as f32;
        if lake_binary_w >= lake_binary_h {
            (max_lake_sdf_dim, (max_lake_sdf_dim as f32 * ratio).round() as usize)
        } else {
            ((max_lake_sdf_dim as f32 / ratio).round() as usize, max_lake_sdf_dim)
        }
    } else {
        (lake_binary_w as usize, lake_binary_h as usize)
    };
    let lake_max_distance = 15.0f32; // same as ocean

    tracing::info!(
        "Generating lake SDF {}x{} from binary {}x{} (max_distance: {})",
        lake_sdf_w, lake_sdf_h, lake_binary_w, lake_binary_h, lake_max_distance
    );

    // Pass image dims as "world" — same trick as ocean for correct search_radius
    let lake_sdf = generate_global_sdf(
        &lake_binary,
        lake_sdf_w,
        lake_sdf_h,
        lake_binary_w as f32,
        lake_binary_h as f32,
        lake_max_distance,
    );

    tracing::info!("✓ Lake SDF generated: {} bytes", lake_sdf.len());

    // Derive mask from SDF (threshold at 128 = shore) instead of separate computation
    let mask_values: Vec<u8> = lake_sdf.iter().map(|&v| if v < 128 { 255 } else { 0 }).collect();

    let lake_world_width = global_state.n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let lake_world_height = global_state.n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    let lake_data = shared::LakeData {
        name: map_name.to_string(),
        width: lake_sdf_w,     // mask = same resolution as SDF
        height: lake_sdf_h,
        mask_values,
        sdf_width: lake_sdf_w,
        sdf_height: lake_sdf_h,
        sdf_values: lake_sdf,
        world_width: lake_world_width,
        world_height: lake_world_height,
        generated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    tracing::info!(
        "✓ Lake data: mask {}x{}, SDF {}x{} ({:.2} KB total)",
        lake_target_w,
        lake_target_h,
        lake_sdf_w,
        lake_sdf_h,
        (lake_data.mask_values.len() + lake_data.sdf_values.len()) as f64 / 1024.0
    );

    db_tables
        .lake_data
        .save_lake_data(lake_data)
        .await
        .expect("Failed to save lake data");

    tracing::info!("✓ World globals generated in {:?}", start.elapsed());
    global_state
}

/// Build world globals from a Ymir metric height field (LL-A).
///
/// Consumes the metric u16 `height` layer directly. The enriched heightmap
/// pipeline (`generate_enriched_heightmap`, steps 1–8 — in particular step 4
/// coastal-SDF and step 6 FBM detail) is **not** run: `heightmap_values` is the
/// Ymir u16 copied verbatim (Y-flipped to Bevy Y-up), preserving full range and
/// bathymetry with no u8 requantise. Land/sea is derived from
/// `metres > sea_level_m` (temporary; coastline is LL-B, real biomes are LL-C).
///
/// River channel radius in cells by Strahler order (LL-E), for the per-cell
/// River biome / Riverbank shore. 1 cell ≈ `metres_per_cell` (~500 m) — kept
/// narrow and aligned to (slightly wider than) the client's rendered channel
/// width so the Riverbank forms a contiguous ring hugging the visible water,
/// rather than a wide zone leaving inner river cells with no bank.
fn strahler_radius_cells(order: u32) -> i32 {
    match order {
        0..=5 => 0,
        _ => 1,
    }
}

/// Stamp river segment polylines into the display-oriented per-cell biome `grid`
/// as `River`, widening with Strahler order. Ymir points are cell space (y=0 =
/// south) → Y-flipped to the grid's north-up orientation. Skips ocean cells so a
/// river never overwrites the sea near its mouth.
fn rasterize_rivers(rivers: &shared::rivers::RiverNetwork, grid: &mut [u8], w: usize, h: usize) {
    let river_id = BiomeTypeEnum::River.to_id() as u8;
    let ocean = BiomeTypeEnum::Ocean.to_id() as u8;
    let deep = BiomeTypeEnum::DeepOcean.to_id() as u8;
    for seg in &rivers.segments {
        let r = strahler_radius_cells(seg.strahler_order);
        for p in &seg.points {
            let cx = p[0].round() as i32;
            let gy = h as i32 - 1 - p[1].round() as i32; // Y-flip
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy > r * r {
                        continue;
                    }
                    let x = cx + dx;
                    let y = gy + dy;
                    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                        continue;
                    }
                    let idx = (y as usize) * w + (x as usize);
                    if grid[idx] != ocean && grid[idx] != deep {
                        grid[idx] = river_id;
                    }
                }
            }
        }
    }
}

fn build_ymir_globals(
    map_name: &str,
    ymir: &YmirMap,
    grid_config: GridConfig,
    scale: Vec2,
) -> (WorldGlobalState, Option<shared::TerrainGlobalData>) {
    let hf = &ymir.height;
    let w = hf.width as usize;
    let h = hf.height as usize;
    let sea_level_norm = hf.sea_level_norm;
    let sea_level_m = ymir.manifest.continent.sea_level_m;

    // LL-C: real biome from biome.u8 (+temperature+height) when present, else the
    // LL-A height-only placeholder (Grassland land / Ocean water).
    let has_biome = !ymir.biome.is_empty();

    // Y-flipped R16 heightmap + biome texture + flipped source buffers used by
    // the per-chunk sampler. Ymir authors y=0 = south; the enriched path flips
    // vertically at its step 8, so we match that orientation.
    let mut heightmap_values = vec![0u8; w * h * 2];
    let mut biome_values = vec![0u8; w * h * 4];
    let mut source_binary_flipped =
        image::ImageBuffer::<Luma<u8>, Vec<u8>>::new(hf.width, hf.height);
    let mut source_biome_flipped_rgba =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::new(hf.width, hf.height);
    // Resolved per-cell biome ids (Y-flipped), fed to sample_biome_for_chunk in
    // place of find_closest_biome on the Ymir path. Empty (→ None) if no biome.u8.
    let mut biome_ids: Vec<u8> = if has_biome { vec![0u8; w * h] } else { Vec::new() };

    // LL-D: derive a per-cell inland-water class (0 land / 1 ocean / 2 inland
    // water) from Ymir's water_class (ocean vs enclosed-below-sea) folded with
    // lake_mask (real drainage lakes, which sit on land at elevation so are
    // water_class 0). A Y-flipped copy feeds the effective_binary and lake source
    // built below. `1` (ocean) is the only non-terrain class.
    let has_water_class = !ymir.water_class.is_empty();
    let has_lake_mask = !ymir.lake_mask.is_empty();
    let has_inland = has_water_class || has_lake_mask;
    let mut water_class_flipped: Vec<u8> =
        if has_inland { vec![0u8; w * h] } else { Vec::new() };

    for y in 0..h {
        let src_row = (h - 1 - y) * w; // vertical flip; raw Ymir row = h-1-y
        let raw_y = (h - 1 - y) as u32;
        let dst_row = y * w;
        for x in 0..w {
            let v = hf.data[src_row + x];
            let height_m = hf.metres_of_u16(v);

            // Land/sea authority: water_class (ocean vs enclosed-below-sea) when
            // present, else the calibrated height threshold (0/1). A lake_mask cell
            // is inland water (class 2) even above sea level. Ocean (class 1) wins
            // and is the only non-terrain class — inland water keeps a terrain mesh
            // (lake bank) and is drawn by the lake pipeline, so it counts as "land"
            // for the ocean binary and the chunk source binary.
            let lake_id = ymir.lake_id_at(x as u32, raw_y).unwrap_or(0);
            let raw_class = if has_water_class {
                ymir.water_class[src_row + x]
            } else if height_m <= sea_level_m {
                1
            } else {
                0
            };
            let class = if raw_class == 1 {
                1
            } else if lake_id != 0 || raw_class == 2 {
                2
            } else {
                0
            };
            let is_land = class != 1;
            if has_inland {
                water_class_flipped[dst_row + x] = class;
            }

            let hb = v.to_le_bytes();
            let ho = (dst_row + x) * 2;
            heightmap_values[ho] = hb[0];
            heightmap_values[ho + 1] = hb[1];

            let biome = if has_biome {
                let whittaker_id = ymir.biome[src_row + x];
                let temp_c = ymir.temperature_c_at(x as u32, raw_y);
                // Wetland vs Lake for inland water from the lake's shallow flag.
                let lake_shallow =
                    lake_id != 0 && ymir.lake(lake_id).map(|l| l.shallow).unwrap_or(false);
                resolve_biome(whittaker_id, temp_c, height_m, sea_level_m, class, lake_shallow)
            } else if class == 2 {
                BiomeTypeEnum::Lake
            } else if is_land {
                BiomeTypeEnum::Grassland
            } else {
                BiomeTypeEnum::Ocean
            };
            let bid = biome.to_id();
            let col = get_biome_color(&biome);

            let bo = (dst_row + x) * 4;
            biome_values[bo] = (bid * 17) as u8; // R = primary biome id * 17
            biome_values[bo + 1] = (bid * 17) as u8; // G = secondary biome id * 17
            biome_values[bo + 2] = 0; // B = blend factor
            biome_values[bo + 3] = 255; // A
            if has_biome {
                biome_ids[dst_row + x] = bid as u8;
            }

            source_binary_flipped
                .put_pixel(x as u32, y as u32, Luma([if is_land { 255 } else { 0 }]));
            source_biome_flipped_rgba.put_pixel(
                x as u32,
                y as u32,
                Rgba([col.red(), col.green(), col.blue(), 255]),
            );
        }
    }

    // Distance-based biome blend (reusing the Azgaar algorithm) over biome.u8:
    // fills G = secondary id, B = blend factor so the shader takes the cheap
    // B-driven blend path instead of the always-on jittered vegetation blur.
    // Runs on the BASE ids (before rivers are stamped), with water/lake ids
    // neutralized to Grassland so land vegetation doesn't bleed toward ocean
    // colour at the coast (the coast itself is handled by the ocean SDF/beach).
    if has_biome {
        let grass = BiomeTypeEnum::Grassland.to_id() as u8;
        let land_ids: Vec<u8> = biome_ids
            .iter()
            .map(|&id| if id <= 2 || id == 13 { grass } else { id })
            .collect();
        biome_values = biome_blend_rgba_from_ids(&land_ids, w, h, w, h, 16, 28.0);
    }

    // LL-E: stamp Ymir river channels into the per-cell biome grid (River = 16),
    // width by Strahler order. Per-cell only (never the id*17 biome texture, which
    // caps at 15); drives hover ("River") + the Riverbank shore-type. Rivers are
    // painted by the dedicated river shader. Overrides land/lake cells, not ocean.
    let has_rivers = !ymir.rivers.segments.is_empty() && !biome_ids.is_empty();
    if has_rivers {
        rasterize_rivers(&ymir.rivers, &mut biome_ids, w, h);
    }

    if has_biome {
        let mut hist = [0usize; 17];
        for &id in &biome_ids {
            if (id as usize) < 17 {
                hist[id as usize] += 1;
            }
        }
        let summary: Vec<String> = (0..17)
            .filter(|&i| hist[i] > 0)
            .map(|i| {
                let name = BiomeTypeEnum::from_id(i as i16)
                    .map(|b| format!("{b:?}"))
                    .unwrap_or_else(|| "?".to_string());
                format!("{name}={}", hist[i])
            })
            .collect();
        tracing::info!("✓ Ymir biome resolved from biome.u8: {}", summary.join(", "));
    } else {
        tracing::info!("Ymir biome.u8 absent — using height-only placeholder biome");
    }

    // Per-chunk lake source (display orientation): inland water (effective class
    // 2 = lake_mask or enclosed below-sea). Feeds sample_biome_for_chunk's lake
    // detection + Lakebank shore-type. Empty when no hydro layer is present.
    let source_lake_flipped = if has_inland {
        image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(hf.width, hf.height, |x, y| {
            let c = water_class_flipped[(y * hf.width + x) as usize];
            Luma([if c == 2 { 255 } else { 0 }])
        })
    } else {
        image::ImageBuffer::<Luma<u8>, Vec<u8>>::new(hf.width, hf.height)
    };

    // Chunk grid — same convention as Azgaar (source px × scale ÷ CHUNK_SIZE).
    // NOTE: world units are not yet calibrated to metres; metres_per_cell is
    // stored in TerrainGlobalData for later (LL-B/C) calibration.
    let scaled_width = hf.width as f32 * scale.x;
    let scaled_height = hf.height as f32 * scale.y;
    let n_chunk_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
    let n_chunk_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;
    let world_width = n_chunk_x as f32 * constants::CHUNK_SIZE.x;
    let world_height = n_chunk_y as f32 * constants::CHUNK_SIZE.y;

    // Synthesize a minimal WorldMaps so the ocean/lake global builders and the
    // `global.maps.unwrap()` in generate_chunk_data are satisfied. Its luma u8
    // heightmap feeds ONLY the ocean depth texture — never heightmap_values.
    let maps = build_ymir_worldmaps(ymir, n_chunk_x, n_chunk_y);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let terrain_global_data = shared::TerrainGlobalData {
        name: map_name.to_string(),
        biome_width: hf.width,
        biome_height: hf.height,
        biome_values,
        heightmap_width: hf.width,
        heightmap_height: hf.height,
        heightmap_values: heightmap_values.clone(),
        world_width,
        world_height,
        world_units_per_cell: scale.x,
        metres_per_cell: hf.metres_per_cell(),
        metres_per_height_unit: hf.metres_per_height_unit(),
        height_min_m: hf.min_m,
        sea_level_norm,
        generated_at: now,
    };

    let mut global_state = WorldGlobalState {
        map_name: map_name.to_string(),
        maps: Some(maps),
        source_binary_flipped,
        source_lake_flipped,
        n_chunk_x,
        n_chunk_y,
        scale,
        sdf_resolution: 64,
        max_distance: 150.0,
        grid_config: Some(grid_config),
        source_biome_flipped_rgba: Some(source_biome_flipped_rgba),
        enriched_heightmap: Some(heightmap_values),
        enriched_heightmap_width: hf.width,
        enriched_heightmap_height: hf.height,
        effective_binary: None,
        effective_binary_smoothed: None,
        water_threshold_norm: sea_level_norm,
        coastline_cells: ymir.coastline.clone(),
        ymir_biome_ids: if has_biome { Some(biome_ids) } else { None },
        water_class_flipped: if has_inland {
            Some(water_class_flipped)
        } else {
            None
        },
    };

    if has_inland {
        // The inland-water class is the authority: the effective binary marks only
        // ocean (class 1) as water, so the ocean SDF/shader never covers inland
        // lakes (class 2) — that removes the inland "blue spots".
        // `source_binary_flipped` was built with exactly this rule (land = class
        // != 1), so reuse it.
        global_state.effective_binary = Some(global_state.source_binary_flipped.clone());
        tracing::info!(
            "✓ Ymir effective_binary from water_class + lake_mask (class 1 = ocean water)"
        );
    } else {
        global_state.build_effective_binary(false);
    }

    tracing::info!(
        "✓ Ymir globals built (enriched pipeline bypassed — coastal-SDF/FBM skipped): {}x{} height, {}x{} chunks, sea_level_norm {:.3}",
        hf.width, hf.height, n_chunk_x, n_chunk_y, sea_level_norm
    );

    (global_state, Some(terrain_global_data))
}

/// Minimal `WorldMaps` synthesized from a Ymir height field for the ocean/lake
/// global builders (and the `maps.unwrap()` in chunk generation). Unflipped
/// (raw source orientation), matching the Azgaar `WorldMaps` convention.
fn build_ymir_worldmaps(ymir: &YmirMap, n_chunk_x: i32, n_chunk_y: i32) -> WorldMaps {
    let hf = &ymir.height;
    let w = hf.width;
    let h = hf.height;
    let sea_level_m = ymir.manifest.continent.sea_level_m;

    // u8 heightmap (high byte) — feeds the ocean depth texture only.
    let heightmap = image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(w, h, |x, y| {
        Luma([(hf.data[(y * w + x) as usize] >> 8) as u8])
    });
    // Land/sea binary (used only by the ocean/lake fallback branch).
    let binary_map = image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(w, h, |x, y| {
        Luma([if hf.metres_at(x, y) > sea_level_m { 255 } else { 0 }])
    });
    // Lake mask (raw orientation): inland water cells — a real drainage lake
    // (lake_mask != 0) or an enclosed below-sea pocket (water_class == 2). Empty if
    // neither layer is present. 255 = lake cell (Azgaar lake_map convention).
    let has_lm = !ymir.lake_mask.is_empty();
    let has_wc = !ymir.water_class.is_empty();
    let lake_map = if !has_lm && !has_wc {
        image::ImageBuffer::<Luma<u8>, Vec<u8>>::new(w, h)
    } else {
        image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(w, h, |x, y| {
            let i = (y * w + x) as usize;
            let in_lake = has_lm && ymir.lake_mask[i] != 0;
            let enclosed = has_wc && ymir.water_class[i] == 2;
            Luma([if in_lake || enclosed { 255 } else { 0 }])
        })
    };
    // Placeholder biome image (Grassland land / Ocean water) for the legacy batch path.
    let biome_rgba = image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(w, h, |x, y| {
        if hf.metres_at(x, y) > sea_level_m {
            Rgba([200, 214, 143, 255])
        } else {
            Rgba([0, 15, 30, 255])
        }
    });

    WorldMaps {
        heightmap,
        biome_map: DynamicImage::ImageRgba8(biome_rgba),
        binary_map,
        lake_map,
        config: WorldConfig {
            map_width: w,
            map_height: h,
            chunks_x: n_chunk_x.max(0) as u32,
            chunks_y: n_chunk_y.max(0) as u32,
            seed: ymir.manifest.continent.seed.unwrap_or(0) as u32,
        },
    }
}

/// Load cached world globals from DB, or generate them if not found.
/// This is the normal server startup path.
pub async fn load_or_generate_world_globals(
    map_name: &str,
    db_tables: &DatabaseTables,
    source_kind: WorldSourceKind,
) -> WorldGlobalState {
    let has_globals = db_tables
        .terrain_global_data
        .load_terrain_global_data(map_name)
        .await
        .ok()
        .flatten()
        .is_some();

    if has_globals {
        tracing::info!("Found cached terrain globals, loading maps only...");
        // Log cached global data dimensions for diagnostic
        if let Ok(Some(ref tg)) = db_tables.terrain_global_data.load_terrain_global_data(map_name).await {
            tracing::info!(
                "  [DIAG] Cached TerrainGlobalData: biome {}x{} ({} bytes), heightmap {}x{} ({} bytes)",
                tg.biome_width, tg.biome_height, tg.biome_values.len(),
                tg.heightmap_width, tg.heightmap_height, tg.heightmap_values.len(),
            );
        }
        let t = std::time::Instant::now();

        // Ymir: the .ymir source is authoritative and cheap to reload; rebuild
        // the in-memory globals from it (the cached ocean/lake in DB are reused,
        // and the heightmap is deterministic so it matches the cached copy).
        if source_kind == WorldSourceKind::Ymir {
            let ymir = YmirMap::load(map_name).expect("Failed to load Ymir map");
            let grid_config = setup_grid_config();
            let scale = Vec2::splat(100.);
            let (global_state, _tgd) = build_ymir_globals(map_name, &ymir, grid_config, scale);
            tracing::info!(
                "✓ Ymir world globals rebuilt in {:?} ({}x{} chunks)",
                t.elapsed(),
                global_state.n_chunk_x,
                global_state.n_chunk_y
            );
            return global_state;
        }

        let maps = WorldMaps::load(map_name, 12345).expect("Failed to load world maps");
        let grid_config = setup_grid_config();
        let scale = Vec2::splat(100.);

        let scaled_width = maps.binary_map.width() as f32 * scale.x;
        let scaled_height = maps.binary_map.height() as f32 * scale.y;
        let n_chunk_x = (scaled_width / constants::CHUNK_SIZE.x).ceil() as i32;
        let n_chunk_y = (scaled_height / constants::CHUNK_SIZE.y).ceil() as i32;

        let source_binary_flipped = image::imageops::flip_vertical(&maps.binary_map);
        let source_lake_flipped = image::imageops::flip_vertical(&maps.lake_map);
        let source_biome_flipped = image::imageops::flip_vertical(&maps.biome_map.to_rgba8());

        // Load enriched heightmap from cached terrain global data
        let terrain_global = db_tables
            .terrain_global_data
            .load_terrain_global_data(map_name)
            .await
            .ok()
            .flatten();

        let (enriched_hm, ehm_w, ehm_h) = if let Some(ref tg) = terrain_global {
            (Some(tg.heightmap_values.clone()), tg.heightmap_width, tg.heightmap_height)
        } else {
            (None, 0, 0)
        };

        let mut global_state = WorldGlobalState {
            map_name: map_name.to_string(),
            maps: Some(maps),
            source_binary_flipped,
            source_lake_flipped,
            n_chunk_x,
            n_chunk_y,
            scale,
            sdf_resolution: 64,
            max_distance: 150.0,
            grid_config: Some(grid_config),
            source_biome_flipped_rgba: Some(source_biome_flipped),
            enriched_heightmap: enriched_hm,
            enriched_heightmap_width: ehm_w,
            enriched_heightmap_height: ehm_h,
            effective_binary: None,
            effective_binary_smoothed: None,
            // Azgaar convention: ocean is exactly 0u16 → threshold 0.0.
            water_threshold_norm: 0.0,
            coastline_cells: Vec::new(),
            ymir_biome_ids: None,
            water_class_flipped: None,
        };

        global_state.build_effective_binary(true);

        tracing::info!(
            "✓ World globals loaded in {:?} ({}x{} chunks)",
            t.elapsed(),
            n_chunk_x,
            n_chunk_y
        );
        global_state
    } else {
        tracing::info!("No cached globals found, generating from scratch...");
        generate_world_globals(map_name, db_tables, source_kind).await
    }
}

/// Check if voronoi zone seeds exist in DB; generate them if missing.
/// Called at server startup. Only stores seeds — zone cell membership
/// is computed on demand via shared::voronoi (like the exploration/mist system).
pub async fn ensure_voronoi_zones(db_tables: &DatabaseTables, global_state: &WorldGlobalState) {
    let zone_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM terrain.voronoi_zones")
        .fetch_one(&db_tables.pool)
        .await
        .unwrap_or(0);

    if zone_count > 0 {
        tracing::info!("✓ Voronoi zones OK: {} seeds in database", zone_count);
        return;
    }

    tracing::info!("⚠️ Voronoi zones table is empty — generating seeds...");

    let Some(ref grid_config) = global_state.grid_config else {
        tracing::warn!("No grid config — cannot generate Voronoi zones");
        return;
    };

    // Compute hex bounds from all 4 world corners
    let n_chunk_x = global_state.n_chunk_x;
    let n_chunk_y = global_state.n_chunk_y;
    let world_w = n_chunk_x as f32 * shared::constants::CHUNK_SIZE.x;
    let world_h = n_chunk_y as f32 * shared::constants::CHUNK_SIZE.y;

    let c0 = grid_config.layout.world_pos_to_hex(Vec2::new(0.0, 0.0));
    let c1 = grid_config.layout.world_pos_to_hex(Vec2::new(world_w, 0.0));
    let c2 = grid_config.layout.world_pos_to_hex(Vec2::new(0.0, world_h));
    let c3 = grid_config
        .layout
        .world_pos_to_hex(Vec2::new(world_w, world_h));

    let min_q = c0.x.min(c1.x).min(c2.x).min(c3.x) - 1;
    let max_q = c0.x.max(c1.x).max(c2.x).max(c3.x) + 2;
    let min_r = c0.y.min(c1.y).min(c2.y).min(c3.y) - 1;
    let max_r = c0.y.max(c1.y).max(c2.y).max(c3.y) + 2;

    tracing::info!(
        "World hex bounds: q[{},{}] r[{},{}]",
        min_q,
        max_q,
        min_r,
        max_r
    );

    // Get terrain chunks for land filtering (seeds only on land)
    let chunk_rows = sqlx::query("SELECT DISTINCT chunk_x, chunk_y FROM terrain.terrains")
        .fetch_all(&db_tables.pool)
        .await
        .unwrap_or_default();

    let terrain_chunks: std::collections::HashSet<(i32, i32)> = chunk_rows
        .iter()
        .map(|r| (r.get::<i32, _>("chunk_x"), r.get::<i32, _>("chunk_y")))
        .collect();

    // Generate seeds, filter to land
    let base_spacing = 16;
    let jitter = 3;
    let voronoi_seed = 12345u64;

    let seeds = crate::world::voronoi::seed_generator::generate_seeds_simple(
        min_q,
        max_q,
        min_r,
        max_r,
        base_spacing,
        jitter,
        voronoi_seed,
    );

    let land_seeds: Vec<shared::grid::GridCell> = seeds
        .into_iter()
        .filter(|s| {
            let chunk = s.to_chunk_id(&grid_config.layout);
            terrain_chunks.contains(&(chunk.x, chunk.y))
        })
        .collect();

    tracing::info!("Generated {} land seeds", land_seeds.len());

    if land_seeds.is_empty() {
        tracing::warn!("No land seeds generated — cannot create Voronoi zones");
        return;
    }

    // Store seeds in DB (that's all — no cell partitioning needed)
    let mut count = 0;
    for seed_cell in &land_seeds {
        if db_tables
            .voronoi_zones
            .create_zone(*seed_cell, shared::BiomeTypeEnum::Grassland)
            .await
            .is_ok()
        {
            count += 1;
        }
    }

    tracing::info!("✓ Voronoi seed generation complete: {} seeds stored", count);
}

/// Generate a single chunk's data on demand: terrain mesh, cells, buildings.
/// Saves everything to DB and returns the data for immediate client response.
pub async fn generate_chunk_data(
    chunk_id: &TerrainChunkId,
    global: &WorldGlobalState,
    db_tables: &DatabaseTables,
    game_state: &GameState,
) -> (
    shared::TerrainChunkData,
    Vec<shared::grid::CellData>,
    Vec<BuildingData>,
) {
    let t_total = std::time::Instant::now();
    let map_name = &global.map_name;
    let grid_config = global.grid_config.as_ref().unwrap();

    // 1. Generate terrain mesh + SDF
    let t1 = std::time::Instant::now();
    let terrain_chunk = TerrainMeshData::generate_single_chunk(*chunk_id, global);

    let terrain_data = match terrain_chunk {
        Some(chunk) => chunk.to_shared_terrain_chunk_data(map_name, *chunk_id),
        None => shared::TerrainChunkData {
            name: map_name.to_string(),
            id: *chunk_id,
            ..Default::default()
        },
    };
    let t1_elapsed = t1.elapsed();

    // 2. Sample hex cells for this chunk
    let t2 = std::time::Instant::now();
    let maps = global.maps.as_ref().unwrap();
    let grid_config = global.grid_config.as_ref().unwrap();

    // 2. Sample hex cells for this chunk
    let t2 = std::time::Instant::now();
    let chunk_cells = if let Some(ref source_biome) = global.source_biome_flipped_rgba {
        BiomeMeshData::sample_biome_for_chunk(
            source_biome,
            &global.source_binary_flipped,
            &global.source_lake_flipped,
            &global.scale,
            &grid_config.layout,
            chunk_id,
            global.enriched_heightmap.as_ref().map(|data| {
                (data.as_slice(), global.enriched_heightmap_width, global.enriched_heightmap_height)
            }),
            (
                global.n_chunk_x as f32 * constants::CHUNK_SIZE.x,
                global.n_chunk_y as f32 * constants::CHUNK_SIZE.y,
            ),
            global.water_threshold_norm,
            global.ymir_biome_ids.as_deref(),
        )
    } else {
        vec![]
    };
    let t2_elapsed = t2.elapsed();

    // 3. Generate trees
    let t3 = std::time::Instant::now();
    let trees = NaturalBuildingGenerator::generate(&chunk_cells, game_state);
    let building_data: Vec<BuildingData> = trees.buildings.values().cloned().collect();
    let t3_elapsed = t3.elapsed();

    // 4. Save to DB
    let t4 = std::time::Instant::now();
    for attempt in 0..3 {
        match db_tables.terrains.save_terrain(terrain_data.clone()).await {
            Ok(_) => break,
            Err(e) if attempt < 2 => {
                tracing::warn!(
                    "Terrain save attempt {} failed, retrying: {}",
                    attempt + 1,
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    50 * (attempt as u64 + 1),
                ))
                .await;
            }
            Err(e) => {
                tracing::error!("Failed to save terrain after 3 attempts: {}", e);
            }
        }
    }

    if !chunk_cells.is_empty() {
        for attempt in 0..3 {
            match db_tables.cells.save_cells(&chunk_cells).await {
                Ok(_) => break,
                Err(e) if attempt < 2 => {
                    tracing::warn!(
                        "Cells save attempt {} failed (deadlock?), retrying: {}",
                        attempt + 1,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        50 * (attempt as u64 + 1),
                    ))
                    .await;
                }
                Err(e) => {
                    tracing::error!("Failed to save cells after 3 attempts: {}", e);
                }
            }
        }
    }

    if !building_data.is_empty() {
        for attempt in 0..3 {
            match db_tables.buildings.save_buildings(&building_data).await {
                Ok(_) => break,
                Err(e) if attempt < 2 => {
                    tracing::warn!(
                        "Buildings save attempt {} failed (deadlock?), retrying: {}",
                        attempt + 1,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        50 * (attempt as u64 + 1),
                    ))
                    .await;
                }
                Err(e) => {
                    tracing::error!("Failed to save buildings after 3 attempts: {}", e);
                }
            }
        }
    }
    let t4_elapsed = t4.elapsed();

    tracing::info!(
        "✓ Chunk ({},{}) generated in {:?} [terrain: {:?}, cells: {:?} ({}), trees: {:?} ({}), db: {:?}]",
        chunk_id.x,
        chunk_id.y,
        t_total.elapsed(),
        t1_elapsed,
        t2_elapsed,
        chunk_cells.len(),
        t3_elapsed,
        building_data.len(),
        t4_elapsed
    );

    (terrain_data, chunk_cells, building_data)
}

/// Generate everything in batch (convenience for dev/testing).
/// Uses generate_world_globals + generate_chunk_data for each chunk.
pub async fn generate_world(
    map_name: &str,
    db_tables: &DatabaseTables,
    game_state: &GameState,
    source_kind: WorldSourceKind,
) {
    tracing::info!("Starting full world generation...");
    let start = std::time::Instant::now();

    let global_state = generate_world_globals(map_name, db_tables, source_kind).await;
    let global_maps = global_state.maps.as_ref().unwrap();
    let global_grid_config = global_state.grid_config.as_ref().unwrap();

    // Generate all chunks
    let total_chunks = global_state.n_chunk_x * global_state.n_chunk_y;
    tracing::info!("Generating {} chunks...", total_chunks);

    let mut generated = 0;
    for cy in 0..global_state.n_chunk_y {
        for cx in 0..global_state.n_chunk_x {
            let chunk_id = TerrainChunkId { x: cx, y: cy };

            if global_state.chunk_has_land(&chunk_id) {
                generate_chunk_data(&chunk_id, &global_state, db_tables, game_state).await;
                generated += 1;
            }
        }
    }

    // Load cached scaled binary map for biome mesh generation
    let cache_path = format!("assets/maps/{}_binarymap.bin", map_name);
    let scaled_binary_map: image::ImageBuffer<image::Luma<u8>, Vec<u8>> =
        crate::utils::file_system::load_from_disk(&cache_path)
            .expect("Scaled binary map should be cached after generate_world_globals");

    let biome_mesh_data = BiomeMeshData::from_image(
        map_name,
        &global_maps.biome_map,
        &scaled_binary_map,
        &std::collections::HashMap::new(),
        &global_state.scale,
        "assets/maps/",
    );

    for (id, chunk) in biome_mesh_data.chunks.into_iter() {
        db_tables
            .terrains
            .save_terrain_biome(chunk.to_shared_biome_chunk_data(map_name, id))
            .await
            .expect("Failed to save terrain biome");
    }

    // Voronoi zones (still global)
    tracing::info!("=== GENERATING VORONOI ZONES ===");
    let all_cells = BiomeMeshData::sample_biome(
        map_name,
        &global_maps.biome_map.to_rgba8(),
        &global_state.scale,
        &global_grid_config.layout,
        "assets/maps/",
    );

    // NOTE: BiomeMeshData::from_image and global sample_biome are legacy
    // and still require upscaled images. Skip at large scales.
    tracing::info!("Skipping legacy biome mesh and Voronoi generation (use on-demand)");

    let cells_with_biomes: Vec<(shared::grid::GridCell, shared::BiomeTypeEnum)> = all_cells
        .iter()
        .map(|cell_data| (cell_data.cell, cell_data.biome))
        .collect();

    let min_q = cells_with_biomes
        .iter()
        .map(|(c, _)| c.q)
        .min()
        .unwrap_or(0);
    let max_q = cells_with_biomes
        .iter()
        .map(|(c, _)| c.q)
        .max()
        .unwrap_or(0)
        + 1;
    let min_r = cells_with_biomes
        .iter()
        .map(|(c, _)| c.r)
        .min()
        .unwrap_or(0);
    let max_r = cells_with_biomes
        .iter()
        .map(|(c, _)| c.r)
        .max()
        .unwrap_or(0)
        + 1;

    let voronoi_seed = 12345u64;
    let bounds = (min_q, max_q, min_r, max_r);

    match crate::world::voronoi::generate_and_save_zones(
        &db_tables.voronoi_zones,
        &cells_with_biomes,
        bounds,
        voronoi_seed,
    )
    .await
    {
        Ok(zone_count) => tracing::info!("✓ Generated {} Voronoi zones", zone_count),
        Err(e) => tracing::error!("Failed to generate Voronoi zones: {}", e),
    }

    // NOTE: Exploration Voronoi seeds are deterministic and computed on-demand.
    // No pre-generation needed — see exploration_voronoi_gen.rs.

    tracing::info!(
        "✓ Full world generated in {:?} ({} land chunks out of {})",
        start.elapsed(),
        generated,
        total_chunks
    );
}

/// Selective world clearing: removes procedurally generated data while preserving player data.
///
/// DELETES:
///   - terrain.terrains (chunk meshes)
///   - terrain.terrain_biomes (biome chunk data)
///   - terrain.cells (hex cells)
///   - terrain.voronoi_zones (seeds only — cell membership computed on demand)
///   - terrain.ocean_data
///   - terrain.lake_data
///   - terrain.road_chunk_visibility (regenerated cache, not player roads)
///   - terrain.terrain_global_data
///   - buildings.trees → buildings.buildings_base WHERE category_id = 1 (Natural only)
///
/// PRESERVES:
///   - buildings.buildings_base WHERE category_id != 1 (player buildings)
///   - terrain.road_segments (player-built roads)
///   - organizations.* (orgs, territory, members)
///   - game.players, game.characters
///   - units, inventories, actions
pub async fn clear_world(map_name: &str, db_tables: &DatabaseTables) {
    tracing::info!(
        "=== Starting Selective World Clearing for '{}' ===",
        map_name
    );
    let start = std::time::Instant::now();

    let pool = &db_tables.pool;
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // 1. Natural buildings sub-tables (FK: trees → buildings_base)
    let trees_deleted = sqlx::query(
        "DELETE FROM buildings.trees WHERE building_id IN (SELECT id FROM buildings.buildings_base WHERE category_id = 1)"
    ).execute(&mut *tx).await.expect("Failed to clear trees");
    tracing::info!(
        "  🗑️  buildings.trees: {} rows",
        trees_deleted.rows_affected()
    );

    // 2. Natural buildings base
    let natural_deleted = sqlx::query("DELETE FROM buildings.buildings_base WHERE category_id = 1")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear natural buildings");
    tracing::info!(
        "  🗑️  buildings.buildings_base (Natural): {} rows",
        natural_deleted.rows_affected()
    );

    // 3. Voronoi seeds
    let vz_deleted = sqlx::query("DELETE FROM terrain.voronoi_zones")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear voronoi zones");
    tracing::info!(
        "  🗑️  terrain.voronoi_zones: {} rows",
        vz_deleted.rows_affected()
    );

    // 3b. Exploration Voronoi (only explored state — seeds are deterministic)
    let ev_deleted = sqlx::query("DELETE FROM terrain.explored_voronoi")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear explored voronoi");
    tracing::info!(
        "  🗑️  terrain.explored_voronoi: {} rows",
        ev_deleted.rows_affected()
    );

    // 4. Cells
    let cells_deleted = sqlx::query("DELETE FROM terrain.cells")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear cells");
    tracing::info!(
        "  🗑️  terrain.cells: {} rows",
        cells_deleted.rows_affected()
    );

    // 5. Terrain chunks
    let terrains_deleted = sqlx::query("DELETE FROM terrain.terrains WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrains");
    tracing::info!(
        "  🗑️  terrain.terrains: {} rows",
        terrains_deleted.rows_affected()
    );

    // 6. Biome chunks
    let biomes_deleted = sqlx::query("DELETE FROM terrain.terrain_biomes WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrain biomes");
    tracing::info!(
        "  🗑️  terrain.terrain_biomes: {} rows",
        biomes_deleted.rows_affected()
    );

    // 7. Ocean data
    let ocean_deleted = sqlx::query("DELETE FROM terrain.ocean_data WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear ocean data");
    tracing::info!(
        "  🗑️  terrain.ocean_data: {} rows",
        ocean_deleted.rows_affected()
    );

    // 8. Lake data (only if table exists may not exist on all DB versions)
    let lake_table_exists = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'terrain' AND table_name = 'lake_data')"
)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(false);

    if lake_table_exists {
        let lake_deleted = sqlx::query("DELETE FROM terrain.lake_data WHERE name = $1")
            .bind(map_name)
            .execute(&mut *tx)
            .await
            .expect("Failed to clear lake data");
        tracing::info!(
            "  🗑️  terrain.lake_data: {} rows",
            lake_deleted.rows_affected()
        );
    } else {
        tracing::info!("  ⏩  terrain.lake_data: table not found, skipping");
    }

    // 9. Road chunk visibility cache (not the road_segments themselves)
    let road_cache_deleted = sqlx::query("DELETE FROM terrain.road_chunk_visibility")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear road chunk visibility");
    tracing::info!(
        "  🗑️  terrain.road_chunk_visibility: {} rows",
        road_cache_deleted.rows_affected()
    );

    // 10. Terrain global data
    let global_deleted = sqlx::query("DELETE FROM terrain.terrain_global_data WHERE name = $1")
        .bind(map_name)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear terrain global data");
    tracing::info!(
        "  🗑️  terrain.terrain_global_data: {} rows",
        global_deleted.rows_affected()
    );

    // 11. Territory contours (only if table exists and regenerated from territory data)
    let table_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'terrain' AND table_name = 'territory_chunk_contours')"
    )
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(false);

    if table_exists {
        let contours_deleted = sqlx::query("DELETE FROM terrain.territory_chunk_contours")
            .execute(&mut *tx)
            .await
            .expect("Failed to clear territory chunk contours");
        tracing::info!(
            "  🗑️  terrain.territory_chunk_contours: {} rows",
            contours_deleted.rows_affected()
        );
    } else {
        tracing::info!("  ⏩  terrain.territory_chunk_contours: table not found, skipping");
    }

    tx.commit()
        .await
        .expect("Failed to commit clear transaction");

    // 12. Clean up .bin cache files
    let cache_patterns = [
        format!("assets/maps/{}_binarymap.bin", map_name),
        format!("assets/maps/{}_biomemap.bin", map_name),
    ];
    for pattern in &cache_patterns {
        if std::fs::remove_file(pattern).is_ok() {
            tracing::info!("  🗑️  Removed cache file: {}", pattern);
        }
    }

    tracing::info!(
        "=== ✓ World '{}' cleared in {:?} ===",
        map_name,
        start.elapsed()
    );
    tracing::info!("  Preserved: player buildings, road segments, organizations, units, players");
}

pub async fn save_world_to_png(map_name: &str) {
    tracing::info!("Starting saving...");
    let start = std::time::Instant::now();
    let _ = TerrainMeshData::save_png_image(
        map_name,
        &format!("assets/maps/{}_binarymap.bin", map_name),
    );
    let _ = BiomeMeshData::save_png_image(map_name, "assets/maps/");
    tracing::info!("✓ Saving {} map in {:?}", map_name, start.elapsed());
}

/// Check if territory cells exist but contours are missing, and regenerate if needed.
/// Called at startup after loading world globals to recover from --clear.
pub async fn ensure_territory_contours(db_tables: &DatabaseTables) {
    let contour_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM organizations.territory_contours")
            .fetch_one(&db_tables.pool)
            .await
            .unwrap_or(0);

    let territory_count: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT organization_id) FROM organizations.territory_cells")
            .fetch_one(&db_tables.pool)
            .await
            .unwrap_or(0);

    if territory_count == 0 {
        tracing::info!("✓ No territories — skipping contour check");
        return;
    }

    if contour_count > 0 {
        tracing::info!(
            "✓ Territory contours OK: {} chunks for {} organizations",
            contour_count,
            territory_count
        );
        return;
    }

    tracing::warn!(
        "⚠️ {} organizations have territory but 0 contour chunks — regenerating...",
        territory_count
    );
    regenerate_territory_contours(db_tables).await;
}

/// Regenerate territory contours for all existing organizations
pub async fn regenerate_territory_contours(db_tables: &DatabaseTables) {
    tracing::info!("=== Starting Territory Contours Regeneration ===");
    let start = std::time::Instant::now();

    // Setup grid config
    let grid_config = setup_grid_config();

    // Get all organizations
    let organizations = match sqlx::query("SELECT id, name FROM organizations.organizations")
        .fetch_all(&db_tables.pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch organizations: {}", e);
            return;
        }
    };

    tracing::info!("Found {} organizations to process", organizations.len());

    let mut total_contours = 0;
    let mut processed_orgs = 0;

    for org_row in organizations {
        let org_id: i64 = org_row.get("id");
        let org_name: String = org_row.get("name");

        tracing::info!("Processing organization {} ({})", org_id, org_name);

        // Get territory cells for this organization
        let territory_cells_result = db_tables
            .organizations
            .load_territory_cells(org_id as u64)
            .await;

        match territory_cells_result {
            Ok(cells) if !cells.is_empty() => {
                tracing::info!("  Found {} territory cells", cells.len());

                // Convert GridCells to Hex
                let territory_hex: std::collections::HashSet<hexx::Hex> =
                    cells.iter().map(|cell| cell.to_hex()).collect();

                // Generate and split contours
                let contour_chunks = world::territory::generate_and_split_contour(
                    &territory_hex,
                    &grid_config.layout,
                    4.0,   // jitter amplitude
                    12345, //org_id as u64, // jitter seed (ensures consistency)
                );

                // Log cell world positions and expected chunks for debug
                for cell in cells.iter().take(5) {
                    let hex = cell.to_hex();
                    let world_pos = grid_config.layout.hex_to_world_pos(hex);
                    let expected_chunk = shared::TerrainChunkId::from_world_pos(world_pos);
                    tracing::info!(
                        "  [DIAG] cell q={} r={} → world=({:.0},{:.0}) → chunk ({},{})",
                        cell.q, cell.r, world_pos.x, world_pos.y,
                        expected_chunk.x, expected_chunk.y
                    );
                }

                tracing::info!("  Generated {} contour chunks:", contour_chunks.len());

                // Store contours in database
                let mut stored_count = 0;
                for (chunk_id, contour_segments) in contour_chunks {
                    tracing::info!(
                        "    chunk ({},{}) → {} segments",
                        chunk_id.x, chunk_id.y, contour_segments.len()
                    );
                    match db_tables
                        .territory_contours
                        .store_contour(org_id as u64, chunk_id.x, chunk_id.y, &contour_segments)
                        .await
                    {
                        Ok(_) => {
                            stored_count += 1;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "  Failed to store contour for chunk ({},{}): {}",
                                chunk_id.x,
                                chunk_id.y,
                                e
                            );
                        }
                    }
                }

                tracing::info!(
                    "  ✓ Stored {} territory contour chunks for organization {}",
                    stored_count,
                    org_name
                );
                total_contours += stored_count;
                processed_orgs += 1;
            }
            Ok(_) => {
                tracing::warn!("  Organization {} has no territory cells", org_name);
            }
            Err(e) => {
                tracing::error!(
                    "  Failed to load territory cells for organization {}: {}",
                    org_name,
                    e
                );
            }
        }
    }

    tracing::info!("=== Territory Contours Regeneration Complete ===");
    tracing::info!("Processed {} organizations", processed_orgs);
    tracing::info!("Generated {} total contour chunks", total_contours);
    tracing::info!("Completed in {:?}", start.elapsed());
}
