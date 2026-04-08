use bevy::prelude::*;
use shared::{
    BuildingCategoryEnum, BuildingTypeEnum, RoadCategory, TerrainChunkId, constants,
    grid::{GridCell, GridConfig},
};

use crate::{
    grid::resources::SelectedHexes,
    networking::client::game_client::{SendActionBuildBuilding, SendActionBuildRoad},
    state::resources::{ConnectionStatus, GameDataCache},
    ui::{
        components::{
            ActionCategory, ActionRunButton, ActionTabButton, ActionTabsContainer,
            ActionsPanelMarker, BuildingButton, BuildingGridContainer, RecipeContainer,
        },
        resources::ActionState,
    },
};

/// Map a building string ID (from to_name_lowercase) to the database building_type_id.
fn building_id_to_type_id(building_id: &str) -> Option<i32> {
    BuildingTypeEnum::iter()
        .find(|bt| bt.to_name_lowercase() == building_id)
        .map(|bt| bt.to_id() as i32)
}

pub fn update_action_panel_content(
    mut commands: Commands,
    action_state: Res<ActionState>,
    asset_server: Res<AssetServer>,
    tabs_query: Query<Entity, With<ActionTabsContainer>>,
    building_grid_query: Query<Entity, With<BuildingGridContainer>>,
    recipe_query: Query<Entity, With<RecipeContainer>>,
    tabs_children_query: Query<&Children>,
    panel_query: Query<Entity, With<ActionsPanelMarker>>,
    game_data_cache: Res<GameDataCache>,
) {
    if !action_state.is_changed() {
        return;
    }

    // Clear existing tabs
    for entity in tabs_query.iter() {
        if let Ok(children) = tabs_children_query.get(entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
    }

    // Clear existing building grid
    for entity in building_grid_query.iter() {
        if let Ok(children) = tabs_children_query.get(entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
    }

    // Clear existing recipe
    for entity in recipe_query.iter() {
        if let Ok(children) = tabs_children_query.get(entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
    }

    if let Some(category) = action_state.selected_category {
        match category {
            ActionCategory::Roads => {
                // Roads have no tabs, just content
                populate_roads_content(
                    &mut commands,
                    &building_grid_query,
                    &asset_server,
                    &action_state,
                );
            }
            ActionCategory::Buildings => {
                // Buildings have tabs for each building category
                populate_building_tabs(&mut commands, &tabs_query, &asset_server, &action_state);
                if action_state.selected_building_category.is_some() {
                    populate_building_content(
                        &mut commands,
                        &building_grid_query,
                        &recipe_query,
                        &asset_server,
                        &action_state,
                        &game_data_cache,
                    );
                }
            }
            ActionCategory::Production
            | ActionCategory::Management
            | ActionCategory::Entertainment => {
                // These categories have no tabs for now
                populate_empty_content(&mut commands, &building_grid_query, category);
            }
        }

        // Add or update Run button if needed
        update_run_button(&mut commands, &panel_query, &action_state, &asset_server);
    }
}

fn populate_roads_content(
    commands: &mut Commands,
    grid_query: &Query<Entity, With<BuildingGridContainer>>,
    asset_server: &Res<AssetServer>,
    action_state: &ActionState,
) {
    // For now, just a placeholder
    for entity in grid_query.iter() {
        commands.entity(entity).with_children(|parent| {
            let categories = [
                (RoadCategory::DirtPath, "Chemin"),
                (RoadCategory::PavedRoad, "Route"),
                (RoadCategory::Highway, "Grande voie"),
            ];

            let tab_normal: Handle<Image> =
                asset_server.load("ui/ui_wood_tab_bar_button_normal.png");

            for (category, label) in categories.iter() {
                info!("Adding category tab: {:?}", category);
                let is_selected = action_state.selected_road_category == Some(*category);

                parent
                    .spawn((
                        Button,
                        Node {
                            width: px(120.),
                            height: px(40.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(px(8.)),
                            ..default()
                        },
                        ImageNode {
                            image: tab_normal.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        BackgroundColor(if is_selected {
                            Color::srgb_u8(180, 160, 130)
                        } else {
                            Color::srgb_u8(140, 120, 90)
                        }),
                        ActionTabButton {
                            tab_id: format!("building_{:?}", category),
                        },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: true,
                        },
                    ))
                    .with_children(|tab_button| {
                        tab_button.spawn((
                            Text::new(*label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(223, 210, 194)),
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ));
                    });
            }
        });
    }
}

fn populate_building_tabs(
    commands: &mut Commands,
    tabs_query: &Query<Entity, With<ActionTabsContainer>>,
    asset_server: &Res<AssetServer>,
    action_state: &ActionState,
) {
    info!("Populate building tabs");
    let tab_normal: Handle<Image> = asset_server.load("ui/ui_wood_tab_bar_button_normal.png");

    let categories = [
        (BuildingCategoryEnum::Infrastructure, "Urbanisme"),
        (BuildingCategoryEnum::Residential, "Habitations"),
        (BuildingCategoryEnum::SemiSpecialized, "Ateliers"),
        (BuildingCategoryEnum::Metal, "Métallurgie"),
        (BuildingCategoryEnum::Earth, "Terre"),
        (BuildingCategoryEnum::Food, "Alimentation"),
        (BuildingCategoryEnum::Wood, "Bois"),
        (BuildingCategoryEnum::Textile, "Textile"),
        (BuildingCategoryEnum::FineArtisan, "Artisanat Fin"),
        (BuildingCategoryEnum::AnimalBreeding, "Élevage"),
        (BuildingCategoryEnum::ServiceCombined, "Services Combinés"),
        (BuildingCategoryEnum::ServiceDedicated, "Services Dédiés"),
        (BuildingCategoryEnum::Defense, "Militaire"),
    ];

    for entity in tabs_query.iter() {
        commands.entity(entity).with_children(|parent| {
            for (category, label) in categories.iter() {
                info!("Adding category tab: {:?}", category);
                let is_selected = action_state.selected_building_category == Some(*category);

                parent
                    .spawn((
                        Button,
                        Node {
                            width: px(120.),
                            height: px(40.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(px(8.)),
                            ..default()
                        },
                        ImageNode {
                            image: tab_normal.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        BackgroundColor(if is_selected {
                            Color::srgb_u8(180, 160, 130)
                        } else {
                            Color::srgb_u8(140, 120, 90)
                        }),
                        ActionTabButton {
                            tab_id: format!("building_{:?}", category),
                        },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: true,
                        },
                    ))
                    .with_children(|tab_button| {
                        tab_button.spawn((
                            Text::new(*label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(223, 210, 194)),
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ));
                    });
            }
        });
    }
}

fn populate_building_content(
    commands: &mut Commands,
    grid_query: &Query<Entity, With<BuildingGridContainer>>,
    recipe_query: &Query<Entity, With<RecipeContainer>>,
    asset_server: &Res<AssetServer>,
    action_state: &ActionState,
    game_data_cache: &GameDataCache,
) {
    if let Some(building_category) = action_state.selected_building_category {
        // Get buildings for this category
        let buildings = get_buildings_for_category(building_category);
        info!(
            "Building count {} for category {:?}",
            buildings.len(),
            building_category
        );

        // Populate building grid
        for entity in grid_query.iter() {
            commands.entity(entity).with_children(|parent| {
                for (building_id, building_name, icon_path) in buildings.iter() {
                    let is_selected =
                        action_state.selected_building_id.as_deref() == Some(*building_id);

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: px(100.),
                                height: px(120.),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(px(4.)),
                                border: UiRect::all(px(2.)),
                                ..default()
                            },
                            BorderColor::all(if is_selected {
                                Color::srgb_u8(200, 150, 100)
                            } else {
                                Color::srgb_u8(100, 90, 70)
                            }),
                            BackgroundColor(Color::srgb_u8(80, 70, 50)),
                            BuildingButton {
                                building_id: building_id.to_string(),
                                building_name: building_name.to_string(),
                            },
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: true,
                            },
                        ))
                        .with_children(|button| {
                            // Building icon
                            button.spawn((
                                Node {
                                    width: px(64.),
                                    height: px(64.),
                                    margin: UiRect::bottom(px(4.)),
                                    ..default()
                                },
                                ImageNode {
                                    image: asset_server.load(*icon_path),
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));

                            // Building name
                            button.spawn((
                                Text::new(*building_name),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Node {
                                    max_width: px(92.),
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });
                }
            });
        }

        // Populate recipe panel
        if let Some(building_id) = &action_state.selected_building_id {
            for entity in recipe_query.iter() {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text::new("Recette de construction"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        Node {
                            margin: UiRect::bottom(px(8.)),
                            ..default()
                        },
                    ));

                    // Get construction costs from GameDataCache
                    if let Some(type_id) = building_id_to_type_id(building_id) {
                        let costs = game_data_cache.building_costs(type_id);
                        for cost in &costs {
                            let item_name = game_data_cache.item_name(cost.item_id, 1);
                            parent.spawn((
                                Text::new(format!("- {} x{}", item_name, cost.quantity)),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(200, 190, 170)),
                            ));
                        }
                    }
                });
            }
        }
    }
}

fn populate_empty_content(
    commands: &mut Commands,
    grid_query: &Query<Entity, With<BuildingGridContainer>>,
    category: ActionCategory,
) {
    for entity in grid_query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text::new(format!("{:?}: Pas encore implémenté", category)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(223, 210, 194)),
            ));
        });
    }
}

