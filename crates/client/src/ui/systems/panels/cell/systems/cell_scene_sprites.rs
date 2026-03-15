use crate::camera::components::{CellSceneSprite, CellSceneUnitSprite};
use crate::camera::resources::CELL_SCENE_LAYER;
use crate::state::resources::{UnitsCache, UnitsDataCache};
use crate::ui::components::SlotIndicator;
use crate::ui::resources::CellState;
use crate::ui::systems::cell_view::*;
use bevy::prelude::*;

/// Spawn/update cell scene sprites when cell view changes.
pub fn sync_cell_scene_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    existing: Query<Entity, (With<CellSceneSprite>, Without<CellSceneUnitSprite>)>,
    windows: Query<&Window>,
) {
    if !cell_state.is_changed() {
        return;
    }

    // Despawn old sprites
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some(_viewed_cell) = cell_state.cell() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };
    let screen_w = window.width();
    let screen_h = window.height();

    // 1. Terrain background — sprite plein écran
    let biome = cell_state.biome();
    let terrain_path = get_terrain_background_path(biome);
    commands.spawn((
        Sprite {
            image: asset_server.load(&terrain_path),
            custom_size: Some(Vec2::new(screen_w, screen_h)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        CellSceneSprite,
        CELL_SCENE_LAYER,
    ));

    // 2. Building background
    if let Some(building_data) = cell_state.building_data {
        let building_path = get_building_background_path(&building_data);

        // Centré, proportions adaptées (carré si intérieur, 16:9 sinon)
        let (bg_w, bg_h) = if cell_state.has_interior() {
            let side = screen_h;
            (side, side)
        } else {
            (screen_w, screen_h)
        };

        commands.spawn((
            Sprite {
                image: asset_server.load(&building_path),
                custom_size: Some(Vec2::new(bg_w, bg_h)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            CellSceneSprite,
            CELL_SCENE_LAYER,
        ));

        // 3. Separators (only for buildings with interior)
        if cell_state.has_interior() {
            let (left_sep_path, right_sep_path) = get_separator_paths(Some(&building_data));
            let sep_width = screen_h * 0.08; // approximate separator width

            // Left separator — positioned at left edge of building
            commands.spawn((
                Sprite {
                    image: asset_server.load(&left_sep_path),
                    custom_size: Some(Vec2::new(sep_width, screen_h)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-bg_w / 2.0, 0.0, 2.0)),
                CellSceneSprite,
                CELL_SCENE_LAYER,
            ));

            // Right separator
            commands.spawn((
                Sprite {
                    image: asset_server.load(&right_sep_path),
                    custom_size: Some(Vec2::new(sep_width, screen_h)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(bg_w / 2.0, 0.0, 2.0)),
                CellSceneSprite,
                CELL_SCENE_LAYER,
            ));
        }
    }
}

/// Sync unit portrait sprites on CELL_SCENE_LAYER.
pub fn sync_cell_scene_unit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    existing: Query<Entity, With<CellSceneUnitSprite>>,
    windows: Query<&Window>,
    mut spawned_state: Local<Vec<(shared::SlotPosition, u64)>>,
) {
    let Some(viewed_cell) = cell_state.cell() else {
        if !spawned_state.is_empty() {
            for entity in &existing {
                commands.entity(entity).despawn();
            }
            spawned_state.clear();
        }
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let occupied_slots = units_cache.get_occupied_slots(&viewed_cell);
    let expected: Vec<(shared::SlotPosition, u64)> = occupied_slots
        .iter()
        .map(|(pos, uid)| (*pos, *uid))
        .collect();

    // Skip if already matching
    if *spawned_state == expected {
        return;
    }

    // Despawn old
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    spawned_state.clear();

    let screen_w = window.width();
    let screen_h = window.height();

    // Reproduce the same layout math as setup_cell_slots
    let slot_hex_layout = hexx::HexLayout::pointy().with_hex_size(70.0);
    let top_bar = 64.0;
    let margin = 10.0;
    let gap = 50.0;

    let has_interior = cell_state.has_interior();
    let slot_config = cell_state.slot_configuration();

    // Parent container: starts at (margin+0, top_bar+margin) → content area
    // parent left = margin (10), top = top_bar + margin (74)
    let parent_left = margin;
    let parent_top = top_bar + margin;

    if has_interior {
        let side = screen_h - top_bar - 4.0 * margin;
        let interior_width = screen_h - top_bar;
        // Each child has margin 10 on each side
        // Exterior width (flex_grow fills remaining)
        let exterior_container_w =
            (screen_w - 2.0 * margin - interior_width - 2.0 * gap) / 2.0 - 2.0 * margin;
        let exterior_container_h = side;
        let interior_container_w = side;
        let interior_container_h = side;

        // Container X positions (accounting for parent margin + child margins + gaps)
        let left_ext_x = parent_left + margin; // left exterior starts here
        let interior_x = left_ext_x + exterior_container_w + 2.0 * margin + gap;
        let right_ext_x = interior_x + interior_width + gap + margin; // approximate

        let exterior_container_size = Vec2::new(exterior_container_w, exterior_container_h);
        let interior_container_size = Vec2::new(interior_container_w, interior_container_h);

        // Generate positions (same as setup_cell_slots)
        let exterior_positions = slot_config
            .exterior_layout
            .generate_positions(exterior_container_size, &slot_hex_layout);
        let interior_positions = slot_config
            .interior_layout
            .generate_positions(interior_container_size, &slot_hex_layout);

        let offset = 56.0; // same as setup_cell_slots

        for (slot_pos, unit_id) in &occupied_slots {
            let screen_pos = match slot_pos.slot_type {
                shared::SlotType::Interior => {
                    if let Some(pos) = interior_positions.get(slot_pos.index) {
                        // Interior: left = pos.x - 56, top = pos.y - 65
                        Vec2::new(
                            interior_x + margin + pos.x - 56.0 + 56.0,
                            parent_top + margin + pos.y - 65.0 + 65.0,
                        )
                    } else {
                        continue;
                    }
                }
                shared::SlotType::Exterior => {
                    if let Some(pos) = exterior_positions.get(slot_pos.index) {
                        // Left exterior: left = pos.x - 56 - offset
                        Vec2::new(
                            left_ext_x + pos.x - 56.0 - offset + 56.0,
                            parent_top + margin + pos.y - 65.0 + 65.0,
                        )
                    } else if let Some(pos) =
                        exterior_positions.get(slot_pos.index - exterior_positions.len())
                    {
                        // Right exterior
                        Vec2::new(
                            right_ext_x + pos.x - 56.0 + 56.0,
                            parent_top + margin + pos.y - 65.0 + 65.0,
                        )
                    } else {
                        continue;
                    }
                }
            };

            // Convert screen coords (top-left, Y-down) to camera coords (center, Y-up)
            let world_x = screen_pos.x - screen_w / 2.0;
            let world_y = screen_h / 2.0 - screen_pos.y;

            let avatar_path = units_data_cache
                .get_unit(*unit_id)
                .and_then(|u| u.avatar_url.clone())
                .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

            commands.spawn((
                Sprite {
                    image: asset_server.load(&avatar_path),
                    custom_size: Some(Vec2::new(112.0, 130.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(world_x, world_y, 3.0)),
                CellSceneSprite,
                CellSceneUnitSprite,
                CELL_SCENE_LAYER,
            ));

            spawned_state.push((*slot_pos, *unit_id));
        }
    } else {
        // Exterior only — same logic
        let container_size = Vec2::new(
            screen_w - top_bar - 4.0 * margin,
            screen_h - top_bar - 4.0 * margin,
        );
        let exterior_positions = slot_config
            .exterior_layout
            .generate_positions(container_size, &slot_hex_layout);

        let container_x = parent_left + margin;

        for (slot_pos, unit_id) in &occupied_slots {
            if let Some(pos) = exterior_positions.get(slot_pos.index) {
                let screen_pos = Vec2::new(container_x + pos.x, parent_top + margin + pos.y);

                let world_x = screen_pos.x - screen_w / 2.0;
                let world_y = screen_h / 2.0 - screen_pos.y;

                let avatar_path = units_data_cache
                    .get_unit(*unit_id)
                    .and_then(|u| u.avatar_url.clone())
                    .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

                commands.spawn((
                    Sprite {
                        image: asset_server.load(&avatar_path),
                        custom_size: Some(Vec2::new(80.0, 93.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(world_x, world_y, 3.0)),
                    CellSceneSprite,
                    CellSceneUnitSprite,
                    CELL_SCENE_LAYER,
                ));

                spawned_state.push((*slot_pos, *unit_id));
            }
        }
    }
}

/// Cleanup on exit
pub fn cleanup_cell_scene_sprites(
    mut commands: Commands,
    sprites: Query<Entity, With<CellSceneSprite>>,
) {
    for entity in &sprites {
        commands.entity(entity).despawn();
    }
}
