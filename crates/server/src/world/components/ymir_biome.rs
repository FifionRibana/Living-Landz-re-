//! LL-C: resolve Ymir `biome.u8` (`ymir.WhittakerBiome@v1`) + metric context into
//! the game's `BiomeTypeEnum`, replacing `find_closest_biome` on the Ymir path.
//!
//! The base id→enum table and the threshold overrides are centralized here and
//! are meant to be TUNED (the authoritative Whittaker legend and the thresholds
//! are provisional — see the LL-C plan).

use shared::BiomeTypeEnum;

// ── Tunable thresholds (single documented place) ──────────────────────────

/// A Whittaker-ocean cell deeper than this (metres, negative = below sea level)
/// becomes `DeepOcean`; shallower stays `Ocean`.
pub const DEEP_OCEAN_DEPTH_M: f32 = -200.0;
/// Land at or below this temperature (°C) becomes `Ice`.
pub const ICE_TEMP_C: f32 = -8.0;
/// A Whittaker-desert cell at or below this temperature (°C) becomes `ColdDesert`.
pub const COLD_DESERT_TEMP_C: f32 = 5.0;

/// Base mapping `ymir.WhittakerBiome@v1` id → `BiomeTypeEnum`.
///
/// INFERRED (LL-C): only ids 1–6 occur in current maps; 0/7/8/9 are best-guess
/// defaults. The Azgaar-derived enum has no temperate-seasonal/deciduous forest,
/// so temperate forests map to the nearest available variant. TUNE when the
/// authoritative legend is available.
pub fn whittaker_to_biome(id: u8) -> BiomeTypeEnum {
    match id {
        0 => BiomeTypeEnum::Ocean, // water (default)
        1 => BiomeTypeEnum::Ocean, // water — split Deep/shallow by depth in resolve_biome
        2 => BiomeTypeEnum::Tundra, // coldest, highest altitude
        3 => BiomeTypeEnum::Taiga, // boreal
        4 => BiomeTypeEnum::Grassland, // temperate open
        5 => BiomeTypeEnum::TemperateRainForest, // temperate forest
        6 => BiomeTypeEnum::TropicalDeciduousForest, // warm-temperate forest
        7 => BiomeTypeEnum::Savanna, // warm / dry (default)
        8 => BiomeTypeEnum::Desert, // hot / dry (default)
        9 => BiomeTypeEnum::TropicalRainForest, // hot / wet (default)
        _ => BiomeTypeEnum::Undefined,
    }
}

/// Resolve the final `BiomeTypeEnum` from a Whittaker id plus metric context.
///
/// Order: (1) water → `DeepOcean`/`Ocean` split by depth; (2) cold land → `Ice`;
/// (3) cold desert → `ColdDesert`. Lake (from `lake_mask`) and Wetland
/// (`flow_accumulation` + low slope) are applied by the caller only when those
/// layers are present — absent in current maps.
pub fn resolve_biome(whittaker_id: u8, temp_c: Option<f32>, height_m: f32) -> BiomeTypeEnum {
    let base = whittaker_to_biome(whittaker_id);

    // 1. Water: split by bathymetry.
    if matches!(base, BiomeTypeEnum::Ocean | BiomeTypeEnum::DeepOcean) {
        return if height_m <= DEEP_OCEAN_DEPTH_M {
            BiomeTypeEnum::DeepOcean
        } else {
            BiomeTypeEnum::Ocean
        };
    }

    // 2/3. Temperature overrides on land.
    if let Some(t) = temp_c {
        if t <= ICE_TEMP_C {
            return BiomeTypeEnum::Ice;
        }
        if matches!(base, BiomeTypeEnum::Desert) && t <= COLD_DESERT_TEMP_C {
            return BiomeTypeEnum::ColdDesert;
        }
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_splits_by_depth() {
        assert_eq!(resolve_biome(1, Some(10.0), -500.0), BiomeTypeEnum::DeepOcean);
        assert_eq!(resolve_biome(1, Some(10.0), -10.0), BiomeTypeEnum::Ocean);
    }

    #[test]
    fn temperate_land_maps_directly() {
        assert_eq!(resolve_biome(4, Some(10.0), 300.0), BiomeTypeEnum::Grassland);
        assert_eq!(resolve_biome(2, Some(-2.0), 3000.0), BiomeTypeEnum::Tundra);
        assert_eq!(resolve_biome(3, Some(2.0), 1500.0), BiomeTypeEnum::Taiga);
    }

    #[test]
    fn ice_override_on_cold_land() {
        assert_eq!(resolve_biome(2, Some(-10.0), 3500.0), BiomeTypeEnum::Ice);
        assert_eq!(resolve_biome(4, Some(-8.5), 500.0), BiomeTypeEnum::Ice);
    }

    #[test]
    fn cold_desert_override() {
        assert_eq!(resolve_biome(8, Some(2.0), 400.0), BiomeTypeEnum::ColdDesert);
        assert_eq!(resolve_biome(8, Some(25.0), 400.0), BiomeTypeEnum::Desert);
    }
}
