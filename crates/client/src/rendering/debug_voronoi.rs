use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use shared::exploration::voronoi;
use shared::grid::{GridCell, GridConfig};
use std::collections::HashMap;

use crate::camera::MainCamera;
use crate::networking::client::http_client::HttpBulkClient;

// ─── Resources ──────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct OrgVoronoiDebug {
    pub enabled: bool,
    pub seeds: Vec<(i64, GridCell)>,
    pub loaded: bool,
}

#[derive(Resource, Default)]
pub struct MistVoronoiDebug {
    pub enabled: bool,
}

#[derive(Resource)]
pub struct OrgVoronoiReceiver {
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<Vec<(i64, GridCell)>>>,
}

#[derive(Resource, Clone)]
pub struct OrgVoronoiSender {
    tx: std::sync::mpsc::Sender<Vec<(i64, GridCell)>>,
}

pub fn init_voronoi_debug_channels(app: &mut App) {
    let (tx, rx) = std::sync::mpsc::channel();
    app.insert_resource(OrgVoronoiReceiver {
        rx: std::sync::Mutex::new(rx),
    });
    app.insert_resource(OrgVoronoiSender { tx });
}

// ─── Shared drawing logic ───────────────────────────────────────────

/// Draw borders between voronoi zones on hex edges.
/// For each cell, checks its 6 neighbors: if a neighbor has a different zone_id
/// (or is absent from the map), draws the shared hex edge.
fn draw_voronoi_borders(
    gizmos: &mut Gizmos,
    cell_zones: &HashMap<GridCell, i64>,
    layout: &hexx::HexLayout,
) {
    for (cell, &zone_id) in cell_zones {
        let hex = cell.to_hex();
        let edges = layout.hex_edge_corners(hex);
        let neighbors = hex.all_neighbors();

        for (dir, neighbor_hex) in neighbors.iter().enumerate() {
            let neighbor_cell = GridCell::from_hex(neighbor_hex);
            let neighbor_zone = cell_zones.get(&neighbor_cell).copied();

            if neighbor_zone != Some(zone_id) {
                let [corner_a, corner_b] = edges[dir];

                let hue = ((zone_id as u32).wrapping_mul(2654435761) % 360) as f32;
                let color = Color::hsl(hue, 0.7, 0.5).with_alpha(0.6);

                gizmos.line_2d(corner_a, corner_b, color);
            }
        }
    }
}

/// Hex Manhattan distance
fn hex_distance(a: GridCell, b: GridCell) -> i32 {
    let s1 = -a.q - a.r;
    let s2 = -b.q - b.r;
    ((a.q - b.q).abs() + (a.r - b.r).abs() + (s1 - s2).abs()) / 2
}

// ─── F11 — Organization Voronoi ─────────────────────────────────────

pub fn toggle_org_voronoi_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut org_debug: ResMut<OrgVoronoiDebug>,
    http_client: Res<HttpBulkClient>,
    sender: Res<OrgVoronoiSender>,
) {
    if keyboard.just_pressed(KeyCode::F11) {
        org_debug.enabled = !org_debug.enabled;
        let state = if org_debug.enabled { "ON" } else { "OFF" };
        info!("Org Voronoi debug: {}", state);

        if org_debug.enabled && !org_debug.loaded {
            let client = http_client.client.clone();
            let base_url = http_client.base_url.clone();
            let sender = sender.clone();

            IoTaskPool::get()
                .spawn(async_compat::Compat::new(async move {
                    match client
                        .get(format!("{}/api/debug/voronoi-seeds", base_url))
                        .send()
                        .await
                    {
                        Ok(response) => match response.json::<Vec<serde_json::Value>>().await {
                            Ok(json) => {
                                let seeds: Vec<(i64, GridCell)> = json
                                    .iter()
                                    .filter_map(|v| {
                                        Some((
                                            v.get("zone_id")?.as_i64()?,
                                            GridCell {
                                                q: v.get("q")?.as_i64()? as i32,
                                                r: v.get("r")?.as_i64()? as i32,
                                            },
                                        ))
                                    })
                                    .collect();
                                bevy::log::info!(
                                    "Loaded {} org voronoi seeds from server",
                                    seeds.len()
                                );
                                let _ = sender.tx.send(seeds);
                            }
                            Err(e) => {
                                bevy::log::error!("Failed to parse voronoi seeds JSON: {}", e)
                            }
                        },
                        Err(e) => bevy::log::error!("Failed to fetch voronoi seeds: {}", e),
                    }
                }))
                .detach();
        }
    }
}

