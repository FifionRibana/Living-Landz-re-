use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use shared::{ActionEntry, ActionModeEnum, GameDataRef};

use crate::camera::resources::SceneRenderTarget;
use crate::state::resources::{GameDataCache, InventoryCache, PlayerInfo};
use crate::states::AppState;
use crate::ui::carousel::components::{Carousel, CarouselAlpha, CarouselItem};
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
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
    pub executable: bool,
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

// ─── Arguments  ─────────────────────────────────────────────
type TitleQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<ActionPanelTitle>,
        Without<ActionPanelSubtitle>,
        Without<ActionPanelEmpty>,
    ),
>;

type SubtitleQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<ActionPanelSubtitle>,
        Without<ActionPanelTitle>,
        Without<ActionPanelEmpty>,
    ),
>;

type EmptyQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut Visibility),
    (
        With<ActionPanelEmpty>,
        Without<ActionPanelTitle>,
        Without<ActionPanelSubtitle>,
    ),
>;

type ListVisQuery<'w, 's> =
    Query<'w, 's, &'static mut Visibility, (With<ActionPanelList>, Without<ActionPanelEmpty>)>;

// ─── Arguments  ─────────────────────────────────────────────

/// Queries for the panel's text/visibility elements (header, subtitle, empty state)
#[derive(SystemParam)]
pub struct ActionPanelQueries<'w, 's> {
    pub list: Query<'w, 's, Entity, With<ActionPanelList>>,
    pub entries: Query<'w, 's, Entity, With<ActionPanelEntry>>,
    pub carousels: Query<'w, 's, Entity, With<Carousel>>,
    pub title: TitleQuery<'w, 's>,
    pub subtitle: SubtitleQuery<'w, 's>,
    pub empty: EmptyQuery<'w, 's>,
    pub list_vis: ListVisQuery<'w, 's>,
}

/// Shared resources needed to build frosted glass cards
#[derive(SystemParam)]
pub struct CardResources<'w> {
    pub asset_server: Res<'w, AssetServer>,
    pub render_target: Res<'w, SceneRenderTarget>,
    pub materials: ResMut<'w, Assets<FrostedGlassMaterial>>,
}

/// Game data needed for action resolution
#[derive(SystemParam)]
pub struct ActionDataResources<'w> {
    pub game_data_cache: Res<'w, GameDataCache>,
    pub inventory_cache: Res<'w, InventoryCache>,
    pub player_info: Res<'w, PlayerInfo>,
}

// ─── Setup ──────────────────────────────────────────────────

/// Spawn the action panel (initially hidden). Lives for the InGame state.
pub fn setup_action_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(80.0),
                right: Val::Px(80.0),
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
                    overflow: Overflow::clip(),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
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
    mut carousel_query: Query<&mut Carousel>,
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

    // Enable/disable action carousel based on panel visibility
    for mut carousel in carousel_query.iter_mut() {
        if carousel.id == 1 {
            carousel.enabled = should_show;
        }
    }
}

// ─── Content ────────────────────────────────────────────────

