use shared::grid::GridCell;
use shared::BiomeTypeEnum;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

/// Configuration de densité de graines par biome
pub struct SeedDensityConfig {
    pub grassland: i32,          // 8 cases
    pub forest: i32,              // 10 cases
    pub mountain: i32,            // 15 cases
    pub desert: i32,              // 15 cases
    pub snow: i32,                // 12 cases
    pub default: i32,             // 10 cases
    pub jitter: i32,              // ±2 cases
}

impl Default for SeedDensityConfig {
    fn default() -> Self {
        Self {
            grassland: 8,
            forest: 10,
            mountain: 15,
            desert: 15,
            snow: 12,
            default: 10,
            jitter: 2,
        }
    }
}

impl SeedDensityConfig {
    /// Get spacing for a specific biome
    pub fn get_spacing(&self, biome: BiomeTypeEnum) -> i32 {
        match biome {
            BiomeTypeEnum::Grassland => self.grassland,
            BiomeTypeEnum::TropicalSeasonalForest
            | BiomeTypeEnum::TropicalRainForest
            | BiomeTypeEnum::TropicalDeciduousForest
            | BiomeTypeEnum::TemperateRainForest
            | BiomeTypeEnum::Taiga => self.forest,
            BiomeTypeEnum::Tundra | BiomeTypeEnum::Ice => self.mountain,
            BiomeTypeEnum::Desert | BiomeTypeEnum::ColdDesert => self.desert,
            _ => self.default,
        }
    }
}

/// Generate Voronoi seeds on a regular grid with jitter
/// This version generates seeds without needing biome information upfront
pub fn generate_seeds_simple(
    min_q: i32,
    max_q: i32,
    min_r: i32,
    max_r: i32,
    base_spacing: i32,
    jitter: i32,
    seed: u64,
) -> Vec<GridCell> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut seeds = Vec::new();
    let mut used_cells = HashSet::new();

    for grid_r in (min_r..max_r).step_by(base_spacing as usize) {
        for grid_q in (min_q..max_q).step_by(base_spacing as usize) {
            // Apply random jitter
            let jitter_q = rng.random_range(-jitter..=jitter);
            let jitter_r = rng.random_range(-jitter..=jitter);

            let cell_q = grid_q + jitter_q;
            let cell_r = grid_r + jitter_r;

            // Check bounds and uniqueness
            if cell_q >= min_q && cell_q < max_q && cell_r >= min_r && cell_r < max_r {
                let cell = GridCell { q: cell_q, r: cell_r };
                if used_cells.insert(cell) {
                    seeds.push(cell);
                }
            }
        }
    }

    tracing::info!(
        "Generated {} Voronoi seeds in region ({},{}) to ({},{})",
        seeds.len(),
        min_q,
        min_r,
        max_q,
        max_r
    );

    seeds
}

/// Generate Voronoi seeds with biome-aware spacing
/// This requires a function to query biome at a given cell
pub fn generate_seeds_with_biome<F>(
    min_q: i32,
    max_q: i32,
    min_r: i32,
    max_r: i32,
    config: &SeedDensityConfig,
    biome_query: F,
    seed: u64,
) -> Vec<(GridCell, BiomeTypeEnum)>
where
    F: Fn(GridCell) -> BiomeTypeEnum,
{
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut seeds = Vec::new();
    let mut used_cells = HashSet::new();

    // We need to check all possible grid positions for all biomes
    // Create grids for each biome spacing and merge them
    let spacings = vec![
        config.grassland,
        config.forest,
        config.mountain,
        config.desert,
        config.snow,
    ];

    for &spacing in &spacings {
        // Calculate the first grid point >= min_r that is a multiple of spacing
        let start_r = ((min_r as f32 / spacing as f32).ceil() as i32) * spacing;
        let start_q = ((min_q as f32 / spacing as f32).ceil() as i32) * spacing;

        for grid_r in (start_r..max_r).step_by(spacing as usize) {
            for grid_q in (start_q..max_q).step_by(spacing as usize) {
                // Query biome at this exact position
                let test_cell = GridCell { q: grid_q, r: grid_r };
                let biome = biome_query(test_cell);

                // Only place seed if the biome's spacing matches current grid
                let expected_spacing = config.get_spacing(biome);
                if expected_spacing != spacing {
                    continue; // This position doesn't match the biome's grid
                }
                // Apply random jitter
                let jitter_q = rng.random_range(-config.jitter..=config.jitter);
                let jitter_r = rng.random_range(-config.jitter..=config.jitter);

                let cell_q = grid_q + jitter_q;
                let cell_r = grid_r + jitter_r;

                // Check bounds and uniqueness
                if cell_q >= min_q && cell_q < max_q && cell_r >= min_r && cell_r < max_r {
                    let cell = GridCell { q: cell_q, r: cell_r };
                    if used_cells.insert(cell) {
                        // Re-query biome at actual seed location (after jitter)
                        let seed_biome = biome_query(cell);
                        seeds.push((cell, seed_biome));
                    }
                }
            }
        }
    }

    tracing::info!(
        "Generated {} biome-aware Voronoi seeds in region ({},{}) to ({},{})",
        seeds.len(),
        min_q,
        min_r,
        max_q,
        max_r
    );

    seeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seeds_simple() {
        let seeds = generate_seeds_simple(0, 100, 0, 100, 10, 2, 12345);

        // Should generate roughly (100/10)^2 = 100 seeds
        assert!(seeds.len() > 80 && seeds.len() < 120, "Expected ~100 seeds, got {}", seeds.len());

        // All seeds should be within bounds
        for seed in &seeds {
            assert!(seed.q >= 0 && seed.q < 100);
            assert!(seed.r >= 0 && seed.r < 100);
        }

        // Seeds should be unique
        let unique: HashSet<_> = seeds.iter().collect();
        assert_eq!(unique.len(), seeds.len());
    }

    #[test]
    fn test_seed_density_config() {
        let config = SeedDensityConfig::default();
        assert_eq!(config.get_spacing(BiomeTypeEnum::Grassland), 8);
        assert_eq!(config.get_spacing(BiomeTypeEnum::Desert), 15);
        assert_eq!(config.get_spacing(BiomeTypeEnum::Taiga), 10);
    }
}
