use anyhow::Result;
use bevy::prelude::*;
use image::DynamicImage;

use shared::constants;

use super::WorldConfig;

#[derive(Resource)]
pub struct WorldMaps {
    pub heightmap: DynamicImage,
    pub biome_map: DynamicImage,
    pub binary_map: DynamicImage,
    pub config: WorldConfig,
}

impl WorldMaps {
    pub fn load(map_name: &str, seed: u32) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Loading world maps...");

        let heightmap = image::open(format!("assets/maps/{}_heightmap.png", map_name))?;
        let biome_map = image::open(format!("assets/maps/{}_biomemap.png", map_name))?;
        let binary_map = image::open(format!("assets/maps/{}_binarymap.png", map_name))?;

        let width = heightmap.width();
        let height = heightmap.height();

        if biome_map.width() != width || biome_map.height() != height {
            return Err("All maps must have same dimensions".into());
        }

        let config = WorldConfig {
            map_width: width,
            map_height: height,
            chunks_x: (width as f32 / constants::CHUNK_SIZE.x) as u32,
            chunks_y: (height as f32 / constants::CHUNK_SIZE.y) as u32,
            seed,
        };

        tracing::info!(
            "✓ Maps: {}x{} → {}x{} chunks",
            width,
            height,
            config.chunks_x,
            config.chunks_y
        );

        Ok(Self {
            heightmap,
            biome_map,
            binary_map,
            config,
        })
    }
}
