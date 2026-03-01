use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy::state::state_scoped::DespawnOnExit;

use crate::states::AppState;
use super::components::*;
use super::resources::{CharacterCreationState, Gender, LayerCategory};

// ─── Palette ────────────────────────────────────────────────────────────────

const PANEL_BG: Color = Color::srgba(0.12, 0.09, 0.06, 0.95);
const PANEL_BORDER: Color = Color::srgb(0.55, 0.45, 0.30);
const SEPARATOR: Color = Color::srgba(0.80, 0.66, 0.30, 0.15);
const GOLD: Color = Color::srgb(0.79, 0.66, 0.30);
const GOLD_DIM: Color = Color::srgb(0.63, 0.50, 0.19);
const TEXT_LIGHT: Color = Color::srgb(0.92, 0.88, 0.80);
const TEXT_DIM: Color = Color::srgb(0.42, 0.30, 0.19);
const ARROW_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.10);
const ARROW_BORDER: Color = Color::srgba(0.55, 0.45, 0.30, 0.5);
const BTN_BG: Color = Color::srgb(0.23, 0.18, 0.12);
const BTN_PRIMARY_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.20);
const PREVIEW_BG: Color = Color::srgb(0.15, 0.11, 0.08);
const FRAME_BORDER: Color = Color::srgb(0.66, 0.56, 0.44);
const GENDER_ACTIVE_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.25);
const GENDER_INACTIVE_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.05);
const GENDER_ACTIVE_BORDER: Color = Color::srgb(0.79, 0.66, 0.30);
const GENDER_INACTIVE_BORDER: Color = Color::srgba(0.55, 0.45, 0.30, 0.3);

// ─── Setup system ───────────────────────────────────────────────────────────

pub fn setup_character_creation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    creation_state: Res<CharacterCreationState>,
) {
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_regular = asset_server.load("fonts/FiraSans-Regular.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.08, 0.05)),
            DespawnOnExit(AppState::CharacterCreation),
            CharacterCreationPanel,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(860.0),
                    height: Val::Px(540.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
                BorderColor::all(PANEL_BORDER),
                BorderRadius::all(Val::Px(6.0)),
            ))
            .with_children(|container| {
                spawn_preview_panel(container, &font_bold, &font_regular, &asset_server, &creation_state);

                container.spawn((
                    Node {
                        width: Val::Px(1.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SEPARATOR),
                ));

                spawn_controls_panel(container, &font_bold, &font_regular, &creation_state);
            });
        });
}

// ─── Preview panel (left) ───────────────────────────────────────────────────

