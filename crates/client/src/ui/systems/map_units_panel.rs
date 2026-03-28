use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use hexx::Hex;

use crate::camera::MainCamera;
use crate::grid::resources::SelectedHexes;
use crate::state::resources::{UnitsCache, UnitsDataCache};
use crate::states::AppState;
use crate::ui::components::PendingLayerComposition;
use crate::ui::resources::{
    MapUnitsPanelMode, MapUnitsPanelState, UnitSelectionState, VisibleUnitsInRange,
};
use shared::ProfessionEnum;
use shared::grid::{GridCell, GridConfig};

// ─── Marker components ───────────────────────────────────────

#[derive(Component)]
pub struct MapUnitsPanelContainer;

#[derive(Component)]
pub struct MapUnitsPanelToggleButton;

#[derive(Component)]
pub struct MapUnitsPanelToggleText;

#[derive(Component)]
pub struct MapUnitsListContainer;

#[derive(Component)]
pub struct MapUnitListItem {
    pub unit_id: u64,
}

#[derive(Component)]
pub struct MapUnitsPanelCountText;

// ─── Setup ───────────────────────────────────────────────────

/// Spawn the map units sidebar (initially hidden).
/// Called on OnEnter(AppState::InGame).
pub fn setup_map_units_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paper_panel_image = asset_server.load("ui/ui_paper_panel_md.png");
    let paper_panel_slicer = TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(42., 76.),
            max_inset: Vec2::new(42., 42.)
        },
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    commands
        .spawn((
            ImageNode {
                image: paper_panel_image,
                image_mode: NodeImageMode::Sliced(paper_panel_slicer),
                ..default()
            },
            Node {
                width: Val::Px(240.0),
                max_height: Val::Percent(55.0),
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(80.0),
                padding: UiRect {
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(16.0),
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            MapUnitsPanelContainer,
            DespawnOnExit(AppState::InGame),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // ── Header row: title + count ──
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: true,
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Units"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(67, 60, 37)),
                    ));
                    header.spawn((
                        Text::new("0"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(120, 110, 90)),
                        MapUnitsPanelCountText,
                    ));
                });

            // ── Toggle mode button ──
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(26.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.15)),
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: true,
                    },
                    MapUnitsPanelToggleButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Mode: Selection"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(90, 80, 60)),
                        MapUnitsPanelToggleText,
                    ));
                });

            // ── Scrollable list ──
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(3.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                MapUnitsListContainer,
            ));
        });
}

// ─── Collect visible units ──────────────────────────────────

/// Recalculate which units are in range based on panel mode and selection.
pub fn collect_visible_units(
    selected_hexes: Res<SelectedHexes>,
    panel_state: Res<MapUnitsPanelState>,
    units_cache: Res<UnitsCache>,
    grid_config: Res<GridConfig>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window>,
    mut visible_units: ResMut<VisibleUnitsInRange>,
) {
    // Only recalculate when inputs change
    if !selected_hexes.is_changed() && !units_cache.is_changed() && !panel_state.is_changed() {
        return;
    }

    visible_units.units.clear();

    let center_hex: Option<Hex> = match panel_state.mode {
        MapUnitsPanelMode::OnSelection => selected_hexes.ids.iter().next().copied(),
        MapUnitsPanelMode::AlwaysVisible => {
            let Ok((camera, transform)) = cameras.single() else {
                return;
            };
            let Ok(window) = windows.single() else {
                return;
            };
            let screen_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
            camera
                .viewport_to_world_2d(transform, screen_center)
                .ok()
                .map(|pos| grid_config.layout.world_pos_to_hex(pos))
        }
    };

    let Some(center) = center_hex else {
        return;
    };

    let radius = panel_state.scan_radius as i32;

    // Hex ring iteration using cube coordinates constraint: q + r + s = 0
    for dq in -radius..=radius {
        let r_min = (-radius).max(-dq - radius);
        let r_max = radius.min(-dq + radius);
        for dr in r_min..=r_max {
            let hex = Hex::new(center.x + dq, center.y + dr);
            let cell = GridCell::from_hex(&hex);
            if let Some(unit_ids) = units_cache.get_units_at_cell(&cell) {
                for &uid in unit_ids {
                    visible_units.units.push((uid, cell));
                }
            }
        }
    }
}

// ─── Update list UI ──────────────────────────────────────────

