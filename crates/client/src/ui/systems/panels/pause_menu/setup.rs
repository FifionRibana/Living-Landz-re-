use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::states::Overlay;

// ─── Component markers ──────────────────────────────────────────────────────

#[derive(Component)]
pub struct PauseMenuPanel;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct DisconnectButton;

// ─── Colors ─────────────────────────────────────────────────────────────────

const PANEL_BG: Color = Color::srgba(0.18, 0.14, 0.10, 0.95);
const BUTTON_BG: Color = Color::srgb(0.30, 0.24, 0.18);
const BUTTON_HOVER: Color = Color::srgb(0.40, 0.32, 0.22);
const BUTTON_BORDER: Color = Color::srgb(0.55, 0.45, 0.30);
const TEXT_LIGHT: Color = Color::srgb(0.92, 0.88, 0.80);
const TEXT_TITLE: Color = Color::srgb(0.95, 0.90, 0.78);
const DISCONNECT_BG: Color = Color::srgb(0.45, 0.18, 0.15);
const DISCONNECT_HOVER: Color = Color::srgb(0.55, 0.22, 0.18);

// ─── Setup ──────────────────────────────────────────────────────────────────

pub fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_regular = asset_server.load("fonts/FiraSans-Regular.ttf");

    // Full-screen darkened overlay
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(100),
            DespawnOnExit(Overlay::PauseMenu),
            PauseMenuPanel,
        ))
        .with_children(|overlay| {
            // Center panel
            overlay
                .spawn((
                    Node {
                        width: Val::Px(360.0),
                        padding: UiRect::all(Val::Px(32.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(16.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(BUTTON_BORDER),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Pause"),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_TITLE),
                        Node {
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        },
                    ));

                    // Resume button
                    spawn_button(
                        panel,
                        "Reprendre",
                        &font_regular,
                        BUTTON_BG,
                        ResumeButton,
                    );

                    // Separator
                    panel.spawn((
                        Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(1.0),
                            margin: UiRect::axes(Val::Px(0.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                    ));

                    // Disconnect button
                    spawn_button(
                        panel,
                        "Se déconnecter",
                        &font_regular,
                        DISCONNECT_BG,
                        DisconnectButton,
                    );
                });
        });
}

fn spawn_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    font: &Handle<Font>,
    bg_color: Color,
    marker: impl Component,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(bg_color),
            BorderColor::all(BUTTON_BORDER),
            BorderRadius::all(Val::Px(4.0)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_LIGHT),
            ));
        });
}

/// Visual hover feedback for pause menu buttons.
pub fn update_pause_button_hover(
    mut resume_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResumeButton>),
    >,
    mut disconnect_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DisconnectButton>, Without<ResumeButton>),
    >,
) {
    for (interaction, mut bg) in resume_query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BUTTON_HOVER),
            Interaction::None => BackgroundColor(BUTTON_BG),
        };
    }
    for (interaction, mut bg) in disconnect_query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(DISCONNECT_HOVER),
            Interaction::None => BackgroundColor(DISCONNECT_BG),
        };
    }
}
