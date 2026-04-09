// =============================================================================
// UI - HUD (Heads-Up Display)
// =============================================================================

use crate::camera::MainCamera;
use crate::state::resources::{ConnectionStatus, WorldCache};

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::{BiomeTypeEnum, ShoreType};

use super::components::*;

use shared::grid::{GridCell, GridConfig};

pub fn setup_debug_ui(mut commands: Commands) {
    // Root node pour HUD
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                padding: UiRect{
                    left: px(10.),
                    top: px(100.),
                    right: px(0.),
                    bottom: px(0.),
                },
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .with_children(|parent| {
            // Top-left: FPS
            parent
                .spawn((
                    Text::new("FPS: -- (avg: --)"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(64, 230, 120)),
                    Node {
                        ..default()
                    },
                    FpsText,
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ))
                .observe(|_over: On<Pointer<Over>>| {
                    println!("oveerd");
                });

            parent.spawn((
                Text::new("Frame time: --ms"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                FrameTimeText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Connection status: -- | id: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                ConnectionStatusText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Total entities: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                EntityCountText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Position: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                CameraPositionText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Zoom level: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                CameraZoomText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Cell: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                HoveredCellInfoText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            // Spacer
            parent.spawn((
                Node {
                    height: px(20.0),
                    ..default()
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            // Debug commands help
            parent.spawn((
                Text::new("=== DEBUG COMMANDS ===\nShift+M: Hamlet\nShift+V: Village\nShift+T: Town\nShift+C: City\nShift+U: Spawn Unit\nShift+D: Delete Org\nShift+H: Help"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(64, 230, 120)),
                Node {
                    ..default()
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));
        });
}

pub fn update_debug_ui(
    diagnostics: Res<DiagnosticsStore>,
    cameras: Query<(&Camera, &GlobalTransform, &Transform, &Projection), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    connection_status: Res<ConnectionStatus>,
    grid_config: Res<GridConfig>,
    world_cache: Option<Res<WorldCache>>,
    images: Res<Assets<Image>>,
    mut query: Query<(
        &mut Text,
        Option<&FpsText>,
        Option<&FrameTimeText>,
        Option<&ConnectionStatusText>,
        Option<&EntityCountText>,
        Option<&CameraPositionText>,
        Option<&CameraZoomText>,
        Option<&HoveredCellInfoText>,
    )>,
) {
    let (fps_value, average_fps) =
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                (fps.value().unwrap_or(0.0), value)
            } else {
                (fps.value().unwrap_or(0.0), -1.0)
            }
        } else {
            (-1.0, -1.0)
        };

    let frame_time_value =
        if let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
            if let Some(value) = frame_time.smoothed() {
                value * 1000.0
            } else {
                -1.0
            }
        } else {
            -1.0
        };

    let (status_logged_in, status_player_id) = if connection_status.logged_in {
        let player_id = if let Some(player_id) = connection_status.player_id {
            format!("{}", player_id)
        } else {
            "--".to_string()
        };

        ("connected".to_string(), player_id)
    } else {
        ("disconnected".to_string(), "--".to_string())
    };

    let entity_count_value =
        if let Some(entity_count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
            entity_count.value().unwrap_or(0.0) as usize
        } else {
            0.0 as usize
        };

    let Ok((camera, global_transform, transform, projection)) = cameras.single() else {
        return;
    };

    let scale = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let position = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(global_transform, p).ok())
        .unwrap_or_default();

    for (
        mut text,
        fps_query,
        frame_time_query,
        connection_status_query,
        entity_count_query,
        position_query,
        zoom_query,
        hovered_cell_query,
    ) in &mut query
    {
        if fps_query.is_some() {
            **text = format!("FPS: {:.1} (avg: {:.1})", fps_value, average_fps);
        } else if frame_time_query.is_some() {
            **text = format!("Frame time: {:.2}ms", frame_time_value);
        } else if entity_count_query.is_some() {
            **text = format!("Total entities: {}", entity_count_value);
        } else if position_query.is_some() {
            **text = format!(
                "Position: ({:.0}, {:.0})",
                transform.translation.x, transform.translation.y
            );
        } else if zoom_query.is_some() {
            **text = format!("Zoom level: {:.2}", scale);
        } else if connection_status_query.is_some() {
            **text = format!("Status: {} | id: {}", status_logged_in, status_player_id);
        } else if hovered_cell_query.is_some() {
            let hovered_cell = grid_config
                .layout
                .world_pos_to_hex(Vec2::new(position.x, position.y));

            let grid_cell = &GridCell {
                q: hovered_cell.x,
                r: hovered_cell.y,
            };
            if let Some(world_cache) = &world_cache {
                let (biome, shore_type) = match world_cache.get_cell(grid_cell) {
                    Some(cell_data) => (cell_data.biome, cell_data.shore_type),
                    _ => (BiomeTypeEnum::Undefined, ShoreType::None)
                };

                // Sample altitude from the heightmap texture
                let alt_str = sample_altitude_at(
                    world_cache.as_ref(),
                    &images,
                    position.x,
                    position.y,
                );

                **text = format!(
                    "Cell: (q: {}, r: {})\nBiome: {:?}\nShore: {:?}\n{}",
                    hovered_cell.x, hovered_cell.y, biome, shore_type, alt_str
                );
            }
        }
    }
}

/// Sample the enriched heightmap at a world position and return a display string.
fn sample_altitude_at(
    world_cache: &WorldCache,
    images: &Assets<Image>,
    world_x: f32,
    world_y: f32,
) -> String {
    let global = match world_cache.get_terrain_global() {
        Some(g) => g,
        None => return String::new(),
    };
    let hm_handle = match world_cache.get_terrain_global_heightmap_handle() {
        Some(h) => h,
        None => return String::new(),
    };
    let image = match images.get(hm_handle) {
        Some(img) => img,
        None => return String::new(),
    };

    let hm_w = global.heightmap_width as usize;
    let hm_h = global.heightmap_height as usize;

    // World position → UV (0-1 over entire map)
    let u = world_x / global.world_width;
    let v = world_y / global.world_height;

    let px = (u * hm_w as f32) as i32;
    let py = (v * hm_h as f32) as i32;

    if px < 0 || py < 0 || px >= hm_w as i32 || py >= hm_h as i32 {
        return String::new();
    }

    let px = px as usize;
    let py = py as usize;
    let data = match image.data.as_ref() {
        Some(d) => d,
        None => return String::new(),
    };

    // Detect format: R16Unorm = 2 bytes/pixel, R8Unorm = 1 byte/pixel
    let expected_u16_len = hm_w * hm_h * 2;
    let is_u16 = data.len() >= expected_u16_len;
    let (height_norm, raw_val) = if is_u16 {
        let idx = (py * hm_w + px) * 2;
        if idx + 1 >= data.len() {
            return String::new();
        }
        let val = u16::from_le_bytes([data[idx], data[idx + 1]]);
        (val as f32 / 65535.0, val as u32)
    } else {
        let idx = py * hm_w + px;
        if idx >= data.len() {
            return String::new();
        }
        (data[idx] as f32 / 255.0, data[idx] as u32)
    };

    let altitude_m = (height_norm * 2500.0).round() as i32;

    if altitude_m == 0 {
        "Alt: sea level".to_string()
    } else {
        format!("Alt: {}m", altitude_m)
    }
}