fn spawn_preview_panel(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font_bold: &Handle<Font>,
    font_regular: &Handle<Font>,
    asset_server: &Res<AssetServer>,
    creation_state: &CharacterCreationState,
) {
    parent
        .spawn((Node {
            width: Val::Px(340.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(12.0),
            ..default()
        },))
        .with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("Aperçu"),
                TextFont {
                    font: font_bold.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(GOLD_DIM),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Portrait frame — square
            panel
                .spawn((
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(260.0),
                        border: UiRect::all(Val::Px(2.0)),
                        position_type: PositionType::Relative,
                        overflow: Overflow::clip(),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(PREVIEW_BG),
                    BorderColor::all(FRAME_BORDER),
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|frame| {
                    for (i, layer) in creation_state.layers.iter().enumerate() {
                        frame.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                ..default()
                            },
                            ImageNode {
                                image: asset_server.load(layer.asset_path()),
                                ..default()
                            },
                            GlobalZIndex(10 + i as i32),
                            LayerPreviewImage {
                                category: layer.category,
                            },
                        ));
                    }

                    frame.spawn((
                        Text::new("?"),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.79, 0.66, 0.30, 0.10)),
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        GlobalZIndex(5),
                    ));
                });

            // Layer dots indicator
            panel
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                },))
                .with_children(|dots_row| {
                    for layer in &creation_state.layers {
                        dots_row.spawn((
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if layer.current > 0 { GOLD } else { Color::NONE }),
                            BorderColor::all(GOLD_DIM),
                            BorderRadius::all(Val::Px(4.0)),
                        ));
                    }

                    dots_row.spawn((
                        Text::new("couches"),
                        TextFont {
                            font: font_regular.clone(),
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                        Node {
                            margin: UiRect::left(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            // Spacer
            panel.spawn((Node { flex_grow: 1.0, ..default() },));

            // ─── Name section ───────────────────────────────────────────
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::top(Val::Px(12.0)),
                        border: UiRect::top(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(SEPARATOR),
                ))
                .with_children(|name_section| {
                    // Label row: "Prénom" + dice button
                    name_section
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },))
                        .with_children(|label_row| {
                            label_row.spawn((
                                Text::new("Prénom"),
                                TextFont {
                                    font: font_bold.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(GOLD_DIM),
                            ));

                            // Random name button (dice)
                            label_row
                                .spawn((
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(ARROW_BG),
                                    BorderColor::all(ARROW_BORDER),
                                    BorderRadius::all(Val::Px(3.0)),
                                    RandomNameButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("⚄"),
                                        TextFont {
                                            font: font_regular.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(GOLD),
                                    ));
                                });
                        });

                    // Text input
                    name_section.spawn((
                        bevy_ui_text_input::TextInputNode {
                            clear_on_submit: false,
                            ..default()
                        },
                        bevy_ui_text_input::TextInputBuffer::default(),
                        bevy_ui_text_input::TextInputQueue::default(),
                        bevy_ui_text_input::TextInputStyle::default(),
                        TextFont {
                            font: font_regular.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(TEXT_LIGHT),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(38.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                        BorderColor::all(PANEL_BORDER),
                        BorderRadius::all(Val::Px(3.0)),
                        FirstNameInput,
                    ));
                });
        });
}

// ─── Controls panel (right) ─────────────────────────────────────────────────

fn spawn_controls_panel(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font_bold: &Handle<Font>,
    font_regular: &Handle<Font>,
    creation_state: &CharacterCreationState,
) {
    parent
        .spawn((Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },))
        .with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("Apparence"),
                TextFont {
                    font: font_bold.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(GOLD_DIM),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // ─── Gender selector ────────────────────────────────────────
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        margin: UiRect::bottom(Val::Px(14.0)),
                        padding: UiRect::bottom(Val::Px(12.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(SEPARATOR),
                ))
                .with_children(|row| {
                    spawn_gender_button(row, font_bold, Gender::Male, creation_state.gender);
                    spawn_gender_button(row, font_bold, Gender::Female, creation_state.gender);
                });

            // ─── Layer selectors ────────────────────────────────────────
            for layer in &creation_state.layers {
                spawn_selector_row(panel, font_bold, font_regular, layer.category, &layer.counter_text());
            }

            // ─── Random all button ──────────────────────────────────────
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        height: Val::Px(34.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    Button,
                    BackgroundColor(ARROW_BG),
                    BorderColor::all(ARROW_BORDER),
                    BorderRadius::all(Val::Px(3.0)),
                    RandomAllButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("⚄"),
                        TextFont {
                            font: font_regular.clone(),
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(GOLD),
                    ));
                    btn.spawn((
                        Text::new("Aléatoire"),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(GOLD),
                    ));
                });

            // Future accessories (greyed)
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::top(Val::Px(8.0)),
                        padding: UiRect::top(Val::Px(10.0)),
                        border: UiRect::top(Val::Px(1.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BorderColor::all(SEPARATOR),
                ))
                .with_children(|future| {
                    future.spawn((
                        Text::new("Bientôt disponible"),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.42, 0.30, 0.19, 0.5)),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    spawn_future_item(future, font_regular, "Couvre-chef");
                    spawn_future_item(future, font_regular, "Bijoux & accessoires");
                });

            // Spacer
            panel.spawn((Node { flex_grow: 1.0, ..default() },));

            // Action buttons
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::top(Val::Px(14.0)),
                        border: UiRect::top(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(SEPARATOR),
                ))
                .with_children(|actions| {
                    spawn_action_button(
                        actions, font_bold, "Retour", BTN_BG, TEXT_LIGHT, PANEL_BORDER, BackToLoginButton,
                    );
                    spawn_action_button(
                        actions, font_bold, "Valider", BTN_PRIMARY_BG, GOLD, GOLD_DIM, ValidateButton,
                    );
                });
        });
}

