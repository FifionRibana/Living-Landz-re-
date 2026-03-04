use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use shared::{ActionEntry, ActionModeEnum};

use crate::states::AppState;
use crate::ui::resources::{ActionContextState, UIState};

// ─── Markers ────────────────────────────────────────────────

/// Root container for the action panel.
#[derive(Component)]
pub struct ActionPanelRoot;

/// The scrollable list of action entries inside the panel.
#[derive(Component)]
pub struct ActionPanelList;

/// A single action entry button.
#[derive(Component)]
pub struct ActionPanelEntry {
    pub action_id: String,
}

/// Title text of the panel.
#[derive(Component)]
pub struct ActionPanelTitle;

/// Subtitle text showing context info.
#[derive(Component)]
pub struct ActionPanelSubtitle;

/// Empty state text when no actions available.
#[derive(Component)]
pub struct ActionPanelEmpty;

// ─── Setup ──────────────────────────────────────────────────

/// Spawn the action panel (initially hidden). Lives for the InGame state.
pub fn setup_action_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paper_panel_image = asset_server.load("ui/ui_paper_panel_md.png");
    let paper_panel_slicer = TextureSlicer {
        border: BorderRect {
            left: 42.,
            right: 42.,
            top: 76.,
            bottom: 42.,
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
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(80.0),
                right: Val::Px(10.0),
                max_height: Val::Px(220.0),
                padding: UiRect {
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(14.0),
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(900),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            ActionPanelRoot,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            // Header row: title + subtitle
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new("Actions"),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(67, 60, 37)),
                    ActionPanelTitle,
                ));
                header.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(120, 110, 90)),
                    ActionPanelSubtitle,
                ));
            });

            // Empty state text
            root.spawn((
                Text::new("Sélectionnez une catégorie d'action"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(140, 130, 100)),
                ActionPanelEmpty,
            ));

            // Scrollable list of action entries
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    overflow: Overflow::scroll_y(),
                    align_content: AlignContent::FlexStart,
                    ..default()
                },
                Visibility::Hidden,
                ActionPanelList,
            ));
        });
}

// ─── Visibility ─────────────────────────────────────────────

/// Show/hide the panel based on whether a mode is selected.
pub fn update_action_panel_visibility(
    ui_state: Res<UIState>,
    action_context: Res<ActionContextState>,
    mut root_query: Query<&mut Visibility, With<ActionPanelRoot>>,
) {
    let should_show = ui_state.action_mode.is_some() && action_context.get().is_some();

    for mut vis in &mut root_query {
        let target = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }
}

// ─── Content ────────────────────────────────────────────────

