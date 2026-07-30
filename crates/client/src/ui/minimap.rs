// =============================================================================
// UI — Minimap
// =============================================================================
//
// A bottom-right overview of the whole world (reusing the global biome texture)
// with a rectangle marking the camera's currently-visible area.
//
// v1: display only (no click-to-move). The map image is the existing global
// biome texture (`WorldCache::get_terrain_global_biome_handle`), so nothing new
// is generated. World → minimap mapping mirrors the terrain shader
// (`uv = world_pos / (world_width, world_height)`); the image is flipped
// vertically so world-north is up (UI V grows downward, world Y grows upward).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::state::resources::WorldCache;
use crate::states::AppState;

/// Longest on-screen side of the minimap, in logical px.
const MINIMAP_MAX_SIDE: f32 = 220.0;
/// Margin from the screen edges, in logical px.
const MINIMAP_MARGIN: f32 = 12.0;

/// Root container (anchored bottom-right; sized to the world aspect ratio).
#[derive(Component)]
pub struct MinimapRoot;

/// The whole-world image (global biome texture).
#[derive(Component)]
pub struct MinimapImage;

/// The rectangle marking the camera's visible area.
#[derive(Component)]
pub struct MinimapViewport;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_minimap)
            .add_systems(
                Update,
                (update_minimap_image, update_minimap_viewport)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// Spawn the minimap hierarchy once, hidden until the global biome texture is
/// available (populated by `update_minimap_image`).
fn spawn_minimap(mut commands: Commands, existing: Query<Entity, With<MinimapRoot>>) {
    if !existing.is_empty() {
        return; // already spawned (re-entered InGame)
    }

    commands
        .spawn((
            MinimapRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(MINIMAP_MARGIN),
                bottom: Val::Px(MINIMAP_MARGIN),
                width: Val::Px(MINIMAP_MAX_SIDE),
                height: Val::Px(MINIMAP_MAX_SIDE),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                display: Display::None, // shown once the biome texture is set
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.55)),
            GlobalZIndex(900),
        ))
        .with_children(|root| {
            // Whole-world image (biome texture handle set later). Flipped
            // vertically so world-north is at the top of the minimap.
            root.spawn((
                MinimapImage,
                ImageNode {
                    flip_y: true,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));

            // Visible-area rectangle (position/size set each frame).
            root.spawn((
                MinimapViewport,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Percent(0.0),
                    width: Val::Percent(20.0),
                    height: Val::Percent(20.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BorderColor::all(Color::srgba(1.0, 0.95, 0.3, 0.95)),
                BackgroundColor(Color::srgba(1.0, 0.95, 0.3, 0.12)),
            ));
        });
}

/// Once the global biome texture exists, assign it to the minimap image, size
/// the container to the world aspect ratio, and reveal it.
fn update_minimap_image(
    world_cache: Option<Res<WorldCache>>,
    mut image: Query<&mut ImageNode, With<MinimapImage>>,
    mut root: Query<&mut Node, With<MinimapRoot>>,
) {
    let Some(cache) = world_cache else {
        return;
    };
    let Some(handle) = cache.get_terrain_global_biome_handle().cloned() else {
        return;
    };
    let Some((world_w, world_h)) = cache
        .get_terrain_global()
        .map(|g| (g.world_width, g.world_height))
    else {
        return;
    };
    if world_w <= 0.0 || world_h <= 0.0 {
        return;
    }

    let Ok(mut image) = image.single_mut() else {
        return;
    };
    if image.image.id() == handle.id() {
        return; // already up to date
    }
    image.image = handle;

    // Fit the world into MINIMAP_MAX_SIDE without distortion.
    let (w, h) = if world_w >= world_h {
        (MINIMAP_MAX_SIDE, MINIMAP_MAX_SIDE * world_h / world_w)
    } else {
        (MINIMAP_MAX_SIDE * world_w / world_h, MINIMAP_MAX_SIDE)
    };
    if let Ok(mut node) = root.single_mut() {
        node.width = Val::Px(w);
        node.height = Val::Px(h);
        node.display = Display::Flex;
    }
}

/// Position/size the viewport rectangle from the camera's visible world extent.
fn update_minimap_viewport(
    world_cache: Option<Res<WorldCache>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut viewport: Query<&mut Node, With<MinimapViewport>>,
) {
    let Some(cache) = world_cache else {
        return;
    };
    let Some((world_w, world_h)) = cache
        .get_terrain_global()
        .map(|g| (g.world_width, g.world_height))
    else {
        return;
    };
    if world_w <= 0.0 || world_h <= 0.0 {
        return;
    }

    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let (sw, sh) = (window.width(), window.height());

    // Visible world rectangle from two opposite screen corners.
    let (Ok(c0), Ok(c1)) = (
        camera.viewport_to_world_2d(camera_transform, Vec2::ZERO),
        camera.viewport_to_world_2d(camera_transform, Vec2::new(sw, sh)),
    ) else {
        return;
    };
    let min_x = c0.x.min(c1.x);
    let max_x = c0.x.max(c1.x);
    let min_y = c0.y.min(c1.y);
    let max_y = c0.y.max(c1.y);

    // World → minimap fractions. X direct; Y inverted because the image is
    // flipped vertically (world-north up ↔ UI top).
    let left = (min_x / world_w).clamp(0.0, 1.0);
    let right = (max_x / world_w).clamp(0.0, 1.0);
    let top = (1.0 - max_y / world_h).clamp(0.0, 1.0);
    let bottom = (1.0 - min_y / world_h).clamp(0.0, 1.0);

    if let Ok(mut node) = viewport.single_mut() {
        node.left = Val::Percent(left * 100.0);
        node.top = Val::Percent(top * 100.0);
        node.width = Val::Percent((right - left).max(0.0) * 100.0);
        node.height = Val::Percent((bottom - top).max(0.0) * 100.0);
    }
}
