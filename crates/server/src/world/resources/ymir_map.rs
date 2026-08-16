//! Native reader for Ymir `.ymir` continent directories (LL-A).
//!
//! A `.ymir` map is a directory `assets/maps/<name>.ymir/` containing a
//! `manifest.json` plus typed binary/vector layer files. This module reads the
//! manifest and the `height` layer (u16 LE, row-major, y=0 = south) into a typed
//! [`HeightField`] whose values are a *normalized* encoding of metric altitude
//! (see [`HeightField::metres_of_u16`]).
//!
//! Only the `height` layer is consumed in LL-A. Coastline/cliffs (LL-B) and the
//! `biome.u8` layer (LL-C) are read later; their manifest entries are parsed but
//! their payloads are ignored here (see the TODOs in [`YmirMap::load`]).

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Supported `format_version` major. Loading errors on any other major.
const SUPPORTED_FORMAT_MAJOR: u64 = 1;

// ---------------------------------------------------------------------------
// manifest.json schema (mirrors Ymir 1.x; unknown fields are ignored)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct YmirManifest {
    pub format_version: String,
    #[serde(default)]
    pub ymir_version: Option<String>,
    pub continent: Continent,
    #[serde(default)]
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Continent {
    pub name: String,
    #[serde(default)]
    pub seed: Option<i64>,
    pub grid: Grid,
    #[serde(default)]
    pub window_km: f32,
    pub km_per_cell: f32,
    pub vertical_scale: VerticalScale,
    /// Absolute metres at sea level (usually 0.0).
    pub sea_level_m: f32,
    /// Metres from sea level to the highest peak.
    pub max_elevation_m: f32,
    /// Metres from sea level to the deepest point (positive magnitude).
    pub max_depth_m: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerticalScale {
    /// Normalized position of sea level in the u16 range (0.0..1.0). 0.5 means
    /// sea level sits at u16 ≈ 32768.
    pub sea_level_norm: f32,
    #[serde(default)]
    pub altitude_norm_half_range: f32,
    #[serde(default)]
    pub depth_scale_m: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Layer {
    pub id: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub present: bool,
    #[serde(default)]
    pub dtype: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub endianness: Option<String>,
}

// ---------------------------------------------------------------------------
// lakes.json schema (Ymir C1Lake) + the mirror we keep
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct RawLake {
    base: RawLakeBase,
    #[serde(default)]
    level_m: f32,
    #[serde(default)]
    depth_m: f32,
    #[serde(default)]
    area_km2: f32,
    #[serde(default)]
    lake_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawLakeBase {
    id: u32,
    #[serde(default)]
    outlet: Option<[i64; 2]>,
    #[serde(default)]
    shallow: bool,
}

/// Endorheic (closed basin, no outlet) vs Exorheic (drains to the sea). Used
/// later for salt/closed-basin visuals; `Unknown` if the field is missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LakeType {
    Exorheic,
    Endorheic,
    Unknown,
}

/// One inland lake, mirrored from `lakes.json`. `lake_mask` cells carry this
/// lake's [`id`](LakeInfo::id). Coordinates (outlet) are erosion-grid cells.
#[derive(Debug, Clone)]
pub struct LakeInfo {
    /// 1-based lake id; matches the per-cell value in `lake_mask` (0 = no lake).
    pub id: u32,
    /// Water-surface elevation, metres.
    pub level_m: f32,
    /// Maximum depth, metres.
    pub depth_m: f32,
    /// Surface area, km².
    pub area_km2: f32,
    /// Flooded shallow depression (#155) → Wetland; deep lake otherwise.
    pub shallow: bool,
    pub lake_type: LakeType,
    /// Outlet cell (erosion grid), if any.
    pub outlet: Option<[i64; 2]>,
}

// ---------------------------------------------------------------------------
// Decoded height layer
// ---------------------------------------------------------------------------

/// The decoded `height` layer: a normalized u16 field with the metric range
/// needed to convert cells to metres.
///
/// The stored u16 spans the full `[0, 65535]` range and maps linearly onto
/// `[min_m, max_m]`; sea level sits at `sea_level_norm` of the range.
pub struct HeightField {
    pub width: u32,
    pub height: u32,
    /// Metres at u16 = 0 (deepest): `sea_level_m - max_depth_m`.
    pub min_m: f32,
    /// Metres at u16 = 65535 (highest peak): `sea_level_m + max_elevation_m`.
    pub max_m: f32,
    pub km_per_cell: f32,
    /// Normalized sea level (0.0..1.0); doubles as the land/sea threshold.
    pub sea_level_norm: f32,
    /// Row-major, y=0 = south (as authored by Ymir; not yet Y-flipped).
    pub data: Vec<u16>,
}

impl HeightField {
    /// Convert a raw u16 sample to absolute metres.
    ///
    /// `metres = min_m + (v / 65535) * (max_m - min_m)`
    #[inline]
    pub fn metres_of_u16(&self, v: u16) -> f32 {
        self.min_m + (v as f32 / 65535.0) * (self.max_m - self.min_m)
    }

    /// Metres at cell `(x, y)` (no bounds check; caller guarantees in-range).
    #[inline]
    pub fn metres_at(&self, x: u32, y: u32) -> f32 {
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.metres_of_u16(self.data[idx])
    }

    /// Metres represented by one u16 step: `(max_m - min_m) / 65535`.
    #[inline]
    pub fn metres_per_height_unit(&self) -> f32 {
        (self.max_m - self.min_m) / 65535.0
    }

    /// Ground-plane metres per grid cell: `km_per_cell * 1000`.
    #[inline]
    pub fn metres_per_cell(&self) -> f32 {
        self.km_per_cell * 1000.0
    }

    /// Re-anchor sea level to the height sampled along the vector coastline.
    ///
    /// The manifest's `sea_level_norm` (0.5) does not match this map's u16
    /// encoding: `coastline.geojson` (the authoritative 0 m contour, shared with
    /// `cliffs.geojson`) actually sits at ~0.574 of the u16 range. Trusting 0.5
    /// puts the height-derived land/sea boundary (`effective_binary`, mesh, biome)
    /// ~0.07 norm — dozens of cells — seaward of the real coast, which makes the
    /// ocean-SDF sign flip far from the coastline (a discontinuous SDF → a 1-px
    /// beach) and paints a below-sea band as land ("blue spots").
    ///
    /// We calibrate `sea_level_norm` to the **median** height-norm along the
    /// coastline (robust; the spread is ~0.001), then re-anchor `min_m`/`max_m` so
    /// `metres_of_u16` reports **0 m at that norm**. The total metric span (hence
    /// `metres_per_height_unit`, hence slopes) is preserved — only the zero moves.
    /// No-op if the coastline is absent (keep the manifest values).
    pub fn calibrate_sea_level_from_coastline(&mut self, coastline: &[Vec<[f32; 2]>]) {
        let w = self.width as usize;
        let mut norms: Vec<f32> = Vec::new();
        for line in coastline {
            for p in line {
                let cx = (p[0].round() as i64).clamp(0, self.width as i64 - 1) as usize;
                let cy = (p[1].round() as i64).clamp(0, self.height as i64 - 1) as usize;
                norms.push(self.data[cy * w + cx] as f32 / 65535.0);
            }
        }
        if norms.is_empty() {
            return;
        }
        norms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = norms[norms.len() / 2];

        let span = self.max_m - self.min_m; // preserve metres-per-u16-step
        let old_norm = self.sea_level_norm;
        self.sea_level_norm = median;
        self.min_m = -median * span;
        self.max_m = self.min_m + span;

        tracing::info!(
            "✓ Ymir sea level calibrated from coastline: sea_level_norm {:.4} → {:.4} \
             (n={}, spread={:.4}); metric range re-anchored to [{:.0}..{:.0}] m",
            old_norm,
            median,
            norms.len(),
            norms[norms.len().saturating_sub(1)] - norms[0],
            self.min_m,
            self.max_m,
        );
    }
}

// ---------------------------------------------------------------------------
// Loaded map
// ---------------------------------------------------------------------------

pub struct YmirMap {
    pub manifest: YmirManifest,
    pub height: HeightField,
    /// Sea-level coastline polylines in **cell space** (x in [0,width], y in
    /// [0,height], y=0 = south), from `coastline.geojson`. Empty if the layer is
    /// absent or `present: false` (callers then fall back to the height threshold).
    pub coastline: Vec<Vec<[f32; 2]>>,
    /// `biome.u8` — one Whittaker id (`ymir.WhittakerBiome@v1`) per cell,
    /// row-major, y=0 = south. Empty if the layer is absent (callers fall back to
    /// a height-derived placeholder biome).
    pub biome: Vec<u8>,
    /// `temperature.i16` — °C × 100 per cell, row-major, y=0 = south. Empty if absent.
    pub temperature: Vec<i16>,
    /// `water_class.u8` — per cell: 0 = land, 1 = ocean (edge-connected sea),
    /// 2 = inland water (enclosed below-sea / lake). Row-major, y=0 = south.
    /// Empty if absent — callers then fall back to the height/coastline threshold.
    pub water_class: Vec<u8>,
    /// `lake_mask.u32` — per-cell lake id (0 = no lake), matching [`LakeInfo::id`].
    /// Row-major, y=0 = south. Empty if absent.
    pub lake_mask: Vec<u32>,
    /// `flow_accumulation.f32` — per-cell upstream flow. Row-major, y=0 = south.
    /// Empty if absent. (Loaded for future river/wetland use.)
    pub flow_accumulation: Vec<f32>,
    /// Inland lakes mirrored from `lakes.json` (empty if absent).
    pub lakes: Vec<LakeInfo>,
    /// `lake id → index into lakes` for O(1) per-cell lookup.
    lakes_by_id: std::collections::HashMap<u32, usize>,
    /// River graph from `rivers.json` (empty if absent). Segments carry Ymir's
    /// authored `navigability`; topology is index-aligned. Consumed for gameplay
    /// (navigability + basin connectivity); rendered client-side.
    pub rivers: shared::rivers::RiverNetwork,
}

/// Where a river ultimately drains, resolved by walking `downstream` to the sink
/// and classifying the sink outlet cell via `water_class` / `lake_mask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiverSinkKind {
    Sea,
    Lake,
    /// Endorheic / land sink (drains to a closed basin, not sea or lake).
    Inland,
}

impl YmirMap {
    /// Temperature in °C at cell `(x, y)`, or `None` if the layer is absent.
    #[inline]
    pub fn temperature_c_at(&self, x: u32, y: u32) -> Option<f32> {
        if self.temperature.is_empty() {
            return None;
        }
        let idx = (y as usize) * (self.height.width as usize) + (x as usize);
        self.temperature.get(idx).map(|&t| t as f32 / 100.0)
    }

    /// Water class at cell `(x, y)` (0 land / 1 ocean / 2 inland), or `None` if
    /// the `water_class` layer is absent.
    #[inline]
    pub fn water_class_at(&self, x: u32, y: u32) -> Option<u8> {
        if self.water_class.is_empty() {
            return None;
        }
        let idx = (y as usize) * (self.height.width as usize) + (x as usize);
        self.water_class.get(idx).copied()
    }

    /// Lake id at cell `(x, y)` (0 = no lake), or `None` if the `lake_mask` layer
    /// is absent.
    #[inline]
    pub fn lake_id_at(&self, x: u32, y: u32) -> Option<u32> {
        if self.lake_mask.is_empty() {
            return None;
        }
        let idx = (y as usize) * (self.height.width as usize) + (x as usize);
        self.lake_mask.get(idx).copied()
    }

    /// Lake metadata for a given `lake_mask` id, or `None`.
    #[inline]
    pub fn lake(&self, id: u32) -> Option<&LakeInfo> {
        self.lakes_by_id.get(&id).map(|&i| &self.lakes[i])
    }

    /// Resolve where river segment `seg_idx` ultimately drains: walk `downstream`
    /// to the sink, then classify the sink's outlet (last) cell via `water_class`
    /// / `lake_mask`. `None` if the segment index is out of range or the hydro
    /// layers needed to classify are absent.
    pub fn river_sink_kind(&self, seg_idx: usize) -> Option<RiverSinkKind> {
        let sink = self.rivers.sink_of(seg_idx)?;
        let outlet = self.rivers.segments.get(sink)?.points.last()?;
        let x = (outlet[0].round() as i64).clamp(0, self.height.width as i64 - 1) as u32;
        let y = (outlet[1].round() as i64).clamp(0, self.height.height as i64 - 1) as u32;
        if self.lake_id_at(x, y).unwrap_or(0) != 0 {
            return Some(RiverSinkKind::Lake);
        }
        match self.water_class_at(x, y) {
            Some(1) => Some(RiverSinkKind::Sea),
            Some(2) => Some(RiverSinkKind::Lake),
            Some(_) => Some(RiverSinkKind::Inland),
            None => None,
        }
    }
}

impl YmirMap {
    /// Directory for a given map name: `assets/maps/<name>.ymir`.
    pub fn dir_for(map_name: &str) -> PathBuf {
        Path::new("assets/maps").join(format!("{map_name}.ymir"))
    }

    /// True if `assets/maps/<name>.ymir/manifest.json` exists.
    pub fn exists(map_name: &str) -> bool {
        Self::dir_for(map_name).join("manifest.json").is_file()
    }

    /// Load the manifest and the metric `height` layer.
    ///
    /// Errors on: missing/invalid manifest, unsupported `format_version` major,
    /// missing/absent/non-u16 height layer, or a height file whose byte length
    /// does not match `width * height * 2`.
    pub fn load(map_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = Self::dir_for(map_name);
        tracing::info!("Loading Ymir map from {}", dir.display());

        // ── manifest.json ──
        let manifest_path = dir.join("manifest.json");
        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Ymir: cannot read {}: {e}", manifest_path.display()))?;
        let manifest: YmirManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Ymir: invalid manifest.json: {e}"))?;

        // ── format_version major gate ──
        let major = manifest
            .format_version
            .split('.')
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| {
                format!("Ymir: malformed format_version '{}'", manifest.format_version)
            })?;
        if major != SUPPORTED_FORMAT_MAJOR {
            return Err(format!(
                "Ymir: unsupported format_version major {major} (this build supports {SUPPORTED_FORMAT_MAJOR}.x)"
            )
            .into());
        }

        // ── locate + validate the height layer ──
        let hl = manifest
            .layers
            .iter()
            .find(|l| l.id == "height")
            .ok_or("Ymir: manifest has no 'height' layer")?;
        if !hl.present {
            return Err("Ymir: 'height' layer is marked not present".into());
        }
        if let Some(dtype) = hl.dtype.as_deref() {
            if dtype != "u16" {
                return Err(
                    format!("Ymir: height layer dtype '{dtype}' unsupported (expected u16)").into(),
                );
            }
        }
        if let Some(endian) = hl.endianness.as_deref() {
            if endian != "le" {
                return Err(
                    format!("Ymir: height layer endianness '{endian}' unsupported (expected le)")
                        .into(),
                );
            }
        }

        let width = hl.width.unwrap_or(manifest.continent.grid.width);
        let height = hl.height.unwrap_or(manifest.continent.grid.height);
        let file = hl.file.clone().unwrap_or_else(|| "height.u16".to_string());

        // ── read + decode u16 LE ──
        let path = dir.join(&file);
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("Ymir: cannot read height layer {}: {e}", path.display()))?;
        let expected = (width as usize) * (height as usize) * 2;
        if bytes.len() != expected {
            return Err(format!(
                "Ymir: {} is {} bytes, expected {} ({}x{}x2)",
                path.display(),
                bytes.len(),
                expected,
                width,
                height
            )
            .into());
        }
        let data: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        // ── metric range from the continent block (no min_m/max_m on the layer) ──
        let c = &manifest.continent;
        let min_m = c.sea_level_m - c.max_depth_m;
        let max_m = c.sea_level_m + c.max_elevation_m;
        let mut field = HeightField {
            width,
            height,
            min_m,
            max_m,
            km_per_cell: c.km_per_cell,
            sea_level_norm: c.vertical_scale.sea_level_norm,
            data,
        };

        tracing::info!(
            "✓ Ymir '{}' loaded: {}x{} height, metric [{:.0}..{:.0}] m, sea_level_norm {:.3}, {:.1} m/cell, {:.4} m/u16-step",
            c.name,
            field.width,
            field.height,
            field.min_m,
            field.max_m,
            field.sea_level_norm,
            field.metres_per_cell(),
            field.metres_per_height_unit(),
        );

        // ── coastline.geojson (LL-B): sea-level MultiLineString in cell space ──
        let coastline = load_coastline(&dir, &manifest);
        tracing::info!(
            "✓ Ymir coastline: {} polylines ({} points)",
            coastline.len(),
            coastline.iter().map(|l| l.len()).sum::<usize>()
        );

        // Calibrate sea level to the actual coastline contour (the manifest's
        // sea_level_norm does not match this map's u16 encoding). Must run after
        // both the height field and the coastline are loaded.
        field.calibrate_sea_level_from_coastline(&coastline);

        // ── biome.u8 + temperature.i16 (LL-C) ──
        let biome = load_biome_u8(&dir, &manifest, width, height);
        let temperature = load_temperature_i16(&dir, &manifest, width, height);
        tracing::info!(
            "✓ Ymir biome layer: {} cells, temperature layer: {} cells",
            biome.len(),
            temperature.len()
        );

        // ── hydro layers (LL-D): water_class, lake_mask, flow_accumulation, lakes ──
        let water_class = load_water_class(&dir, &manifest, width, height);
        let lake_mask = load_lake_mask(&dir, &manifest, width, height);
        let flow_accumulation = load_flow_accumulation(&dir, &manifest, width, height);
        let lakes = load_lakes(&dir, &manifest);
        let lakes_by_id = lakes.iter().enumerate().map(|(i, l)| (l.id, i)).collect();
        let rivers = load_rivers(&dir, &manifest);
        tracing::info!(
            "✓ Ymir hydro: water_class {} cells, lake_mask {} cells, flow_accumulation {} cells, {} lakes, {} river segments",
            water_class.len(),
            lake_mask.len(),
            flow_accumulation.len(),
            lakes.len(),
            rivers.segments.len(),
        );

        // Cliffs (cliffs.geojson) are consumed client-side by the debug overlay
        // (LL-C), read directly from the .ymir asset via the shared GeoJSON reader.

        Ok(Self {
            manifest,
            height: field,
            coastline,
            biome,
            temperature,
            water_class,
            lake_mask,
            flow_accumulation,
            lakes,
            lakes_by_id,
            rivers,
        })
    }
}