fn update_run_button(
    _commands: &mut Commands,
    panel_query: &Query<Entity, With<ActionsPanelMarker>>,
    action_state: &ActionState,
    _asset_server: &Res<AssetServer>,
) {
    // Check if we should show the run button
    let should_show = match action_state.selected_category {
        Some(ActionCategory::Buildings) => action_state.selected_building_id.is_some(),
        Some(ActionCategory::Roads) => true, // Always show for roads
        _ => false,
    };

    if should_show {
        for _entity in panel_query.iter() {
            // Check if run button already exists
            // For now, we'll just ensure it's there
            // TODO: Check if action card already exists before inserting
        }
    }
}

fn get_buildings_for_category(
    category: BuildingCategoryEnum,
) -> Vec<(&'static str, &'static str, &'static str)> {
    match category {
        BuildingCategoryEnum::Residential => vec![
            ("base_camp", "Campement", "sprites/buildings/base_camp_01.png"),
            ("hut_tier_i", "Hutte", "sprites/buildings/forge_01.png"),
            ("cottage_tier_i", "Chaumière", "sprites/buildings/cottage_tier_i_01.png"),
        ],
        BuildingCategoryEnum::Metal => vec![
            ("forge", "Forge", "sprites/buildings/forge_01.png"),
            ("smelter", "Fonderie", "sprites/buildings/smelter_01.png"),
        ],
        BuildingCategoryEnum::Earth => vec![
            ("glassworks", "Verrerie", "sprites/buildings/glassworks_01.png"),
        ],
        BuildingCategoryEnum::Wood => vec![
            ("carpenter_workshop", "Menuiserie", "sprites/buildings/carpenter_workshop_01.png"),
        ],
        BuildingCategoryEnum::Food => vec![
            ("farm", "Ferme", "sprites/buildings/farm_01.png"),
            ("kitchen", "Cuisine", "sprites/buildings/kitchen_01.png"),
            ("brewery", "Brasserie", "sprites/buildings/brewery_01.png"),
            ("slaughterhouse", "Abattoir", "sprites/buildings/slaughterhouse_01.png"),
            ("ice_house", "Glacière", "sprites/buildings/ice_house_01.png"),
        ],
        BuildingCategoryEnum::AnimalBreeding => vec![
            ("cowshed", "Étable", "sprites/buildings/cowshed_01.png"),
            ("pigsty", "Porcherie", "sprites/buildings/pigsty_01.png"),
            ("sheepfold", "Bergerie", "sprites/buildings/sheepfold_01.png"),
            ("stable", "Écurie", "sprites/buildings/stable_01.png"),
        ],
        BuildingCategoryEnum::ServiceDedicated => vec![
            ("theater", "Théâtre", "sprites/buildings/theater_01.png"),
            ("place_of_worship", "Lieu de Culte", "sprites/buildings/place_of_worship_01.png"),
            ("marketplace", "Marché", "sprites/buildings/marketplace_01.png"),
        ],
        _ => vec![],
    }
}

