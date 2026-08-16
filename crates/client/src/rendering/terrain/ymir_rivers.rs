//! LL-E: debug overlay drawing Ymir's `rivers.json` graph over the terrain, to
//! validate the cell→world mapping and inspect Strahler order / navigability.
//!
//! Segments are read directly from the `.ymir` asset (map name from
//! `TerrainGlobalData.name`), parsed with the shared river reader, and drawn as
//! world-space gizmo lines coloured by **Strahler order**; **navigable** reaches
//! (SmallBoat and larger, as authored by Ymir) are highlighted in cyan. Toggled
//! via the F4 debug panel (`DebugOverlayState::ymir_rivers`). No-op for maps
//! without a `.ymir` (Azgaar).

use bevy::prelude::*;

use shared::rivers::RiverNetwork;

use crate::state::resources::WorldCache;
use crate::ui::debug::DebugOverlayState;

/// Lazily-loaded, per-map cache of the parsed river graph.
#[derive(Resource, Default)]
pub struct YmirRiversCache {
    loaded_for: Option<String>,
    network: RiverNetwork,
}

/// Cyan for navigable reaches (any class ≥ SmallBoat).
const NAVIGABLE_COLOR: Color = Color::srgba(0.2, 0.95, 0.95, 0.95);

/// Strahler order → colour (headwater = pale, trunk = deep blue).
fn strahler_color(order: u32) -> Color {
    match order {
        0 | 1 => Color::srgba(0.45, 0.60, 0.85, 0.65),
        2 => Color::srgba(0.30, 0.52, 0.90, 0.75),
        3 => Color::srgba(0.20, 0.46, 0.95, 0.85),
        4 => Color::srgba(0.12, 0.38, 1.00, 0.90),
        5 => Color::srgba(0.06, 0.28, 0.92, 0.95),
        _ => Color::srgba(0.02, 0.18, 0.80, 1.00),
    }
}

/// Try both plausible working directories for the `.ymir` rivers file (client cwd
/// is `crates/client/` in dev, or repo-root when run from there).
fn read_rivers_file(map_name: &str) -> Option<String> {
    let rel = format!("assets/maps/{map_name}.ymir/rivers.json");
    for base in ["../../", ""] {
        let path = format!("{base}{rel}");
        if let Ok(s) = std::fs::read_to_string(&path) {
            return Some(s);
        }
    }
    None
}

/// Draw the Ymir river segments as world-space gizmo lines when the overlay is
/// enabled. Loads + parses the file once per map (cached).
pub fn draw_ymir_rivers_gizmos(
    mut gizmos: Gizmos,
    debug: Res<DebugOverlayState>,
    world_cache: Option<Res<WorldCache>>,
    mut cache: ResMut<YmirRiversCache>,
) {
    if !debug.ymir_rivers {
        return;
    }
    let Some(world_cache) = world_cache else {
        return;
    };
    let Some(tg) = world_cache.get_terrain_global() else {
        return;
    };

    if cache.loaded_for.as_deref() != Some(tg.name.as_str()) {
        let network = read_rivers_file(&tg.name)
            .map(|raw| RiverNetwork::parse(&raw))
            .unwrap_or_default();
        info!(
            "Ymir rivers overlay: loaded {} segments for map '{}'",
            network.segments.len(),
            tg.name
        );
        cache.network = network;
        cache.loaded_for = Some(tg.name.clone());
    }

    if cache.network.segments.is_empty() {
        return;
    }

    // Cell (x in [0,gw], y=0 south) → world, with the same Y-flip the server
    // applies to the heightmap/biome textures (matches ymir_cliffs).
    let gw = tg.heightmap_width.max(1) as f32;
    let gh = tg.heightmap_height.max(1) as f32;
    let ww = tg.world_width;
    let wh = tg.world_height;
    let to_world = |c: &[f32; 2]| Vec2::new(c[0] / gw * ww, (gh - c[1]) / gh * wh);

    for seg in &cache.network.segments {
        let color = if seg.navigability.is_navigable() {
            NAVIGABLE_COLOR
        } else {
            strahler_color(seg.strahler_order)
        };
        for w in seg.points.windows(2) {
            gizmos.line_2d(to_world(&w[0]), to_world(&w[1]), color);
        }
    }
}
