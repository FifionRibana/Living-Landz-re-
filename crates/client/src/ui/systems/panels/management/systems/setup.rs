use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::resources::SceneRenderTarget;
use crate::state::resources::PlayerInfo;
use crate::states::GameView;
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::systems::panels::components::ManagementPanel;

const GOLD: Color = Color::srgb(0.79, 0.66, 0.30);
const TEXT_LIGHT: Color = Color::srgb(0.92, 0.88, 0.80);
const TEXT_DIM: Color = Color::srgb(0.60, 0.52, 0.40);
const TEXT_DARK: Color = Color::srgb(0.20, 0.15, 0.10);

pub fn setup_management_panel(
    mut commands: Commands,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    render_target: Res<SceneRenderTarget>,
    asset_server: Res<AssetServer>,
    player_info: Res<PlayerInfo>,
) {
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_regular = asset_server.load("fonts/FiraSans-Regular.ttf");

    let glass_material = materials.add(FrostedGlassMaterial::from(
        FrostedGlassConfig::dialog()
            .with_border_radius(12.0)
            .with_colors(
                Color::srgb_u8(220, 202, 169),
                Color::srgb_u8(235, 225, 209),
            )
            .with_scene_texture(render_target.0.clone()),
    ));

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
            DespawnOnExit(GameView::CityManagement),
            ManagementPanel,
        ))
        .with_children(|root| {
            // Frosted glass panel
            root.spawn((
                MaterialNode(glass_material),
                Node {
                    width: Val::Px(600.0),
                    min_height: Val::Px(400.0),
                    max_height: Val::Px(600.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(32.0)),
                    row_gap: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            ))
            .with_children(|panel| {
                match &player_info.organization {
                    Some(org) => {
                        spawn_org_content(panel, org, &font_bold, &font_regular, &asset_server);
                    }
                    None => {
                        spawn_no_org_content(panel, &font_bold, &font_regular);
                    }
                }
            });
        });
}

fn spawn_org_content(
    panel: &mut RelatedSpawnerCommands<ChildOf>,
    org: &shared::OrganizationSummary,
    font_bold: &Handle<Font>,
    font_regular: &Handle<Font>,
    asset_server: &Res<AssetServer>,
) {
    // Header row: shield icon + name
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|header| {
            header.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ImageNode {
                    image: asset_server.load("ui/icons/griffin-shield.png"),
                    image_mode: NodeImageMode::Auto,
                    color: GOLD,
                    ..default()
                },
            ));

            header.spawn((
                Text::new(&org.name),
                TextFont {
                    font: font_bold.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_DARK),
            ));
        });

    // Type
    panel.spawn((
        Text::new(format!("{:?}", org.organization_type)),
        TextFont {
            font: font_regular.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(TEXT_DIM),
    ));

    // Separator
    panel.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.55, 0.45, 0.30, 0.3)),
    ));

    // Stats
    let stats = [
        ("Population", format!("{} / {} (logements)", org.population, "?")),
        ("Type", format!("{:?}", org.organization_type)),
    ];

    for (label, value) in &stats {
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(*label),
                    TextFont {
                        font: font_regular.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                ));
                row.spawn((
                    Text::new(value.as_str()),
                    TextFont {
                        font: font_bold.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(TEXT_DARK),
                ));
            });
    }

    // Placeholder for future: buildings list, treasury, etc.
    panel.spawn((
        Node {
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },
    ))
    .with_children(|section| {
        section.spawn((
            Text::new("Bâtiments et gestion à venir..."),
            TextFont {
                font: font_regular.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.5, 0.4, 0.3, 0.5)),
        ));
    });
}

fn spawn_no_org_content(
    panel: &mut RelatedSpawnerCommands<ChildOf>,
    font_bold: &Handle<Font>,
    font_regular: &Handle<Font>,
) {
    panel.spawn((
        Text::new("Aucune organisation"),
        TextFont {
            font: font_bold.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(TEXT_DARK),
    ));

    panel.spawn((
        Text::new("Fondez un hameau pour commencer !\nSélectionnez votre Lord, puis clic droit → Fonder un hameau."),
        TextFont {
            font: font_regular.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(TEXT_DIM),
    ));
}