// ─── Gender button ──────────────────────────────────────────────────────────

fn spawn_gender_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    gender: Gender,
    current: Gender,
) {
    let is_active = gender == current;
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(36.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(if is_active { GENDER_ACTIVE_BG } else { GENDER_INACTIVE_BG }),
            BorderColor::all(if is_active { GENDER_ACTIVE_BORDER } else { GENDER_INACTIVE_BORDER }),
            BorderRadius::all(Val::Px(3.0)),
            GenderButton { gender },
            GenderActiveIndicator { gender },
        ))
        .with_children(|btn| {
            let icon = match gender {
                Gender::Male => "♂",
                Gender::Female => "♀",
            };
            btn.spawn((
                Text::new(format!("{} {}", icon, gender.label())),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(if is_active { GOLD } else { TEXT_DIM }),
            ));
        });
}

// ─── Selector row for one layer ─────────────────────────────────────────────

fn spawn_selector_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font_bold: &Handle<Font>,
    font_regular: &Handle<Font>,
    category: LayerCategory,
    counter_text: &str,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(0.0), Val::Px(10.0)),
                column_gap: Val::Px(12.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(SEPARATOR),
        ))
        .with_children(|row| {
            // Icon box
            row.spawn((
                Node {
                    width: Val::Px(34.0),
                    height: Val::Px(34.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.79, 0.66, 0.30, 0.06)),
                BorderColor::all(Color::srgba(0.55, 0.45, 0.30, 0.3)),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|icon_box| {
                icon_box.spawn((
                    Text::new(category.icon()),
                    TextFont {
                        font: font_bold.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(GOLD_DIM),
                ));
            });

            // Label
            row.spawn((Node { flex_grow: 1.0, ..default() },))
                .with_children(|label_area| {
                    label_area.spawn((
                        Text::new(category.label()),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(TEXT_LIGHT),
                    ));
                });

            // Arrow controls
            row.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },))
            .with_children(|controls| {
                spawn_arrow(controls, font_regular, category, ArrowDirection::Prev, "◀");

                controls.spawn((
                    Text::new(counter_text),
                    TextFont {
                        font: font_regular.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.83, 0.77, 0.65)),
                    Node {
                        min_width: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    LayerCounterText { category },
                ));

                spawn_arrow(controls, font_regular, category, ArrowDirection::Next, "▶");
            });
        });
}

fn spawn_arrow(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    category: LayerCategory,
    direction: ArrowDirection,
    label: &str,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(ARROW_BG),
            BorderColor::all(ARROW_BORDER),
            BorderRadius::all(Val::Px(3.0)),
            LayerArrowButton { category, direction },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(GOLD),
            ));
        });
}

// ─── Future items (greyed) ──────────────────────────────────────────────────

fn spawn_future_item(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    label: &str,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Node {
                    width: Val::Px(26.0),
                    height: Val::Px(26.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.79, 0.66, 0.30, 0.03)),
                BorderColor::all(Color::srgba(0.55, 0.45, 0.30, 0.15)),
                BorderRadius::all(Val::Px(3.0)),
            ))
            .with_children(|icon_box| {
                icon_box.spawn((
                    Text::new("—"),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.42, 0.30, 0.19, 0.3)),
                ));
            });

            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.42, 0.30, 0.19, 0.4)),
            ));
        });
}

// ─── Action button ──────────────────────────────────────────────────────────

fn spawn_action_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    label: &str,
    bg: Color,
    text_color: Color,
    border_color: Color,
    marker: impl Component,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(42.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(bg),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(3.0)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}