/// Rebuild the action list when mode or context changes.
pub fn update_action_panel_content(
    mut commands: Commands,
    ui_state: Res<UIState>,
    action_context: Res<ActionContextState>,
    mut panel: ActionPanelQueries,
    mut cards: CardResources,
    data: ActionDataResources,
    windows: Query<&Window>,
    mut last_action_ids: Local<Vec<String>>,
) {
    if !ui_state.is_changed() && !action_context.is_changed() && !data.inventory_cache.is_changed()
    {
        return;
    }

    // Despawn old entries
    for entity in &panel.entries {
        commands.entity(entity).despawn();
    }

    let Some(mode) = ui_state.action_mode else {
        if !last_action_ids.is_empty() {
            for entity in &panel.entries {
                commands.entity(entity).despawn();
            }
            for entity in &panel.carousels {
                commands.entity(entity).despawn();
            }
            last_action_ids.clear();
        }
        return;
    };

    let Some(ctx) = action_context.get() else {
        return;
    };

    // Update title
    for mut text in &mut panel.title {
        **text = mode.to_name().to_string();
    }

    // Build GameDataRef from cache for DB-driven actions
    let game_data_ref = if data.game_data_cache.loaded {
        let item_names: std::collections::HashMap<i32, String> = data
            .game_data_cache
            .items
            .iter()
            .map(|i| (i.id, data.game_data_cache.item_name(i.id, 1)))
            .collect();

        // Build inventory summary from cache
        let inventory: std::collections::HashMap<i32, i32> = data
            .player_info
            .lord
            .as_ref()
            .and_then(|lord| data.inventory_cache.get_inventory(lord.id))
            .map(|items| items.iter().map(|i| (i.item_id, i.quantity)).collect())
            .unwrap_or_default();

        Some(GameDataRef {
            items: &data.game_data_cache.items,
            recipes: &data.game_data_cache.recipes,
            construction_costs: &data.game_data_cache.construction_costs,
            item_names,
            inventory,
            dev_mode: data.game_data_cache.dev_mode,
        })
    } else {
        None
    };

    // Get available actions
    let actions = mode.available_actions(ctx, game_data_ref.as_ref());

    let new_ids: Vec<String> = actions.iter().map(|a| a.id.clone()).collect();
    if *last_action_ids == new_ids {
        // Actions haven't changed — don't rebuild carousel
        // TODO: update executable state on existing cards if inventory changed
        return;
    }
    *last_action_ids = new_ids;

    // Despawn old entries + carousel
    for entity in &panel.entries {
        commands.entity(entity).despawn();
    }
    for entity in &panel.carousels {
        commands.entity(entity).despawn();
    }

    // Update subtitle with context info
    for mut text in &mut panel.subtitle {
        let building_name = ctx
            .building
            .map(|b| b.to_name_lowercase())
            .unwrap_or("terrain nu");
        let count = actions.len();
        **text = format!(
            "{} — {} action{}",
            building_name,
            count,
            if count > 1 { "s" } else { "" }
        );
    }

    // Show empty or list
    let has_actions = !actions.is_empty();

    for (mut text, mut vis) in &mut panel.empty {
        *vis = if has_actions {
            Visibility::Hidden
        } else {
            **text = "Aucune action disponible dans ce contexte".to_string();
            Visibility::Visible
        };
    }

    for mut vis in &mut panel.list_vis {
        *vis = if has_actions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Spawn entries
    let Ok(list_entity) = panel.list.single() else {
        return;
    };

    // Constants
    const CARD_WIDTH: f32 = 150.0;
    const CARD_HEIGHT: f32 = 160.0;
    const CARD_GAP: f32 = 20.0;

    // Determine layout mode
    let panel_width = windows
        .single()
        .map(|w| w.width() - 100.0) // rough panel width
        .unwrap_or(600.0);
    let max_visible = 5; //((panel_width + CARD_GAP) / (CARD_WIDTH + CARD_GAP)).floor() as usize;
    let use_carousel = actions.len() > max_visible; // && actions.len() > 3;

    // Unique ID for this carousel instance
    let carousel_id = 1u32; // Messages uses 0

    if use_carousel {
        // ── CAROUSEL MODE ──
        let carousel_entity = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(CARD_HEIGHT + 20.0),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Carousel {
                    id: carousel_id,
                    enabled: true,
                    item_width: CARD_WIDTH,
                    spacing: CARD_GAP,
                    total_items: actions.len(),
                    current_scroll: 0.0,
                    target_scroll: 0.0,
                    lerp_speed: 10.0,
                    snap_timer: 0.0,
                },
            ))
            .id();

        for (i, action) in actions.iter().enumerate() {
            let card = spawn_action_card(
                &mut commands,
                &cards.asset_server,
                &mut cards.materials,
                &cards.render_target,
                action,
                CARD_WIDTH,
                CARD_HEIGHT,
            );
            commands
                .entity(card)
                .insert((CarouselItem { carousel_id, index: i },));
            commands.entity(carousel_entity).add_child(card);
        }

        commands.entity(list_entity).add_child(carousel_entity);
    } else {
        // ── SIMPLE LAYOUT ──
        // Flex-row, centered
        let row = commands
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(CARD_GAP),
                ..default()
            })
            .id();

        for action in &actions {
            let card = spawn_action_card(
                &mut commands,
                &cards.asset_server,
                &mut cards.materials,
                &cards.render_target,
                action,
                CARD_WIDTH,
                CARD_HEIGHT,
            );
            commands.entity(row).add_child(card);
        }

        commands.entity(list_entity).add_child(row);
    }
}