/// Rebuild the action list when mode or context changes.
pub fn update_action_panel_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_state: Res<UIState>,
    action_context: Res<ActionContextState>,
    list_query: Query<Entity, With<ActionPanelList>>,
    existing_entries: Query<Entity, With<ActionPanelEntry>>,
    mut title_query: Query<&mut Text, (With<ActionPanelTitle>, Without<ActionPanelSubtitle>, Without<ActionPanelEmpty>)>,
    mut subtitle_query: Query<&mut Text, (With<ActionPanelSubtitle>, Without<ActionPanelTitle>, Without<ActionPanelEmpty>)>,
    mut empty_query: Query<(&mut Text, &mut Visibility), (With<ActionPanelEmpty>, Without<ActionPanelTitle>, Without<ActionPanelSubtitle>)>,
    mut list_vis_query: Query<&mut Visibility, (With<ActionPanelList>, Without<ActionPanelEmpty>)>,
) {
    if !ui_state.is_changed() && !action_context.is_changed() {
        return;
    }

    // Despawn old entries
    for entity in &existing_entries {
        commands.entity(entity).despawn();
    }

    let Some(mode) = ui_state.action_mode else {
        return;
    };

    let Some(ctx) = action_context.get() else {
        return;
    };

    // Update title
    for mut text in &mut title_query {
        **text = mode.to_name().to_string();
    }

    // Get available actions
    let actions = mode.available_actions(ctx);

    // Update subtitle with context info
    for mut text in &mut subtitle_query {
        let building_name = ctx
            .building
            .map(|b| b.to_name_lowercase())
            .unwrap_or("terrain nu");
        let count = actions.len();
        **text = format!("{} — {} action{}", building_name, count, if count > 1 { "s" } else { "" });
    }

    // Show empty or list
    let has_actions = !actions.is_empty();

    for (mut text, mut vis) in &mut empty_query {
        *vis = if has_actions {
            Visibility::Hidden
        } else {
            **text = "Aucune action disponible dans ce contexte".to_string();
            Visibility::Visible
        };
    }

    for mut vis in &mut list_vis_query {
        *vis = if has_actions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Spawn entries
    let Ok(list_entity) = list_query.single() else {
        return;
    };

    let paper_btn_image = asset_server.load("ui/ui_paper_panel_md.png");
    let paper_btn_slicer = TextureSlicer {
        border: BorderRect::all(20.0),
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    for action in actions {
        let entry_entity = commands
            .spawn((
                Button,
                ImageNode {
                    image: paper_btn_image.clone(),
                    image_mode: NodeImageMode::Sliced(paper_btn_slicer.clone()),
                    ..default()
                },
                Node {
                    width: Val::Px(180.0),
                    min_height: Val::Px(60.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                ActionPanelEntry {
                    action_id: action.id.clone(),
                },
            ))
            .with_children(|parent| {
                // Icon
                if !action.icon.is_empty() {
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load(&action.icon),
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
                            Text::new(&action.name),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(67, 60, 37)),
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));

                        // Description
                        if !action.description.is_empty() {
                            col.spawn((
                                Text::new(&action.description),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(120, 110, 90)),
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                        }

                        // Costs row
                        if !action.costs.is_empty() {
                            col.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(6.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ))
                            .with_children(|costs_row| {
                                costs_row.spawn((
                                    Text::new("▼ "),
                                    TextFont {
                                        font_size: 9.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb_u8(160, 100, 60)),
                                    Pickable {
                                        should_block_lower: false,
                                        is_hoverable: false,
                                    },
                                ));
                                for cost in &action.costs {
                                    costs_row.spawn((
                                        Text::new(format!("{} ×{}", cost.name, cost.quantity)),
                                        TextFont {
                                            font_size: 9.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(160, 100, 60)),
                                        Pickable {
                                            should_block_lower: false,
                                            is_hoverable: false,
                                        },
                                    ));
                                }
                            });
                        }

                        // Outputs row
                        if !action.outputs.is_empty() {
                            col.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(6.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ))
                            .with_children(|outputs_row| {
                                outputs_row.spawn((
                                    Text::new("▲ "),
                                    TextFont {
                                        font_size: 9.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb_u8(60, 130, 80)),
                                    Pickable {
                                        should_block_lower: false,
                                        is_hoverable: false,
                                    },
                                ));
                                for output in &action.outputs {
                                    outputs_row.spawn((
                                        Text::new(format!("{} ×{}", output.name, output.quantity)),
                                        TextFont {
                                            font_size: 9.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(60, 130, 80)),
                                        Pickable {
                                            should_block_lower: false,
                                            is_hoverable: false,
                                        },
                                    ));
                                }
                            });
                        }

                        // Duration
                        col.spawn((
                            Text::new(format!("{} tick{}", action.duration_ticks, if action.duration_ticks > 1 { "s" } else { "" })),
                            TextFont {
                                font_size: 9.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(100, 120, 100)),
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                    });
            })
            .id();

        commands.entity(list_entity).add_child(entry_entity);
    }
}

// ─── Interactions ───────────────────────────────────────────

/// Handle click on action entry buttons — dispatch to network.
pub fn handle_action_entry_click(
    entry_query: Query<(&Interaction, &ActionPanelEntry), Changed<Interaction>>,
    mut network_client_opt: Option<ResMut<crate::networking::client::NetworkClient>>,
    connection: Res<crate::state::resources::ConnectionStatus>,
    cell_state: Res<crate::ui::resources::CellState>,
    unit_selection: Res<crate::ui::resources::UnitSelectionState>,
    grid_config: Res<shared::grid::GridConfig>,
    selected_hexes: Res<crate::grid::resources::SelectedHexes>,
) {
    for (interaction, entry) in &entry_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        info!("Action selected: {}", entry.action_id);

        // Validate connection
        if !connection.logged_in {
            warn!("Cannot execute action: not logged in");
            return;
        }
        let Some(player_id) = connection.player_id else {
            warn!("Cannot execute action: no player ID");
            return;
        };
        let Some(network_client) = network_client_opt.as_mut() else {
            warn!("Cannot execute action: no network client");
            return;
        };

        // Resolve cell + chunk
        let (cell, chunk_id) = if let Some(cd) = &cell_state.cell_data {
            let hex = cd.cell.to_hex();
            let world_pos = grid_config.layout.hex_to_world_pos(hex);
            let chunk = shared::TerrainChunkId {
                x: world_pos.x.div_euclid(shared::constants::CHUNK_SIZE.x).ceil() as i32,
                y: world_pos.y.div_euclid(shared::constants::CHUNK_SIZE.y).ceil() as i32,
            };
            (cd.cell, chunk)
        } else if let Some(&hex) = selected_hexes.ids.iter().next() {
            let cell = shared::grid::GridCell { q: hex.x, r: hex.y };
            let world_pos = grid_config.layout.hex_to_world_pos(hex);
            let chunk = shared::TerrainChunkId {
                x: world_pos.x.div_euclid(shared::constants::CHUNK_SIZE.x).ceil() as i32,
                y: world_pos.y.div_euclid(shared::constants::CHUNK_SIZE.y).ceil() as i32,
            };
            (cell, chunk)
        } else {
            warn!("Cannot execute action: no cell context");
            return;
        };

        let action_id = &entry.action_id;

        // ── Build actions ──
        if let Some(building_id) = action_id.strip_prefix("build_") {
            let building_type = match building_id {
                "blacksmith" => shared::BuildingTypeEnum::Blacksmith,
                "carpenter_shop" => shared::BuildingTypeEnum::CarpenterShop,
                "farm" => shared::BuildingTypeEnum::Farm,
                "bakehouse" => shared::BuildingTypeEnum::Bakehouse,
                "brewery" => shared::BuildingTypeEnum::Brewery,
                "market" => shared::BuildingTypeEnum::Market,
                "cowshed" => shared::BuildingTypeEnum::Cowshed,
                "sheepfold" => shared::BuildingTypeEnum::Sheepfold,
                "stable" => shared::BuildingTypeEnum::Stable,
                "temple" => shared::BuildingTypeEnum::Temple,
                "theater" => shared::BuildingTypeEnum::Theater,
                "road_segment" => {
                    // Road segment uses BuildRoad message
                    network_client.send_message(shared::protocol::ClientMessage::ActionBuildRoad {
                        player_id,
                        start_cell: cell,
                        end_cell: cell, // TODO: target adjacent cell
                    });
                    info!("✓ Road segment request sent");
                    return;
                }
                other => {
                    warn!("Unknown building: {}", other);
                    return;
                }
            };

            network_client.send_message(shared::protocol::ClientMessage::ActionBuildBuilding {
                player_id,
                chunk_id,
                cell,
                building_type,
            });
            info!("✓ Build {} request sent", building_id);
        }
        // ── Road planning (map view) ──
        else if action_id.starts_with("plan_") {
            let hexes: Vec<_> = selected_hexes.ids.iter().copied().collect();
            if hexes.len() < 2 {
                warn!("Road planning requires at least 2 selected hexes");
                return;
            }
            let start = shared::grid::GridCell::from_hex(&hexes[0]);
            let end = shared::grid::GridCell::from_hex(hexes.last().unwrap());

            network_client.send_message(shared::protocol::ClientMessage::ActionBuildRoad {
                player_id,
                start_cell: start,
                end_cell: end,
            });
            info!("✓ Road plan request sent");
        }
        // ── Production actions ──
        else if let Some(recipe_id) = action_id.strip_prefix("produce_") {
            network_client.send_message(shared::protocol::ClientMessage::ActionCraftResource {
                player_id,
                chunk_id,
                cell,
                recipe_id: recipe_id.to_string(),
                quantity: 1,
            });
            info!("✓ Production {} request sent", recipe_id);
        }
        // ── Trade actions ──
        else if action_id.starts_with("trade_") {
            // TODO: open trade dialog
            info!("Trade action: {} (not yet implemented)", action_id);
        }
        // ── Training actions ──
        else if let Some(profession_str) = action_id.strip_prefix("train_") {
            let target_profession = match profession_str {
                "baker" => shared::ProfessionEnum::Baker,
                "farmer" => shared::ProfessionEnum::Farmer,
                "warrior" => shared::ProfessionEnum::Warrior,
                "blacksmith" => shared::ProfessionEnum::Blacksmith,
                "carpenter" => shared::ProfessionEnum::Carpenter,
                "miner" => shared::ProfessionEnum::Miner,
                "merchant" => shared::ProfessionEnum::Merchant,
                "hunter" => shared::ProfessionEnum::Hunter,
                "healer" => shared::ProfessionEnum::Healer,
                "scholar" => shared::ProfessionEnum::Scholar,
                "cook" => shared::ProfessionEnum::Cook,
                "fisherman" => shared::ProfessionEnum::Fisherman,
                "lumberjack" => shared::ProfessionEnum::Lumberjack,
                "mason" => shared::ProfessionEnum::Mason,
                "brewer" => shared::ProfessionEnum::Brewer,
                other => {
                    warn!("Unknown profession: {}", other);
                    return;
                }
            };

            // Train first selected unit
            if let Some(&unit_id) = unit_selection.selected_ids().first() {
                network_client.send_message(shared::protocol::ClientMessage::ActionTrainUnit {
                    player_id,
                    unit_id,
                    chunk_id,
                    cell,
                    target_profession,
                });
                info!("✓ Train {} request sent for unit {}", profession_str, unit_id);
            } else {
                warn!("No unit selected for training");
            }
        }
        // ── Upgrade actions ──
        else if action_id.starts_with("upgrade_") {
            info!("Upgrade action: {} (not yet implemented)", action_id);
        }
        // ── Diplomacy actions ──
        else if matches!(action_id.as_str(), "send_envoy" | "propose_trade" | "research") {
            info!("Diplomacy action: {} (not yet implemented)", action_id);
        }
        else {
            warn!("Unknown action: {}", action_id);
        }
    }
}

/// Visual feedback on hover for action entry buttons.
pub fn update_action_entry_hover(
    mut entry_query: Query<
        (&Interaction, &mut ImageNode),
        (With<ActionPanelEntry>, Changed<Interaction>),
    >,
    asset_server: Res<AssetServer>,
) {
    let paper_normal: Handle<Image> = asset_server.load("ui/ui_paper_panel_md.png");

    for (interaction, mut image) in &mut entry_query {
        match interaction {
            Interaction::Pressed => {
                image.color = Color::srgb(0.85, 0.8, 0.7);
            }
            Interaction::Hovered => {
                image.color = Color::srgb(0.95, 0.92, 0.85);
            }
            Interaction::None => {
                image.color = Color::WHITE;
            }
        }
    }
}
