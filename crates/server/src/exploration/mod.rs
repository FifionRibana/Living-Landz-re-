mod rasterizer;
mod voronoi_gen;

pub use rasterizer::*;
pub use voronoi_gen::*;

use shared::grid::{GridCell, GridConfig};
use shared::UnitData;

use crate::database::client::DatabaseTables;

/// Ensure the area around the lord's spawn is explored (voronoi zones).
pub async fn ensure_spawn_explored(
    lord: Option<UnitData>,
    db_tables: &DatabaseTables,
    player_id: i64,
    grid_config: &GridConfig,
) {
    if let Some(ref lord) = lord {
        let spawn_cell = GridCell {
            q: lord.current_cell.q,
            r: lord.current_cell.r,
        };
        let world_seed = EXPLORATION_WORLD_SEED;

        let zone_ids =
            zones_in_radius(&spawn_cell, 20, &grid_config.layout, world_seed);

        match db_tables
            .exploration_voronoi
            .mark_explored(&zone_ids, player_id)
            .await
        {
            Ok(newly) => {
                if !newly.is_empty() {
                    tracing::info!(
                        "Explored {} voronoi zones around spawn ({},{})",
                        newly.len(),
                        spawn_cell.q,
                        spawn_cell.r
                    );
                }
            }
            Err(e) => tracing::error!("Failed to explore spawn area: {}", e),
        }
    }
}