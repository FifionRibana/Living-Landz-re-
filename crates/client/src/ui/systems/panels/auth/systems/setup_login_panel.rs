/// Setup system for the login panel
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputNode, TextInputQueue, TextInputStyle};

use crate::ui::{
    frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial},
    systems::panels::auth::components::*,
};
use crate::states::AuthScreen;

pub fn setup_login_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
) {
    // Root container - full screen with background
    let background_image = asset_server.load("ui/backgrounds/login.jpg");
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
            // BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 1.0)), // Dark blue-grey background
            ImageNode {
                image: background_image.clone(),
                ..default()
            },
            Visibility::Visible, // Visible by default (will be hidden by panel_visibility system when not active)
            DespawnOnExit(AuthScreen::Login),
            LoginPanelMarker,
        ))
        .with_children(|parent| {
            // Login form container (centered box)
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        padding: UiRect::all(Val::Px(40.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    // BackgroundColor(Color::srgba_u8(235, 225, 209, 128)), // Semi-transparent dark panel
                    MaterialNode(
                        materials.add(FrostedGlassMaterial::from(
                            // FrostedGlassConfig::card_fading(FadeDirection::Left)
                            FrostedGlassConfig::dialog()
                                .with_border_radius(8.0)
                                .with_colors(
                                    Color::srgb_u8(220, 202, 169),
                                    Color::srgb_u8(235, 225, 209),
                                )
                                // .with_colors(
                                //     Color::srgba(1.0, 1.0, 1.0, 1.0),    // Blanc en haut
                                //     Color::srgba(0.92, 0.88, 0.82, 1.0), // Beige en bas
                                // )
                                .with_background(background_image),
                        )),
                    ),
                    // BackgroundGradient::from(LinearGradient {
                    //     angle: 0.,
                    //     stops: vec![
                    //         ColorStop::new(Color::srgba_u8(220, 202, 169, 255), Val::Percent(0.)),
                    //         ColorStop::new(Color::srgba_u8(235, 225, 209, 96), Val::Percent(100.)),
                    //     ],
                    //     ..default()
                    // }),
                    BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|form| {
                    // Title
                    form.spawn((
                        Text::new("Living Landz [re]"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(61, 34, 18)),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));

                    // Subtitle
                    form.spawn((
                        Text::new("Connexion"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(61, 34, 18)),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));

                    // Family Name label
                    form.spawn((
                        Text::new("Nom de famille"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(61, 34, 18)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                    ));

                    // Family Name input
                    form.spawn((
                        TextInputNode {
                            clear_on_submit: false,
                            ..default()
                        },
                        TextInputBuffer::default(),
                        TextInputQueue::default(),
                        TextInputStyle::default(),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
                        BorderRadius::all(Val::Px(4.0)),
                        LoginFamilyNameInput,
                    ));

                    // Password label
                    form.spawn((
                        Text::new("Mot de passe"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        Node {
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(5.0),
                                ..default()
                            },
                            ..default()
                        },
                    ));

                    // Password input (TODO: Implement masking)
                    form.spawn((
                        TextInputNode {
                            clear_on_submit: false,
                            ..default()
                        },
                        TextInputBuffer::default(),
                        TextInputQueue::default(),
                        TextInputStyle::default(),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(61, 34, 18)),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
                        BorderRadius::all(Val::Px(4.0)),
                        LoginPasswordInput,
                    ));

                    // Error message (hidden by default)
                    form.spawn((
                        Text::new(""),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(61, 34, 18)), // Red for errors
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                        LoginErrorText,
                    ));

                    // Login button
                    form.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(48.0),
                            margin: UiRect::top(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.5, 0.65, 0.45)),
                        BackgroundColor(Color::srgb(0.35, 0.5, 0.3)),
                        BorderRadius::all(Val::Px(4.0)),
                        LoginSubmitButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Se connecter"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb_u8(61, 34, 18)),
                        ));
                    });

                    // Divider
                    form.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(15.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.5)),
                    ));

                    // Register link
                    form.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.18, 0.8)),
                        BorderRadius::all(Val::Px(4.0)),
                        LoginToRegisterButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Créer un compte"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.8, 0.9)),
                        ));
                    });

                    // ─── Dev/test buttons ───────────────────────────────
                    form.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.3)),
                    ));

                    form.spawn((
                        Text::new("— Tests —"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.6, 0.55, 0.45, 0.6)),
                        Node {
                            align_self: AlignSelf::Center,
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                    ));

                    // Row with two test buttons
                    form.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        // Test character creation
                        row.spawn((
                            Button,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.6, 0.50, 0.30, 0.5)),
                            BackgroundColor(Color::srgba(0.25, 0.20, 0.12, 0.8)),
                            BorderRadius::all(Val::Px(3.0)),
                            TestCharacterCreationButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Personnage"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.80, 0.66, 0.30)),
                            ));
                        });

                        // Test coat of arms
                        row.spawn((
                            Button,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.6, 0.50, 0.30, 0.5)),
                            BackgroundColor(Color::srgba(0.25, 0.20, 0.12, 0.8)),
                            BorderRadius::all(Val::Px(3.0)),
                            TestCoatOfArmsButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Blason"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.80, 0.66, 0.30)),
                            ));
                        });
                    });
                });
        });
}