/// Rebuild the list of unit items when data changes.
pub fn update_map_units_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    visible_units: Res<VisibleUnitsInRange>,
    units_data_cache: Res<UnitsDataCache>,
    unit_selection: Res<UnitSelectionState>,
    list_container: Query<Entity, With<MapUnitsListContainer>>,
    existing_items: Query<Entity, With<MapUnitListItem>>,
    mut count_text: Query<&mut Text, With<MapUnitsPanelCountText>>,
) {
    if !visible_units.is_changed() && !unit_selection.is_changed() {
        return;
    }

    // Update count text
    for mut text in &mut count_text {
        **text = format!("{}", visible_units.units.len());
    }

    // Despawn old items
    for entity in &existing_items {
        commands.entity(entity).despawn();
    }

    let Ok(container) = list_container.single() else {
        return;
    };

    // Spawn new items
    for &(unit_id, _cell) in &visible_units.units {
        let Some(unit_data) = units_data_cache.get_unit(unit_id) else {
            continue;
        };
        let is_selected = unit_selection.is_selected(unit_id);

        let bg = if is_selected {
            Color::srgba(0.25, 0.55, 0.25, 0.35)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.08)
        };

        let item = commands
            .spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(36.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(bg),
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                MapUnitListItem { unit_id },
            ))
            .with_children(|parent| {
                // Unit portrait (small thumbnail)
                // Unit portrait (small thumbnail)
                if let Some([bust, face, clothes, hair]) = unit_data.parse_portrait_layers() {
                    // Lord: compose 4 layers
                    let layer_handles = [
                        asset_server.load(format!(
                            "sprites/character/layers/bust/bust_{:02}.png",
                            bust + 1
                        )),
                        asset_server.load(format!(
                            "sprites/character/layers/face/face_{:02}.png",
                            face + 1
                        )),
                        asset_server.load(format!(
                            "sprites/character/layers/clothes/clothes_{:02}.png",
                            clothes + 1
                        )),
                        asset_server.load(format!(
                            "sprites/character/layers/hair/hair_{:02}.png",
                            hair + 1
                        )),
                    ];
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load("ui/icons/unit_placeholder.png"),
                            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                            ..default()
                        },
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            ..default()
                        },
                        PendingLayerComposition {
                            layer_handles,
                            mask_handle: None, // pas de hex mask pour les thumbnails
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                } else {
                    // NPC: single avatar
                    let avatar_path = unit_data
                        .avatar_url
                        .as_deref()
                        .unwrap_or("ui/icons/unit_placeholder.png");
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load(avatar_path.to_string()),
                            ..default()
                        },
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            ..default()
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                }
                // Profession color dot
                parent.spawn((
                    Node {
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(profession_color(&unit_data.profession)),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ));
                // Name + profession column
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ))
                    .with_children(|col| {
                        col.spawn((
                            Text::new(unit_data.full_name()),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(67, 60, 37)),
                        ));
                        col.spawn((
                            Text::new(unit_data.profession.to_name()),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(120, 110, 90)),
                        ));
                    });
            })
            .id();

        commands.entity(container).add_child(item);
    }
}

// ─── Interaction handlers ────────────────────────────────────

/// Handle click on a unit list item → select unit.
pub fn handle_map_unit_list_click(
    mut unit_selection: ResMut<UnitSelectionState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    item_query: Query<(&Interaction, &MapUnitListItem), Changed<Interaction>>,
) {
    for (interaction, item) in &item_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let ctrl =
            keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        if ctrl {
            unit_selection.toggle(item.unit_id);
        } else if shift {
            unit_selection.add(item.unit_id);
        } else {
            unit_selection.select(item.unit_id);
        }
    }
}

/// Handle toggle button → switch panel mode.
pub fn handle_map_units_panel_toggle(
    mut panel_state: ResMut<MapUnitsPanelState>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<MapUnitsPanelToggleButton>)>,
    mut text_query: Query<&mut Text, With<MapUnitsPanelToggleText>>,
) {
    for interaction in &button_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        panel_state.mode = match panel_state.mode {
            MapUnitsPanelMode::OnSelection => MapUnitsPanelMode::AlwaysVisible,
            MapUnitsPanelMode::AlwaysVisible => MapUnitsPanelMode::OnSelection,
        };
    }

    // Update label
    let label = match panel_state.mode {
        MapUnitsPanelMode::OnSelection => "Mode: Selection",
        MapUnitsPanelMode::AlwaysVisible => "Mode: Radar",
    };
    for mut text in &mut text_query {
        **text = label.to_string();
    }
}

/// Show/hide the panel based on mode and data.
pub fn update_map_units_panel_visibility(
    panel_state: Res<MapUnitsPanelState>,
    selected_hexes: Res<SelectedHexes>,
    visible_units: Res<VisibleUnitsInRange>,
    mut panel_query: Query<&mut Visibility, With<MapUnitsPanelContainer>>,
) {
    let should_show = match panel_state.mode {
        MapUnitsPanelMode::AlwaysVisible => !visible_units.units.is_empty(),
        MapUnitsPanelMode::OnSelection => {
            !selected_hexes.ids.is_empty() && !visible_units.units.is_empty()
        }
    };

    for mut vis in &mut panel_query {
        *vis = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// ─── Helpers ─────────────────────────────────────────────────

fn profession_color(profession: &ProfessionEnum) -> Color {
    use ProfessionEnum::*;
    match profession {
        Farmer => Color::srgb(0.4, 0.7, 0.3),
        Warrior => Color::srgb(0.8, 0.2, 0.2),
        Blacksmith => Color::srgb(0.5, 0.5, 0.6),
        Carpenter => Color::srgb(0.7, 0.5, 0.3),
        Miner => Color::srgb(0.4, 0.4, 0.4),
        Merchant => Color::srgb(0.8, 0.7, 0.2),
        Hunter => Color::srgb(0.3, 0.5, 0.3),
        Healer => Color::srgb(0.9, 0.9, 0.5),
        Scholar => Color::srgb(0.3, 0.3, 0.7),
        Baker | Cook => Color::srgb(0.8, 0.6, 0.4),
        Fisherman => Color::srgb(0.3, 0.5, 0.8),
        Lumberjack => Color::srgb(0.6, 0.4, 0.2),
        Mason => Color::srgb(0.6, 0.6, 0.5),
        Brewer => Color::srgb(0.6, 0.4, 0.1),
        Settler => Color::srgb(0.5, 0.5, 0.5),
        Unknown => Color::srgb(0.5, 0.5, 0.5),
    }
}
