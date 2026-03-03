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

/// Handle click on action entry buttons.
pub fn handle_action_entry_click(
    entry_query: Query<(&Interaction, &ActionPanelEntry), Changed<Interaction>>,
) {
    for (interaction, entry) in &entry_query {
        if matches!(interaction, Interaction::Pressed) {
            info!("Action selected: {}", entry.action_id);
            // TODO: dispatch to action execution system
        }
    }
}