pub fn poll_org_voronoi_seeds(
    receiver: Res<OrgVoronoiReceiver>,
    mut org_debug: ResMut<OrgVoronoiDebug>,
) {
    if let Ok(rx) = receiver.rx.lock() {
        while let Ok(seeds) = rx.try_recv() {
            org_debug.seeds = seeds;
            org_debug.loaded = true;
        }
    }
}

pub fn draw_org_voronoi_debug(
    mut gizmos: Gizmos,
    org_debug: Res<OrgVoronoiDebug>,
    grid_config: Res<GridConfig>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    if !org_debug.enabled || org_debug.seeds.is_empty() {
        return;
    }
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_tf.translation.truncate();

    let center_hex = grid_config.layout.world_pos_to_hex(camera_pos);
    let view_radius = 30;

    // For each visible cell, find the closest org seed → assign zone_id
    let mut cell_zones: HashMap<GridCell, i64> = HashMap::new();

    for dq in -view_radius..=view_radius {
        for dr in -view_radius..=view_radius {
            let cell = GridCell {
                q: center_hex.x + dq,
                r: center_hex.y + dr,
            };

            let mut min_dist = i32::MAX;
            let mut closest_zone = 0i64;

            for (zone_id, seed_cell) in &org_debug.seeds {
                let dist = hex_distance(cell, *seed_cell);
                if dist < min_dist {
                    min_dist = dist;
                    closest_zone = *zone_id;
                }
            }

            cell_zones.insert(cell, closest_zone);
        }
    }

    draw_voronoi_borders(&mut gizmos, &cell_zones, &grid_config.layout);
}

// ─── F12 — Mist/Exploration Voronoi ─────────────────────────────────

pub fn toggle_mist_voronoi_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mist_debug: ResMut<MistVoronoiDebug>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        mist_debug.enabled = !mist_debug.enabled;
        let state = if mist_debug.enabled { "ON" } else { "OFF" };
        info!("Mist Voronoi debug: {}", state);
    }
}

pub fn draw_mist_voronoi_debug(
    mut gizmos: Gizmos,
    mist_debug: Res<MistVoronoiDebug>,
    grid_config: Res<GridConfig>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    if !mist_debug.enabled {
        return;
    }
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_tf.translation.truncate();

    let center_hex = grid_config.layout.world_pos_to_hex(camera_pos);
    let view_radius = 40;

    let min_q = center_hex.x - view_radius;
    let max_q = center_hex.x + view_radius;
    let min_r = center_hex.y - view_radius;
    let max_r = center_hex.y + view_radius;

    // Compute seeds in visible area + margin
    let margin = voronoi::EXPLORATION_SEED_SPACING + 2;
    let seeds = voronoi::compute_seeds_in_region(
        min_q - margin,
        max_q + margin,
        min_r - margin,
        max_r + margin,
        &grid_config.layout,
        voronoi::EXPLORATION_WORLD_SEED,
    );

    // For each visible cell, find nearest seed by world-space euclidean distance
    let mut cell_zones: HashMap<GridCell, i64> = HashMap::new();

    for q in min_q..max_q {
        for r in min_r..max_r {
            let cell = GridCell { q, r };
            let cell_world = grid_config.layout.hex_to_world_pos(cell.to_hex());

            let mut min_dist_sq = f32::MAX;
            let mut closest_zone = 0i64;

            for seed in &seeds {
                let dx = cell_world.x - seed.x;
                let dy = cell_world.y - seed.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    closest_zone = seed.zone_id;
                }
            }

            cell_zones.insert(cell, closest_zone);
        }
    }

    draw_voronoi_borders(&mut gizmos, &cell_zones, &grid_config.layout);
}
