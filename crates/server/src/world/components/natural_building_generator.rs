use std::collections::HashMap;

use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use shared::{
    BiomeType, BuildingBaseData, BuildingCategory, BuildingData, BuildingSpecific, BuildingType,
    GameState, TreeData, TreeType,
    grid::{CellData, GridCell},
};

// TODO Move into utils
struct NoiseGenerator {
    distribution: Perlin,
    quality: Perlin,
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            distribution: Perlin::new(seed),
            quality: Perlin::new(seed + 1),
        }
    }

    pub fn sample_distribution(&self, cell: &GridCell) -> f32 {
        let scale = 0.05;
        self.distribution
            .get([cell.q as f64 * scale, cell.r as f64 * scale]) as f32
    }

    pub fn sample_quality(&self, cell: &GridCell) -> f32 {
        let scale = 0.02;
        self.quality
            .get([cell.q as f64 * scale, cell.r as f64 * scale]) as f32
    }
}

pub struct NaturalBuildingGenerator {
    pub buildings: HashMap<GridCell, BuildingData>,
}

impl NaturalBuildingGenerator {
    pub fn generate(cells: &[CellData], game_state: &GameState) -> Self {
        let mut generator = Self {
            buildings: HashMap::new(),
        };

        generator.generate_trees(cells, game_state);

        generator.compute_tree_density();

        generator
    }

    pub fn generate_trees(&mut self, cells: &[CellData], game_state: &GameState) {
        let tree_generator = NoiseGenerator::new(12345);

        let trees: Vec<(GridCell, BuildingData)> = cells
            .par_iter()
            .filter_map(|cell_data| {
                if tree_generator.sample_distribution(&cell_data.cell)
                    < (Self::get_tree_spawn_chance(cell_data.biome) - 1.0)
                {
                    return None;
                }

                let id = Self::generate_building_id(&cell_data.cell);
                let mut rng = rand::rngs::StdRng::seed_from_u64(id);

                let spawnable_trees = TreeType::from_biome(cell_data.biome);
                if spawnable_trees.len() == 0 {
                    return None;
                }

                let tree_type_idx = rng.random_range(..spawnable_trees.len());

                let tree_type = spawnable_trees[tree_type_idx];
                let tree_variations = game_state
                    .tree_atlas
                    .get_variations(tree_type)
                    .expect(format!("No variations for {:?} type", tree_type).as_str());

                let tree_variant_idx = rng.random_range(..tree_variations.len());
                let tree_variant = tree_variations[tree_variant_idx].clone();

                let type_id = game_state
                    .get_building_type_id(&tree_type.to_name())
                    .expect(format!("Invalid building type {:?}", tree_type).as_str());

                let base_data = BuildingBaseData {
                    id,
                    building_type: BuildingType::tree(tree_type, tree_variant, type_id),
                    chunk: cell_data.chunk,
                    cell: cell_data.cell,
                    created_at: Self::timestamp(),
                    quality: 1.0,
                    durability: 1.0,
                    damage: 0.0,
                };

                let specific_data = BuildingSpecific::Tree(TreeData {
                    density: 0.5,
                    age: rng.random_range(0..200),
                    tree_type,
                    variant: (tree_variant_idx + 1) as i32,
                });

                Some((
                    cell_data.cell,
                    BuildingData {
                        base_data,
                        specific_data,
                    },
                ))
            })
            .collect();
        
        tracing::info!("{} trees generated", &trees.len());

        self.buildings.extend(trees);
        
    }

    fn compute_tree_density(&mut self) {
        let density_map = self
            .buildings
            .par_iter()
            .filter_map(|(grid_cell, building)| {
                if !matches!(&building.specific_data, BuildingSpecific::Tree(_tree_data)) {
                    return None;
                }

                let neighbors = grid_cell.neighbors();
                let tree_neighbors = neighbors
                    .iter()
                    .filter(|n| {
                        self.buildings
                            .get(n)
                            .map(|b| matches!(&b.specific_data, BuildingSpecific::Tree(_)))
                            .unwrap_or(false)
                    })
                    .count();

                let density = ((tree_neighbors as f32 / 6.0) * 0.7 + 0.3).clamp(0.3, 1.0);
                Some((*grid_cell, density))
            })
            .collect::<Vec<(GridCell, f32)>>();

        for (cell, density) in density_map {
            if let Some(building) = self.buildings.get_mut(&cell) {
                if let BuildingSpecific::Tree(tree_data) = &mut building.specific_data {
                    tree_data.density = density;
                }
            }
        }
    }

    // TODO: Move to utils
    fn generate_building_id(cell: &GridCell) -> u64 {
        let mut hash = 0x517cc1b727220a95u64;
        hash ^= cell.q as u64;
        hash = hash.wrapping_mul(0x3243f6a8885a308d);
        hash ^= cell.r as u64;
        hash = hash.wrapping_mul(0x3243f6a8885a308d);
        hash
    }

    fn get_tree_spawn_chance(biome: BiomeType) -> f32 {
        match biome {
            BiomeType::Savanna => 0.1,
            BiomeType::Grassland => 0.3,
            BiomeType::TropicalSeasonalForest => 0.6,
            BiomeType::TropicalRainForest => 0.6,
            BiomeType::TropicalDeciduousForest => 0.6,
            BiomeType::TemperateRainForest => 0.6,
            BiomeType::Wetland => 0.45,
            BiomeType::Taiga => 0.4,
            _ => 0.0,
        }
    }

    // TODO: Move to utils
    fn timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
