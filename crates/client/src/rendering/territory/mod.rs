pub mod cache;
pub mod materials;
pub mod systems;

pub use cache::*;
pub use materials::*;
pub use systems::*;

use bevy::{prelude::*, sprite_render::Material2dPlugin};

/// Plugin for rendering territory borders
pub struct TerritoryBorderPlugin;

impl Plugin for TerritoryBorderPlugin {
    fn build(&self, app: &mut App) {
        app
            // Materials
            .add_plugins(Material2dPlugin::<TerritoryBorderMaterial>::default())
            .add_plugins(Material2dPlugin::<TerritoryMaterial>::default())
            .add_plugins(Material2dPlugin::<TerritoryChunkMaterial>::default())
            // Resources
            .init_resource::<TerritoryBorderSdfCache>()
            .init_resource::<TerritoryContourCache>()
            // Systems
            .add_systems(Update, (
                // Old SDF-based system (deprecated)
                update_territory_border_time,
                process_territory_border_sdf_data,
                debug_territory_borders,
                visualize_border_cells,
                // New contour-based system
                render_territory_contours,
            ));
    }
}
