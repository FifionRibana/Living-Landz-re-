/// Setup system for the register panel
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputNode, TextInputQueue, TextInputStyle};

use crate::ui::{
    components::PanelContainer, resources::PanelEnum, systems::panels::auth::components::*,
};

pub fn setup_register_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root container - full screen with background
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
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 1.0)), // Dark blue-grey background
            Visibility::Hidden,                                   // Hidden by default
            PanelContainer {
                panel: PanelEnum::RegisterPanel,
            },
            RegisterPanelMarker,
        ))
        .with_children(|parent| {
            // Register form container (centered box)
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
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.95)), // Semi-transparent dark panel
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.35)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|form| {
                    // Title
                    form.spawn((
                        Text::new("Créer votre maison"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.85, 0.75)),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
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
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
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
                        RegisterFamilyNameInput,
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

                    // Password input
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
                        RegisterPasswordInput,
                    ));

                    // Confirm Password label
                    form.spawn((
                        Text::new("Confirmer le mot de passe"),
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

                    // Confirm Password input
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
                        RegisterPasswordConfirmInput,
                    ));

                    // Password requirements text
                    form.spawn((
                        Text::new(
                            "Au moins 8 caractères, Une majuscule, Une minuscule, Un chiffre",
                        ),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        },
                        RegisterPasswordRequirementsText,
                    ));

                    // Error message (hidden by default)
                    form.spawn((
                        Text::new(""),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.3, 0.3)), // Red for errors
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                        RegisterErrorText,
                    ));

                    // Success message (hidden by default)
                    form.spawn((
                        Text::new(""),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.9, 0.3)), // Green for success
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                        RegisterSuccessText,
                    ));

                    // Register button
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
                        RegisterSubmitButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Créer le compte"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        ));
                    });

                    // Back button
                    form.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.18, 0.8)),
                        BorderRadius::all(Val::Px(4.0)),
                        RegisterBackButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Retour"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.8, 0.9)),
                        ));
                    });
                });
        });
}
