use bevy::prelude::*;

pub const HEX_SIZE: f32 = 36.0; // total width = 2 * size = 2 * 36px = 72px
pub const HEX_RATIO: Vec2 = Vec2::new(1.0, 0.866); // isometric ratio (√3/2)

pub const CHUNK_SIZE: Vec2 = Vec2::new(900., 756.);

/// Normalized height threshold below which terrain is considered water.
/// Used for SDF generation, biome classification, and shader ocean rendering.
/// Value 0.003 normalized ≈ 7.5m at max_elevation 2500m.
pub const WATER_HEIGHT_THRESHOLD: f32 = 0.003;