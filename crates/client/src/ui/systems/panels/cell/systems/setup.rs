use bevy::prelude::*;
use shared::grid::GridCell;

use crate::ui::{
    components::{CellViewBackgroundImage, PanelContainer},
    resources::{CellState, PanelEnum, UIState},
    systems::{
        load_building_background, load_separators, load_terrain_background,
        panels::components::CellViewPanel,
    },
};

pub fn setup_cell_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)), // Dark semi-transparent background
            Visibility::Hidden,                                 // Hidden by default),
            PanelContainer {
                panel: PanelEnum::CellView,
            },
            CellViewPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CELL VIEW"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn setup_cell_layout(
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewPanel>>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    ui_state: Res<UIState>,
    mut last_viewed_cell: Local<Option<GridCell>>,
) {
    // Si on n'est pas dans CellView, reset l'état
    if ui_state.panel_state != PanelEnum::CellView {
        if last_viewed_cell.is_some() {
            *last_viewed_cell = None;
        }
        return;
    }

    // Only rebuild content when the viewed cell actually changes, not on every state change
    let Some(viewed_cell) = cell_state.cell() else {
        // Cell view closed - reset state
        if last_viewed_cell.is_some() {
            *last_viewed_cell = None;
        }
        return;
    };

    // Check if this is the same cell we're already displaying
    if *last_viewed_cell == Some(viewed_cell) {
        return;
    }

    // Cell changed - update the display
    *last_viewed_cell = Some(viewed_cell);

    // Get cell data
    let biome = cell_state.biome();

    // Clear existing content in container
    for container_entity in &container_query {
        if let Ok(children) = children_query.get(container_entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // Rebuild content
        commands.entity(container_entity).with_children(|parent| {
            info!("Setting up cell layout: {:?}", viewed_cell);
            // 1. Terrain background (full screen, behind everything)
            let terrain_bg = load_terrain_background(&asset_server, biome);
            parent.spawn((
                ImageNode {
                    image: terrain_bg,
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

            // 2. Building background and separators container (if building exists)
            if let Some(building_data) = cell_state.building_data {
                let building_bg = load_building_background(&asset_server, &building_data);

                // Check if building has interior slots
                if cell_state.has_interior() {
                    // Buildings WITH interior: square 1:1 ratio with separators
                    let (left_separator, right_separator) =
                        load_separators(&asset_server, Some(&building_data));

                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },))
                        .with_children(|building_container| {
                            // Left separator
                            building_container.spawn((
                                ImageNode {
                                    image: left_separator,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    width: Val::Auto,
                                    ..default()
                                },
                            ));

                            // Building background (square 1:1 ratio, height constrained)
                            building_container.spawn((
                                ImageNode {
                                    image: building_bg,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    aspect_ratio: Some(1.0), // Force 1:1 ratio
                                    ..default()
                                },
                            ));

                            // Right separator
                            building_container.spawn((
                                ImageNode {
                                    image: right_separator,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    width: Val::Auto,
                                    ..default()
                                },
                            ));
                        });
                } else {
                    // Buildings WITHOUT interior (only exterior): 16:9 ratio, no separators
                    parent.spawn((
                        ImageNode {
                            image: building_bg,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                }
            }
        });
    }
}
