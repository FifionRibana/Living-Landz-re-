use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

use shared::BuildingCategoryEnum;

use crate::ui::components::{
    ActionContentContainer, ActionRunButton, ActionTabButton, ActionTabsContainer,
    ActionsPanelMarker, BuildingGridContainer, RecipeContainer,
};

pub fn setup_action_panel(parent: &mut RelatedSpawnerCommands<ChildOf>, asset_server: &Res<AssetServer>) {
    let wood_panel_image = asset_server.load("ui/ui_wood_panel.png");
    let wood_panel_slicer = TextureSlicer {
        border: BorderRect {
            left: 42.,
            right: 42.,
            top: 42.,
            bottom: 42.,
        },
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    // Load tab button sprites
    let tab_normal: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_normal.png");
    // let tab_hovered: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_hovered.png");
    // let tab_selected: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_selected.png");

    parent
        .spawn((
            ImageNode {
                image: wood_panel_image.clone(),
                image_mode: NodeImageMode::Sliced(wood_panel_slicer.clone()),
                ..default()
            },
            Node {
                height: px(180.),
                position_type: PositionType::Absolute,
                bottom: px(0.0),
                left: px(80.),
                right: px(0.0),
                margin: UiRect::all(px(10.)),
                padding: UiRect::all(px(16.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ActionsPanelMarker,
            Visibility::Hidden,
            Interaction::None,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|actions_panel| {
            // Top action bar with run button
            actions_panel
                .spawn((
                    Node {
                        width: percent(100.),
                        height: px(40.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(px(8.)),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|top_bar| {
                    // Spacer on the left
                    top_bar.spawn((
                        Node {
                            flex_grow: 1.,
                            ..default()
                        },
                    ));

                    // Run button on the right
                    top_bar
                        .spawn((
                            Button,
                            Node {
                                width: px(120.),
                                height: px(40.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(px(8.)),
                                border: UiRect::all(px(2.)),
                                ..default()
                            },
                            BorderColor::all(Color::srgb_u8(100, 150, 100)),
                            BackgroundColor(Color::srgb_u8(80, 130, 80)),
                            ActionRunButton,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: true,
                            },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Exécuter"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(255, 255, 255)),
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });
                });

            // Tab bar (will be populated dynamically)
            actions_panel.spawn((
                Node {
                    width: percent(100.),
                    height: px(40.),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.),
                    margin: UiRect::bottom(px(8.)),
                    ..default()
                },
                ActionTabsContainer,
                Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
            ));

            // Content container (will be populated dynamically based on selected category/tab)
            actions_panel
                .spawn((
                    Node {
                        width: percent(100.),
                        flex_grow: 1.,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(8.),
                        ..default()
                    },
                    ActionContentContainer,
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|content_parent| {
                    // Left side: Building grid
                    content_parent.spawn((
                        Node {
                            width: percent(60.),
                            height: percent(100.),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: px(8.),
                            column_gap: px(8.),
                            align_content: AlignContent::FlexStart,
                            ..default()
                        },
                        BuildingGridContainer,
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));

                    // Right side: Recipe
                    content_parent.spawn((
                        Node {
                            width: percent(40.),
                            height: percent(100.),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(px(8.)),
                            ..default()
                        },
                        RecipeContainer,
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                });
        });
}

pub fn handle_action_tab_button_interactions(
    mut query: Query<(&ActionTabButton, &mut ImageNode, &Interaction), Changed<Interaction>>,
    mut action_state: ResMut<crate::ui::resources::ActionState>,
    asset_server: Res<AssetServer>,
) {
    let tab_normal: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_normal.png");
    let tab_hovered: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_hovered.png");
    let tab_selected: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_selected.png");

    for (tab_button, mut image_node, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                image_node.image = tab_selected.clone();
                action_state.select_tab(tab_button.tab_id.clone());

                // Parse tab_id to extract building category
                if tab_button.tab_id.starts_with("building_") {
                    let category_str = tab_button.tab_id.strip_prefix("building_").unwrap();
                    let category = parse_building_category(category_str);
                    if let Some(cat) = category {
                        action_state.select_building_category(cat);
                        info!("Building category selected: {:?}", cat);
                    }
                }

                info!("Action tab selected: {}", tab_button.tab_id);
            }
            Interaction::Hovered => {
                if action_state.selected_tab.as_ref() != Some(&tab_button.tab_id) {
                    image_node.image = tab_hovered.clone();
                }
            }
            Interaction::None => {
                if action_state.selected_tab.as_ref() != Some(&tab_button.tab_id) {
                    image_node.image = tab_normal.clone();
                } else {
                    image_node.image = tab_selected.clone();
                }
            }
        }
    }
}

fn parse_building_category(category_str: &str) -> Option<BuildingCategoryEnum> {
    match category_str {
        "Urbanism" => Some(BuildingCategoryEnum::Urbanism),
        "Dwellings" => Some(BuildingCategoryEnum::Dwellings),
        "ManufacturingWorkshops" => Some(BuildingCategoryEnum::ManufacturingWorkshops),
        "Agriculture" => Some(BuildingCategoryEnum::Agriculture),
        "Entertainment" => Some(BuildingCategoryEnum::Entertainment),
        "Military" => Some(BuildingCategoryEnum::Military),
        _ => None,
    }
}

pub fn update_action_panel_visibility(
    action_state: Res<crate::ui::resources::ActionState>,
    mut panel_query: Query<&mut Visibility, With<ActionsPanelMarker>>,
) {
    if action_state.is_changed() {
        for mut visibility in panel_query.iter_mut() {
            *visibility = if action_state.selected_category.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