fn spawn_action_card(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<FrostedGlassMaterial>,
    render_target: &SceneRenderTarget,
    action: &ActionEntry,
    width: f32,
    height: f32,
) -> Entity {
    let (name_color, desc_color, cost_color, output_color, card_opacity) = if action.executable {
        (
            Color::srgb_u8(67, 60, 37),
            Color::srgb_u8(120, 110, 90),
            Color::srgb_u8(160, 100, 60),
            Color::srgb_u8(60, 130, 80),
            1.0_f32,
        )
    } else {
        (
            Color::srgba_u8(67, 60, 37, 100),
            Color::srgba_u8(120, 110, 90, 80),
            Color::srgba_u8(200, 60, 60, 120),
            Color::srgba_u8(60, 130, 80, 80),
            0.4,
        )
    };

    let material = materials.add(FrostedGlassMaterial::from(
        FrostedGlassConfig::card()
            .with_border_radius(10.0)
            .with_scene_texture(render_target.0.clone()),
    ));

    let card = commands
        .spawn((
            Interaction::default(),
            MaterialNode(material),
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            BorderRadius::all(Val::Px(10.0)),
            // Pickable {
            //     should_block_lower: true,
            //     is_hoverable: true,
            // },
            ActionPanelEntry {
                action_id: action.id.clone(),
                executable: action.executable,
            },
        ))
        .with_children(|card| {
            // ── Top section: Icon + Name ──
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                CarouselAlpha::new(card_opacity),
            ))
            .with_children(|top| {
                if !action.icon.is_empty() {
                    top.spawn((
                        ImageNode {
                            image: asset_server.load(&action.icon),
                            color: Color::srgba(1.0, 1.0, 1.0, card_opacity),
                            ..default()
                        },
                        Node {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            ..default()
                        },
                        CarouselAlpha::new(card_opacity),
                        Pickable::IGNORE,
                    ));
                }
                top.spawn((
                    Text::new(&action.name),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(name_color),
                    CarouselAlpha::new(card_opacity),
                    Pickable::IGNORE,
                ));
            });

            // ── Middle: Description ──
            if !action.description.is_empty() && action.description != action.name {
                card.spawn((
                    Text::new(&action.description),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(desc_color),
                    CarouselAlpha::new(card_opacity),
                    Pickable::IGNORE,
                ));
            }

            // ── Bottom section: Costs + Outputs ──
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                CarouselAlpha::new(card_opacity),
            ))
            .with_children(|bottom| {
                // Costs
                if !action.costs.is_empty() {
                    let costs_text = action
                        .costs
                        .iter()
                        .map(|c| format!("{} ×{}", c.name, c.quantity))
                        .collect::<Vec<_>>()
                        .join(", ");

                    bottom.spawn((
                        Text::new(format!("▼ {}", costs_text)),
                        TextFont {
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(cost_color),
                        CarouselAlpha::new(card_opacity),
                        Pickable::IGNORE,
                    ));
                }

                // Outputs
                if !action.outputs.is_empty() {
                    let outputs_text = action
                        .outputs
                        .iter()
                        .map(|o| format!("{} ×{}", o.name, o.quantity))
                        .collect::<Vec<_>>()
                        .join(", ");

                    bottom.spawn((
                        Text::new(format!("▲ {}", outputs_text)),
                        TextFont {
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(output_color),
                        CarouselAlpha::new(card_opacity),
                        Pickable::IGNORE,
                    ));
                }
            });
        })
        .id();

    card
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

        // Block non-executable actions
        if !entry.executable {
            info!(
                "Action {} is not executable (missing resources)",
                entry.action_id
            );
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
                x: world_pos
                    .x
                    .div_euclid(shared::constants::CHUNK_SIZE.x)
                    .ceil() as i32,
                y: world_pos
                    .y
                    .div_euclid(shared::constants::CHUNK_SIZE.y)
                    .ceil() as i32,
            };
            (cd.cell, chunk)
        } else if let Some(&hex) = selected_hexes.ids.iter().next() {
            let cell = shared::grid::GridCell { q: hex.x, r: hex.y };
            let world_pos = grid_config.layout.hex_to_world_pos(hex);
            let chunk = shared::TerrainChunkId {
                x: world_pos
                    .x
                    .div_euclid(shared::constants::CHUNK_SIZE.x)
                    .ceil() as i32,
                y: world_pos
                    .y
                    .div_euclid(shared::constants::CHUNK_SIZE.y)
                    .ceil() as i32,
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
                "settler" => shared::ProfessionEnum::Settler,
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
                info!(
                    "✓ Train {} request sent for unit {}",
                    profession_str, unit_id
                );
            } else {
                warn!("No unit selected for training");
            }
        }
        // ── Upgrade actions ──
        else if action_id.starts_with("upgrade_") {
            info!("Upgrade action: {} (not yet implemented)", action_id);
        }
        // ── Diplomacy actions ──
        else if matches!(
            action_id.as_str(),
            "send_envoy" | "propose_trade" | "research"
        ) {
            info!("Diplomacy action: {} (not yet implemented)", action_id);
        } else {
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
