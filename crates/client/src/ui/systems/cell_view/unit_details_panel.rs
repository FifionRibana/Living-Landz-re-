use crate::state::resources::UnitsDataCache;
use crate::states::GameView;
use crate::ui::resources::UnitSelectionState;
use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use shared::ProfessionEnum;

// ─── Marker components (local to this module) ───────────────

/// Root container for the entire tab + panel.
#[derive(Component)]
pub struct UnitDetailsRoot;

/// The small clickable tab showing selection count.
#[derive(Component)]
pub struct UnitDetailsTab;

/// The badge text inside the tab (e.g. "3").
#[derive(Component)]
pub struct UnitDetailsTabBadge;

/// The expandable panel below the tab.
#[derive(Component)]
pub struct UnitDetailsExpandedPanel;

/// Scrollable list container inside the expanded panel.
#[derive(Component)]
pub struct UnitDetailsListContainer;

/// One entry in the unit list.
#[derive(Component)]
pub struct UnitDetailsListItem {
    pub unit_id: u64,
}

/// Tracks whether the panel is expanded.
#[derive(Component)]
pub struct UnitDetailsPanelState {
    pub expanded: bool,
}

// ─── Setup ──────────────────────────────────────────────────

/// Spawn the unit details tab + panel. Called on OnEnter(GameView::Cell).
/// Auto-despawned via DespawnOnExit(GameView::Cell).
pub fn setup_unit_details_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(80.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(0.0),
                ..default()
            },
            // Start hidden — shown when there's a selection
            Visibility::Hidden,
            UnitDetailsRoot,
            UnitDetailsPanelState { expanded: false },
            DespawnOnExit(GameView::Cell),
        ))
        .with_children(|root| {
            // ── Tab button ──
            root.spawn((
                Button,
                Node {
                    min_width: Val::Px(46.0),
                    height: Val::Px(36.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ImageNode {
                    image: paper_panel_image.clone(),
                    image_mode: NodeImageMode::Sliced(paper_panel_slicer.clone()),
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                UnitDetailsTab,
            ))
            .with_children(|tab| {
                tab.spawn((
                    Text::new("Units"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(67, 60, 37)),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ));
                // Badge background + count
                tab.spawn((
                    Node {
                        width: Val::Px(22.0),
                        height: Val::Px(22.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(11.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.5, 0.15)),
                    GlobalZIndex(5),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ))
                .with_children(|badge_bg| {
                    badge_bg.spawn((
                        Text::new("0"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        UnitDetailsTabBadge,
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                });
            });

            // ── Expanded panel ──
            root.spawn((
                ImageNode {
                    image: paper_panel_image,
                    image_mode: NodeImageMode::Sliced(paper_panel_slicer),
                    ..default()
                },
                Node {
                    width: Val::Px(260.0),
                    max_height: Val::Px(400.0),
                    padding: UiRect {
                        left: Val::Px(14.0),
                        right: Val::Px(14.0),
                        top: Val::Px(18.0),
                        bottom: Val::Px(14.0),
                    },
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                Visibility::Hidden,
                UnitDetailsExpandedPanel,
                UnitDetailsListContainer,
            ));
        });
}

// ─── Systems ────────────────────────────────────────────────

/// Show/hide the root based on whether there's a selection.
pub fn update_panel_visibility(
    unit_selection: Res<UnitSelectionState>,
    mut root_query: Query<&mut Visibility, With<UnitDetailsRoot>>,
) {
    for mut vis in &mut root_query {
        *vis = if unit_selection.has_selection() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Update the badge count text.
pub fn update_tab_badge(
    unit_selection: Res<UnitSelectionState>,
    mut badge_query: Query<&mut Text, With<UnitDetailsTabBadge>>,
) {
    let count = unit_selection.count();
    for mut text in &mut badge_query {
        **text = format!("{}", count);
    }
}

/// Handle tab click → toggle expanded state.
pub fn handle_tab_click(
    tab_query: Query<&Interaction, (Changed<Interaction>, With<UnitDetailsTab>)>,
    mut state_query: Query<&mut UnitDetailsPanelState>,
    mut panel_query: Query<&mut Visibility, With<UnitDetailsExpandedPanel>>,
) {
    for interaction in &tab_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        for mut state in &mut state_query {
            state.expanded = !state.expanded;
        }
        for mut vis in &mut panel_query {
            // Will be set properly in the next block, but toggle immediately
            let expanded = state_query
                .iter()
                .next()
                .map(|s| s.expanded)
                .unwrap_or(false);
            *vis = if expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Rebuild the unit list when selection or data changes.
pub fn update_panel_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    unit_selection: Res<UnitSelectionState>,
    units_data_cache: Res<UnitsDataCache>,
    list_container: Query<Entity, With<UnitDetailsListContainer>>,
    existing_items: Query<Entity, With<UnitDetailsListItem>>,
) {
    if !unit_selection.is_changed() && !units_data_cache.is_changed() {
        return;
    }

    // Despawn old items
    for entity in &existing_items {
        commands.entity(entity).despawn();
    }

    let Ok(container) = list_container.single() else {
        return;
    };

    // Spawn an entry for each selected unit
    for &unit_id in unit_selection.selected_ids() {
        let Some(unit_data) = units_data_cache.get_unit(unit_id) else {
            continue;
        };

        let avatar_path = unit_data
            .avatar_url
            .as_deref()
            .unwrap_or("ui/icons/unit_placeholder.png");

        let item = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(6.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.06)),
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
                UnitDetailsListItem { unit_id },
            ))
            .with_children(|parent| {
                // Portrait
                parent.spawn((
                    ImageNode {
                        image: asset_server.load(avatar_path.to_string()),
                        ..default()
                    },
                    Node {
                        width: Val::Px(48.0),
                        height: Val::Px(48.0),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ));

                // Info column
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            row_gap: Val::Px(2.0),
                            ..default()
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ))
                    .with_children(|col| {
                        // Name
                        col.spawn((
                            Text::new(unit_data.full_name()),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(67, 60, 37)),
                        ));

                        // Profession + Level row
                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|row| {
                            // Profession color dot
                            row.spawn((
                                Node {
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    border_radius: BorderRadius::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(profession_color(&unit_data.profession)),
                            ));
                            row.spawn((
                                Text::new(unit_data.profession.to_name()),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(100, 90, 70)),
                            ));
                            row.spawn((
                                Text::new(format!("Lvl {}", unit_data.level)),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(120, 110, 90)),
                            ));
                        });

                        // Money
                        col.spawn((
                            Text::new(format!("{} coins", unit_data.money)),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(140, 130, 80)),
                        ));
                    });
            })
            .id();

        commands.entity(container).add_child(item);
    }
}

/// Collapse panel when selection is cleared.
pub fn collapse_on_deselect(
    unit_selection: Res<UnitSelectionState>,
    mut state_query: Query<&mut UnitDetailsPanelState>,
    mut panel_query: Query<&mut Visibility, With<UnitDetailsExpandedPanel>>,
) {
    if !unit_selection.is_changed() {
        return;
    }
    if !unit_selection.has_selection() {
        for mut state in &mut state_query {
            state.expanded = false;
        }
        for mut vis in &mut panel_query {
            *vis = Visibility::Hidden;
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────

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