/// Parse `rivers.json` (Ymir river graph) via the shared reader. Empty network if
/// the layer is absent/not present or unreadable.
fn load_rivers(dir: &Path, manifest: &YmirManifest) -> shared::rivers::RiverNetwork {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "rivers") else {
        return shared::rivers::RiverNetwork::default();
    };
    if !layer.present {
        return shared::rivers::RiverNetwork::default();
    }
    let file = layer.file.clone().unwrap_or_else(|| "rivers.json".to_string());
    let path = dir.join(&file);
    match std::fs::read_to_string(&path) {
        Ok(raw) => shared::rivers::RiverNetwork::parse(&raw),
        Err(e) => {
            tracing::warn!("Ymir: cannot read rivers {}: {e}", path.display());
            shared::rivers::RiverNetwork::default()
        }
    }
}

/// Load `coastline.geojson` into polylines in cell space, via the shared GeoJSON
/// reader (`shared::geojson::parse_multilinestring_geojson`, also used by the
/// client cliff overlay). Returns empty if the layer is absent/not present or
/// unreadable (callers fall back to the metric-height threshold).
fn load_coastline(dir: &Path, manifest: &YmirManifest) -> Vec<Vec<[f32; 2]>> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "coastline") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "coastline.geojson".to_string());
    let path = dir.join(&file);
    match std::fs::read_to_string(&path) {
        Ok(raw) => shared::geojson::parse_multilinestring_geojson(&raw),
        Err(e) => {
            tracing::warn!("Ymir: cannot read coastline {}: {e}", path.display());
            Vec::new()
        }
    }
}

