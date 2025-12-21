use bevy::prelude::*;
use shared::grid::GridCell;
use crate::ui::components::{
    ActionBarMarker, ActionsPanelMarker, CellDetailsPanelMarker, CellViewBackgroundImage,
    CellViewContainer, ChatPanelMarker, TopBarMarker, SlotGridContainer, SlotIndicator, CellViewBackButton,
};
use crate::ui::resources::CellViewState;
use crate::ui::systems::cell_view::load_background_image;
use crate::state::resources::WorldCache;
use shared::{BiomeTypeEnum, SlotConfiguration, SlotPosition, SlotType};

/// Update cell view and world UI visibility based on CellViewState
pub fn update_cell_view_visibility(
    cell_view_state: Res<CellViewState>,
    mut cell_view_query: Query<&mut Visibility, With<CellViewContainer>>,
    mut world_ui_query: Query<
        &mut Visibility,
        (
            Without<CellViewContainer>,
            Or<(
                With<ActionBarMarker>,
                With<ActionsPanelMarker>,
                With<CellDetailsPanelMarker>,
                With<TopBarMarker>,
                With<ChatPanelMarker>,
            )>,
        ),
    >,
) {
    if !cell_view_state.is_changed() {
        return;
    }

    // Show/hide cell view container
    for mut visibility in &mut cell_view_query {
        *visibility = if cell_view_state.is_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Hide world UI when in cell view mode (keep top bar and chat visible)
    for mut visibility in &mut world_ui_query {
        *visibility = if cell_view_state.is_active {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// Update cell view content when the viewed cell changes
pub fn update_cell_view_content(
    cell_view_state: Res<CellViewState>,
    world_cache: Res<WorldCache>,
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewContainer>>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
) {
    if !cell_view_state.is_changed() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    // Get cell data
    let cell_data = world_cache.get_cell(&viewed_cell);
    let building = world_cache.get_building(&viewed_cell);

    let biome = cell_data
        .map(|c| c.biome)
        .unwrap_or(BiomeTypeEnum::Undefined);

    // Determine slot configuration
    // For now, use terrain type since we need BuildingTypeEnum
    // TODO: Map BuildingData to BuildingTypeEnum properly
    let slot_config = SlotConfiguration::for_terrain_type(biome);

    // Clear existing content in container
    for container_entity in &container_query {
        if let Ok(children) = children_query.get(container_entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // Rebuild content
        commands.entity(container_entity).with_children(|parent| {
            // Background image
            let bg_image = load_background_image(&asset_server, building, biome);
            parent.spawn((
                ImageNode {
                    image: bg_image,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                CellViewBackgroundImage,
            ));

            // Main content container (on top of background)
            parent
                .spawn((Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(85.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    position_type: PositionType::Relative,
                    ..default()
                },))
                .with_children(|content| {
                    // Title
                    content.spawn((
                        Text::new(format!("Cell: q={}, r={}", viewed_cell.q, viewed_cell.r)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Slot grids container
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            column_gap: Val::Px(40.0),
                            ..default()
                        },))
                        .with_children(|grids| {
                            // Interior slots (if any)
                            if slot_config.has_interior() {
                                let (rows, cols) = slot_config.interior_grid_size;
                                let hex_image = asset_server.load("ui/slot_hex_interior.png");

                                grids.spawn((Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(10.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },)).with_children(|section| {
                                    section.spawn((
                                        Text::new("Interior Slots"),
                                        TextFont { font_size: 20.0, ..default() },
                                        TextColor(Color::WHITE),
                                    ));

                                    section.spawn((
                                        Node {
                                            display: Display::Grid,
                                            grid_template_columns: vec![RepeatedGridTrack::flex(cols as u16, 1.0)],
                                            grid_template_rows: vec![RepeatedGridTrack::flex(rows as u16, 1.0)],
                                            column_gap: Val::Px(8.0),
                                            row_gap: Val::Px(8.0),
                                            padding: UiRect::all(Val::Px(16.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                                        BorderRadius::all(Val::Px(8.0)),
                                        SlotGridContainer { slot_type: SlotType::Interior },
                                    )).with_children(|grid| {
                                        for index in 0..slot_config.interior_slots {
                                            grid.spawn((
                                                Button,
                                                Node {
                                                    width: Val::Px(64.0),
                                                    height: Val::Px(74.0),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },
                                                ImageNode { image: hex_image.clone(), ..default() },
                                                SlotIndicator {
                                                    position: SlotPosition::interior(index),
                                                    occupied_by: None,
                                                },
                                                Interaction::None,
                                            ));
                                        }
                                    });
                                });
                            }

                            // Exterior slots
                            let (rows, cols) = slot_config.exterior_grid_size;
                            let hex_image = asset_server.load("ui/slot_hex_exterior.png");

                            grids.spawn((Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },)).with_children(|section| {
                                section.spawn((
                                    Text::new("Exterior Slots"),
                                    TextFont { font_size: 20.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));

                                section.spawn((
                                    Node {
                                        display: Display::Grid,
                                        grid_template_columns: vec![RepeatedGridTrack::flex(cols as u16, 1.0)],
                                        grid_template_rows: vec![RepeatedGridTrack::flex(rows as u16, 1.0)],
                                        column_gap: Val::Px(8.0),
                                        row_gap: Val::Px(8.0),
                                        padding: UiRect::all(Val::Px(16.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                                    BorderRadius::all(Val::Px(8.0)),
                                    SlotGridContainer { slot_type: SlotType::Exterior },
                                )).with_children(|grid| {
                                    for index in 0..slot_config.exterior_slots {
                                        grid.spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(64.0),
                                                height: Val::Px(74.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            ImageNode { image: hex_image.clone(), ..default() },
                                            SlotIndicator {
                                                position: SlotPosition::exterior(index),
                                                occupied_by: None,
                                            },
                                            Interaction::None,
                                        ));
                                    }
                                });
                            });
                        });
                });

            // Back button (positioned absolutely)
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.7, 0.2, 0.2, 0.9)),
                BorderRadius::all(Val::Px(8.0)),
                CellViewBackButton,
            )).with_children(|button| {
                button.spawn((
                    Text::new("← Back"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

        info!(
            "Cell view content updated for cell: q={}, r={} (interior: {}, exterior: {})",
            viewed_cell.q, viewed_cell.r, slot_config.interior_slots, slot_config.exterior_slots
        );
    }
}
