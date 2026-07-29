//! Signed coastal distance field from vector coastline polylines (LL-B, Ymir path).
//!
//! Replaces the Gaussian-smoothed pixel-mask EDT with a distance computed
//! directly from Ymir's `coastline.geojson` polylines. Output matches the
//! existing SDF byte convention (`((signed/max_distance + 1) * 0.5 * 255)`,
//! i.e. 0 = deep water, 128 = coast, 255 = far inland) so downstream ocean/lake
//! shaders and the `beach_start/beach_end` thresholds are unchanged.

use image::{ImageBuffer, Luma};
use rayon::prelude::*;

/// One coast segment in **output-texel** space.
struct Seg {
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
}

#[inline]
fn dist_point_seg(px: f32, py: f32, s: &Seg) -> f32 {
    let abx = s.bx - s.ax;
    let aby = s.by - s.ay;
    let apx = px - s.ax;
    let apy = py - s.ay;
    let ab2 = abx * abx + aby * aby;
    let t = if ab2 > 0.0 {
        ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let cx = s.ax + t * abx;
    let cy = s.ay + t * aby;
    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}

/// Compute the signed coastal distance field from `polylines_cell` (coastline
/// polylines in Ymir **cell space**, y=0 = south) at resolution `out_w × out_h`.
///
/// - Distances are measured in **output texels** and normalized by
///   `max_distance_texels` (match the ocean SDF's `max_distance`).
/// - Sign comes from `land_mask` (the Ymir `effective_binary`: land = height >
///   sea level); positive inland, negative in water.
/// - Polyline coords are mapped cell→texel with the **same vertical flip** as the
///   LL-A heightmap (cell y=0/south → bottom of the display-oriented texture),
///   so the coast lines up with the land mask and heightmap.
pub fn coastal_signed_distance_field(
    polylines_cell: &[Vec<[f32; 2]>],
    grid_w: u32,
    grid_h: u32,
    out_w: usize,
    out_h: usize,
    max_distance_texels: f32,
    land_mask: &ImageBuffer<Luma<u8>, Vec<u8>>,
) -> Vec<u8> {
    let sx = out_w as f32 / grid_w as f32;
    let sy = out_h as f32 / grid_h as f32;
    let gh = grid_h as f32;

    // cell (cx, cy_south) → output texel (display-oriented, north-up)
    let to_texel = |c: &[f32; 2]| -> (f32, f32) {
        let tx = c[0] * sx;
        let ty = (gh - c[1]) * sy; // vertical flip: south (y=0) → bottom
        (tx, ty)
    };

    // Flatten polylines into segments in texel space.
    let mut segs: Vec<Seg> = Vec::new();
    for line in polylines_cell {
        for w in line.windows(2) {
            let (ax, ay) = to_texel(&w[0]);
            let (bx, by) = to_texel(&w[1]);
            segs.push(Seg { ax, ay, bx, by });
        }
    }
    if segs.is_empty() {
        // No coastline → everything saturates to the sign of the land mask.
        return sign_only(land_mask, out_w, out_h);
    }

    // Uniform bin spatial index: bin size = search radius, so each query texel
    // only scans its own bin ± 1 (3×3 bins).
    let bin = max_distance_texels.ceil().max(1.0);
    let nbx = ((out_w as f32) / bin).ceil() as usize + 1;
    let nby = ((out_h as f32) / bin).ceil() as usize + 1;
    let mut bins: Vec<Vec<u32>> = vec![Vec::new(); nbx * nby];
    for (i, s) in segs.iter().enumerate() {
        let min_bx = ((s.ax.min(s.bx)) / bin).floor().max(0.0) as usize;
        let max_bx = ((s.ax.max(s.bx)) / bin).floor().min((nbx - 1) as f32) as usize;
        let min_by = ((s.ay.min(s.by)) / bin).floor().max(0.0) as usize;
        let max_by = ((s.ay.max(s.by)) / bin).floor().min((nby - 1) as f32) as usize;
        for by in min_by..=max_by {
            for bx in min_bx..=max_bx {
                bins[by * nbx + bx].push(i as u32);
            }
        }
    }

    let mask_w = land_mask.width();
    let mask_h = land_mask.height();

    (0..out_w * out_h)
        .into_par_iter()
        .map(|idx| {
            let tx = (idx % out_w) as f32 + 0.5;
            let ty = (idx / out_w) as f32 + 0.5;

            // nearest-distance search over the 3×3 bin neighborhood
            let cbx = (tx / bin).floor() as isize;
            let cby = (ty / bin).floor() as isize;
            let mut best = max_distance_texels;
            for dby in -1..=1 {
                for dbx in -1..=1 {
                    let bx = cbx + dbx;
                    let by = cby + dby;
                    if bx < 0 || by < 0 || bx as usize >= nbx || by as usize >= nby {
                        continue;
                    }
                    for &si in &bins[by as usize * nbx + bx as usize] {
                        let d = dist_point_seg(tx, ty, &segs[si as usize]);
                        if d < best {
                            best = d;
                        }
                    }
                }
            }

            // Sign from the land mask (display-oriented, same as output).
            let mx = ((idx % out_w) as f32 / out_w as f32 * mask_w as f32) as u32;
            let my = ((idx / out_w) as f32 / out_h as f32 * mask_h as f32) as u32;
            let is_land = land_mask
                .get_pixel(mx.min(mask_w - 1), my.min(mask_h - 1))[0]
                > 128;

            let signed = if is_land { best } else { -best };
            let normalized = (signed / max_distance_texels).clamp(-1.0, 1.0);
            ((normalized + 1.0) * 0.5 * 255.0) as u8
        })
        .collect()
}

/// Fallback when there are no polylines: encode the saturated sign of the mask.
fn sign_only(land_mask: &ImageBuffer<Luma<u8>, Vec<u8>>, out_w: usize, out_h: usize) -> Vec<u8> {
    let mask_w = land_mask.width();
    let mask_h = land_mask.height();
    (0..out_w * out_h)
        .into_par_iter()
        .map(|idx| {
            let mx = ((idx % out_w) as f32 / out_w as f32 * mask_w as f32) as u32;
            let my = ((idx / out_w) as f32 / out_h as f32 * mask_h as f32) as u32;
            let is_land = land_mask
                .get_pixel(mx.min(mask_w - 1), my.min(mask_h - 1))[0]
                > 128;
            if is_land { 255u8 } else { 0u8 }
        })
        .collect()
}