/// Load the `biome.u8` layer (one Whittaker id per cell, row-major, y=0 south).
/// Empty if absent/not present or size mismatch (caller falls back to a
/// height-derived placeholder biome).
fn load_biome_u8(dir: &Path, manifest: &YmirManifest, w: u32, h: u32) -> Vec<u8> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "biome") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "biome.u8".to_string());
    let path = dir.join(&file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Ymir: cannot read biome {}: {e}", path.display());
            return Vec::new();
        }
    };
    let expected = (w as usize) * (h as usize);
    if bytes.len() != expected {
        tracing::warn!(
            "Ymir: biome.u8 is {} bytes, expected {} ({}x{}) — ignoring",
            bytes.len(),
            expected,
            w,
            h
        );
        return Vec::new();
    }
    bytes
}

/// Load the `temperature.i16` layer (°C × 100, LE, row-major, y=0 south).
/// Empty if absent/not present or size mismatch.
fn load_temperature_i16(dir: &Path, manifest: &YmirManifest, w: u32, h: u32) -> Vec<i16> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "temperature") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "temperature.i16".to_string());
    let path = dir.join(&file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Ymir: cannot read temperature {}: {e}", path.display());
            return Vec::new();
        }
    };
    let expected = (w as usize) * (h as usize) * 2;
    if bytes.len() != expected {
        tracing::warn!(
            "Ymir: temperature.i16 is {} bytes, expected {} ({}x{}x2) — ignoring",
            bytes.len(),
            expected,
            w,
            h
        );
        return Vec::new();
    }
    bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/// Load the `water_class.u8` layer (0 land / 1 ocean / 2 inland, row-major,
