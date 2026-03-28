use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::resources::SceneRenderTarget;
use crate::states::GameView;
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::systems::panels::components::SettingsPanel;

pub fn setup_settings_panel(
    mut commands: Commands,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    render_target: Res<SceneRenderTarget>,
) {
    let config = FrostedGlassConfig::dialog()
        .with_border_radius(8.0)
        .with_colors(Color::srgb_u8(220, 202, 169), Color::srgb_u8(235, 225, 209));

    let mut material = FrostedGlassMaterial::from(config);

    // Inject the live scene texture
    material.scene_texture = Some(render_target.0.clone());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
            DespawnOnExit(GameView::Settings),
            SettingsPanel,
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
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    MaterialNode(materials.add(material)),
                    BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        });
}
