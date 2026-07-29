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
        let field = HeightField {
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

        // ── biome.u8 + temperature.i16 (LL-C) ──
        let biome = load_biome_u8(&dir, &manifest, width, height);
        let temperature = load_temperature_i16(&dir, &manifest, width, height);
        tracing::info!(
            "✓ Ymir biome layer: {} cells, temperature layer: {} cells",
            biome.len(),
            temperature.len()
        );

        // Cliffs (cliffs.geojson) are consumed client-side by the debug overlay
        // (LL-C), read directly from the .ymir asset via the shared GeoJSON reader.
        // TODO(LL-C+): flow_accumulation / lake_mask (absent in current maps) for
        //             Wetland / Lake biome rules.

        Ok(Self {
            manifest,
            height: field,
            coastline,
            biome,
            temperature,
        })
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