/// y=0 south). Empty if absent/not present or size mismatch.
fn load_water_class(dir: &Path, manifest: &YmirManifest, w: u32, h: u32) -> Vec<u8> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "water_class") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "water_class.u8".to_string());
    let path = dir.join(&file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Ymir: cannot read water_class {}: {e}", path.display());
            return Vec::new();
        }
    };
    let expected = (w as usize) * (h as usize);
    if bytes.len() != expected {
        tracing::warn!(
            "Ymir: water_class.u8 is {} bytes, expected {} ({}x{}) — ignoring",
            bytes.len(),
            expected,
            w,
            h
        );
        return Vec::new();
    }
    bytes
}

/// Load the `lake_mask.u32` layer (per-cell lake id, LE, row-major, y=0 south).
/// Empty if absent/not present or size mismatch.
fn load_lake_mask(dir: &Path, manifest: &YmirManifest, w: u32, h: u32) -> Vec<u32> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "lake_mask") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "lake_mask.u32".to_string());
    let path = dir.join(&file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Ymir: cannot read lake_mask {}: {e}", path.display());
            return Vec::new();
        }
    };
    let expected = (w as usize) * (h as usize) * 4;
    if bytes.len() != expected {
        tracing::warn!(
            "Ymir: lake_mask.u32 is {} bytes, expected {} ({}x{}x4) — ignoring",
            bytes.len(),
            expected,
            w,
            h
        );
        return Vec::new();
    }
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Load the `flow_accumulation.f32` layer (LE, row-major, y=0 south).
/// Empty if absent/not present or size mismatch.
fn load_flow_accumulation(dir: &Path, manifest: &YmirManifest, w: u32, h: u32) -> Vec<f32> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "flow_accumulation") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer
        .file
        .clone()
        .unwrap_or_else(|| "flow_accumulation.f32".to_string());
    let path = dir.join(&file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Ymir: cannot read flow_accumulation {}: {e}", path.display());
            return Vec::new();
        }
    };
    let expected = (w as usize) * (h as usize) * 4;
    if bytes.len() != expected {
        tracing::warn!(
            "Ymir: flow_accumulation.f32 is {} bytes, expected {} ({}x{}x4) — ignoring",
            bytes.len(),
            expected,
            w,
            h
        );
        return Vec::new();
    }
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Parse `lakes.json` (Ymir C1Lake array) into a [`LakeInfo`] mirror. Empty if
/// absent/not present or unparseable.
fn load_lakes(dir: &Path, manifest: &YmirManifest) -> Vec<LakeInfo> {
    let Some(layer) = manifest.layers.iter().find(|l| l.id == "lakes") else {
        return Vec::new();
    };
    if !layer.present {
        return Vec::new();
    }
    let file = layer.file.clone().unwrap_or_else(|| "lakes.json".to_string());
    let path = dir.join(&file);
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Ymir: cannot read lakes {}: {e}", path.display());
            return Vec::new();
        }
    };
    let parsed: Vec<RawLake> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Ymir: invalid lakes.json: {e}");
            return Vec::new();
        }
    };
    parsed
        .into_iter()
        .map(|r| LakeInfo {
            id: r.base.id,
            level_m: r.level_m,
            depth_m: r.depth_m,
            area_km2: r.area_km2,
            shallow: r.base.shallow,
            lake_type: match r.lake_type.as_deref() {
                Some("Exorheic") => LakeType::Exorheic,
                Some("Endorheic") => LakeType::Endorheic,
                _ => LakeType::Unknown,
            },
            outlet: r.base.outlet,
        })
        .collect()
}
