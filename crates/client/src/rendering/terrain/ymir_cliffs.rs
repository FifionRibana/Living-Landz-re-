//! LL-C: debug overlay drawing Ymir's `cliffs.geojson` polylines (producer-side
//! ground truth) over the terrain, to validate the LL-B metric slope / cliff
//! threshold against Ymir's own cliff edges.
//!
//! The cliff polylines are read directly from the `.ymir` asset on disk (the map
//! name comes from `TerrainGlobalData.name`), parsed with the shared GeoJSON
//! reader, and drawn as world-space gizmo lines. Toggled via the F4 debug panel
//! (`DebugOverlayState::ymir_cliffs`). No-op for maps without a `.ymir` (Azgaar).

use bevy::prelude::*;

use crate::state::resources::WorldCache;
use crate::ui::debug::DebugOverlayState;

/// Lazily-loaded, per-map cache of the cliff polylines (cell space, y=0 south).
#[derive(Resource, Default)]
pub struct YmirCliffsCache {
    /// The map name the polylines were loaded for (`TerrainGlobalData.name`).
    loaded_for: Option<String>,
    /// Parsed cliff polylines in cell space; empty when the map has no cliffs.
    polylines: Vec<Vec<[f32; 2]>>,
}

const CLIFF_COLOR: Color = Color::srgba(1.0, 0.35, 0.05, 0.9); // bright orange

/// Try both plausible working directories for the `.ymir` cliff file (the client
/// runs with cwd = `crates/client/` in dev, but repo-root when run from there).
fn read_cliffs_file(map_name: &str) -> Option<String> {
    let rel = format!("assets/maps/{map_name}.ymir/cliffs.geojson");
    for base in ["../../", ""] {
        let path = format!("{base}{rel}");
        if let Ok(s) = std::fs::read_to_string(&path) {
            return Some(s);
        }
    }
    None
}

/// Draw the Ymir cliff polylines as world-space gizmo lines when the overlay is
/// enabled. Loads + parses the file once per map (cached).
pub fn draw_ymir_cliffs_gizmos(
    mut gizmos: Gizmos,
    debug: Res<DebugOverlayState>,
    world_cache: Option<Res<WorldCache>>,
    mut cache: ResMut<YmirCliffsCache>,
) {
    if !debug.ymir_cliffs {
        return;
    }
    let Some(world_cache) = world_cache else {
        return;
    };
    let Some(tg) = world_cache.get_terrain_global() else {
        return;
    };

    // Lazy (re)load when the loaded map changes.
    if cache.loaded_for.as_deref() != Some(tg.name.as_str()) {
        let polylines = read_cliffs_file(&tg.name)
            .map(|raw| shared::geojson::parse_multilinestring_geojson(&raw))
            .unwrap_or_default();
        info!(
            "Ymir cliffs overlay: loaded {} polylines for map '{}'",
            polylines.len(),
            tg.name
        );
        cache.polylines = polylines;
        cache.loaded_for = Some(tg.name.clone());
    }

    if cache.polylines.is_empty() {
        return;
    }

    // Cell (x in [0,gw], y=0 south) → world space, with the same Y-flip the server
    // applies to the heightmap/biome textures (see build_ymir_globals).
    let gw = tg.heightmap_width.max(1) as f32;
    let gh = tg.heightmap_height.max(1) as f32;
    let ww = tg.world_width;
    let wh = tg.world_height;
    let to_world = |c: &[f32; 2]| Vec2::new(c[0] / gw * ww, (gh - c[1]) / gh * wh);

    for line in &cache.polylines {
        for seg in line.windows(2) {
            gizmos.line_2d(to_world(&seg[0]), to_world(&seg[1]), CLIFF_COLOR);
        }
    }
}
