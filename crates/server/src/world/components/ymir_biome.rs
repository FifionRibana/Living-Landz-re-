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

/// Resolve the final `BiomeTypeEnum` from a Whittaker id plus metric + hydro context.
///
/// **`water_class` is the authority for land vs water** (Ymir's border flood-fill:
/// `0` land, `1` ocean = edge-connected sea, `2` inland water = enclosed lake). It
/// must agree with the `effective_binary`, ocean SDF and lake pipeline, which are
/// all driven by the same `water_class`. The Whittaker id only chooses the *land*
/// biome type. When the `water_class` layer is absent the caller derives it from
/// the calibrated height threshold (`≤ sea_level` → 1, else 0), preserving the
/// previous height-authority behaviour (no class 2 without the layer).
///
/// Order:
/// 1. `water_class == 1` (ocean) → `DeepOcean`/`Ocean` split by bathymetry.
/// 2. `water_class == 2` (inland water) → **`Wetland`** when the lake is a shallow
///    flooded depression (#155) else **`Lake`**. This is the one documented place
///    for the Lake/Wetland predicate (was a no-op until `lake_mask`/`lakes` shipped).
/// 3. land → the Whittaker land biome (Whittaker-water on a land cell falls back to
///    `Grassland`), then cold land → `Ice`, cold Desert → `ColdDesert`.
pub fn resolve_biome(
    whittaker_id: u8,
    temp_c: Option<f32>,
    height_m: f32,
    sea_level_m: f32,
    water_class: u8,
    lake_shallow: bool,
) -> BiomeTypeEnum {
    // 1. Ocean (edge-connected sea): bathymetry split.
    if water_class == 1 {
        return if height_m <= sea_level_m + DEEP_OCEAN_DEPTH_M {
            BiomeTypeEnum::DeepOcean
        } else {
            BiomeTypeEnum::Ocean
        };
    }

    // 2. Inland water (enclosed): Wetland if a shallow flooded depression, else Lake.
    if water_class == 2 {
        return if lake_shallow {
            BiomeTypeEnum::Wetland
        } else {
            BiomeTypeEnum::Lake
        };
    }

    // 3. Land: use the Whittaker land biome; if Whittaker classed this cell as
    //    water, fall back to Grassland so land never renders as ocean.
    let base = match whittaker_to_biome(whittaker_id) {
        BiomeTypeEnum::Ocean | BiomeTypeEnum::DeepOcean | BiomeTypeEnum::Lake => {
            BiomeTypeEnum::Grassland
        }
        other => other,
    };

    // Temperature overrides on land.
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
    fn ocean_splits_by_depth() {
        assert_eq!(resolve_biome(1, Some(10.0), -500.0, 0.0, 1, false), BiomeTypeEnum::DeepOcean);
        assert_eq!(resolve_biome(1, Some(10.0), -10.0, 0.0, 1, false), BiomeTypeEnum::Ocean);
    }

    #[test]
    fn temperate_land_maps_directly() {
        assert_eq!(resolve_biome(4, Some(10.0), 300.0, 0.0, 0, false), BiomeTypeEnum::Grassland);
        assert_eq!(resolve_biome(2, Some(-2.0), 3000.0, 0.0, 0, false), BiomeTypeEnum::Tundra);
        assert_eq!(resolve_biome(3, Some(2.0), 1500.0, 0.0, 0, false), BiomeTypeEnum::Taiga);
    }

    #[test]
    fn ice_override_on_cold_land() {
        assert_eq!(resolve_biome(2, Some(-10.0), 3500.0, 0.0, 0, false), BiomeTypeEnum::Ice);
        assert_eq!(resolve_biome(4, Some(-8.5), 500.0, 0.0, 0, false), BiomeTypeEnum::Ice);
    }

    #[test]
    fn cold_desert_override() {
        assert_eq!(resolve_biome(8, Some(2.0), 400.0, 0.0, 0, false), BiomeTypeEnum::ColdDesert);
        assert_eq!(resolve_biome(8, Some(25.0), 400.0, 0.0, 0, false), BiomeTypeEnum::Desert);
    }

    #[test]
    fn water_class_is_the_land_sea_authority() {
        // Whittaker "water" (id 1) on a class-0 cell → land (Grassland fallback),
        // NOT ocean — regardless of what the Whittaker classifier said.
        assert_eq!(resolve_biome(1, Some(10.0), 50.0, 0.0, 0, false), BiomeTypeEnum::Grassland);
        // Whittaker "land" (id 5) on a class-1 cell → ocean.
        assert_eq!(resolve_biome(5, Some(10.0), -50.0, 0.0, 1, false), BiomeTypeEnum::Ocean);
    }

    #[test]
    fn inland_water_is_lake_or_wetland() {
        // Enclosed inland water (class 2): deep → Lake, shallow depression → Wetland,
        // whatever the Whittaker id / height say.
        assert_eq!(resolve_biome(4, Some(10.0), -30.0, 0.0, 2, false), BiomeTypeEnum::Lake);
        assert_eq!(resolve_biome(4, Some(10.0), -2.0, 0.0, 2, true), BiomeTypeEnum::Wetland);
    }
}
