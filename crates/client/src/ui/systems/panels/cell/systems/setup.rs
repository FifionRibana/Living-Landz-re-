use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::resources::CELL_SCENE_LAYER;
use crate::states::GameView;
use crate::ui::{
    components::CellSceneVisual,
    resources::CellState,
    systems::{
        load_building_background, load_separators, load_terrain_background,
        panels::components::CellViewPanel,
    },
};

pub fn setup_cell_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        // No BackgroundColor — the CellSceneDisplay sprite provides the background
        DespawnOnExit(GameView::Cell),
        CellViewPanel,
        Pickable {
            should_block_lower: false,
            is_hoverable: false
        }
    ));
}

pub fn setup_cell_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    windows: Query<&Window>,
    old_visuals: Query<Entity, With<CellSceneVisual>>,
) {
    let Some(_viewed_cell) = cell_state.cell() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    // Despawn previous scene visuals
    for entity in &old_visuals {
        commands.entity(entity).despawn();
    }

    let screen_w = window.width();
    let screen_h = window.height();
    let biome = cell_state.biome();

    info!("Setting up cell scene visuals");

    // 1. Terrain background — full screen Sprite
    let terrain_bg = load_terrain_background(&asset_server, biome);
    commands.spawn((
        Sprite {
            image: terrain_bg,
            custom_size: Some(Vec2::new(screen_w, screen_h)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        CellSceneVisual,
        CELL_SCENE_LAYER,
    ));

    // 2. Building background + separators
    if let Some(building_data) = cell_state.building_data {
        let building_bg = load_building_background(&asset_server, &building_data);

        if cell_state.has_interior() {
            // Building: square centered, height-based
            let side = screen_h;
            commands.spawn((
                Sprite {
                    image: building_bg,
                    custom_size: Some(Vec2::new(side, side)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ));

            // Separators
            let (left_sep, right_sep) = load_separators(&asset_server, Some(&building_data));
            let sep_width = screen_h * 0.08;

            commands.spawn((
                Sprite {
                    image: left_sep,
                    custom_size: Some(Vec2::new(sep_width, screen_h)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-side / 2.0, 0.0, 2.0)),
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ));

            commands.spawn((
                Sprite {
                    image: right_sep,
                    custom_size: Some(Vec2::new(sep_width, screen_h)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(side / 2.0, 0.0, 2.0)),
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ));
        } else {
            // No interior — full screen
            commands.spawn((
                Sprite {
                    image: building_bg,
                    custom_size: Some(Vec2::new(screen_w, screen_h)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ));
        }
    }
}

pub fn cleanup_cell_scene_visuals(
    mut commands: Commands,
    visuals: Query<Entity, With<CellSceneVisual>>,
) {
    for entity in &visuals {
        commands.entity(entity).despawn();
    }
}