pub fn handle_building_button_interactions(
    mut query: Query<(&BuildingButton, &Interaction), Changed<Interaction>>,
    mut action_state: ResMut<ActionState>,
) {
    for (building_button, interaction) in &mut query {
        if *interaction == Interaction::Pressed {
            action_state.select_building(building_button.building_id.clone());
            info!(
                "Building selected: {} ({})",
                building_button.building_name, building_button.building_id
            );
        }
    }
}

pub fn handle_action_run_button(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ActionRunButton>),
    >,
    action_state: Res<ActionState>,
    grid_config: Res<GridConfig>,
    connection: Res<ConnectionStatus>,
    mut selected_hexes: ResMut<SelectedHexes>,
    mut build_building_events: MessageWriter<SendActionBuildBuilding>,
    mut build_road_events: MessageWriter<SendActionBuildRoad>,
) {
    for (interaction, mut background_color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 120, 80));
                info!("Run action pressed!");
                execute_action(
                    &action_state,
                    &grid_config,
                    &connection,
                    &mut selected_hexes,
                    &mut build_building_events,
                    &mut build_road_events,
                );
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb_u8(100, 150, 100));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 130, 80));
            }
        }
    }
}

fn execute_action(
    action_state: &ActionState,
    grid_config: &Res<GridConfig>,
    connection: &ConnectionStatus,
    selected_hexes: &mut SelectedHexes,
    build_building_events: &mut MessageWriter<SendActionBuildBuilding>,
    build_road_events: &mut MessageWriter<SendActionBuildRoad>,
) {
    // Check if connected
    if !connection.logged_in {
        warn!("Cannot execute action: not logged in");
        return;
    }

    let Some(_player_id) = connection.player_id else {
        warn!("Cannot execute action: no player ID");
        return;
    };

    // Get the first selected hex (for now, we only support single selection)
    let Some(&selected_hex) = selected_hexes.ids.iter().next() else {
        warn!("Cannot execute action: no cell selected");
        return;
    };

    // Convert Hex to GridCell
    let cell = GridCell {
        q: selected_hex.x,
        r: selected_hex.y,
    };

    let world_pos = grid_config.layout.hex_to_world_pos(selected_hex);

    let chunk_id = TerrainChunkId {
        x: world_pos.x.div_euclid(constants::CHUNK_SIZE.x).ceil() as i32,
        y: world_pos.y.div_euclid(constants::CHUNK_SIZE.y).ceil() as i32,
    };

    match action_state.selected_category {
        Some(ActionCategory::Buildings) => {
            if let Some(building_id) = &action_state.selected_building_id {
                info!(
                    "Executing building construction: {} at cell {:?}",
                    building_id, cell
                );

                // Map building_id to BuildingTypeEnum via to_name_lowercase
                let Some(building_type) = BuildingTypeEnum::iter()
                    .find(|bt| bt.to_name_lowercase() == building_id.as_str())
                else {
                    warn!("Unknown building type: {}", building_id);
                    return;
                };

                build_building_events.write(SendActionBuildBuilding {
                    chunk_id,
                    cell,
                    building_type,
                });

                info!("✓ Building construction request sent via lightyear");
            }
        }
        Some(ActionCategory::Roads) => {
            info!("Executing road construction with selected hexes");

            let hexes_vec: Vec<_> = selected_hexes.ids.iter().copied().collect();

            if hexes_vec.is_empty() {
                warn!("No hexes selected for road construction");
                return;
            }

            let start_hex = hexes_vec.first().unwrap();
            let end_hex = if hexes_vec.len() == 1 {
                start_hex
            } else {
                hexes_vec.last().unwrap()
            };

            let start_cell = shared::grid::GridCell::from_hex(start_hex);
            let end_cell = shared::grid::GridCell::from_hex(end_hex);

            info!("Building road from {:?} to {:?}", start_cell, end_cell);

            build_road_events.write(SendActionBuildRoad {
                start_cell,
                end_cell,
            });

            selected_hexes.clear();

            info!("✓ Road construction request sent via lightyear");
        }
        _ => {
            warn!("No action to execute");
        }
    }
}
