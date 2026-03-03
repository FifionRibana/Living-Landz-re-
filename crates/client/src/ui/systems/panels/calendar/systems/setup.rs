use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::states::GameView;
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::systems::panels::components::CalendarPanel;

pub fn setup_calendar_panel(
    mut commands: Commands,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
) {
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
            DespawnOnExit(GameView::Calendar),
            CalendarPanel,
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
                                ),
                            // .with_colors(
                            //     Color::srgba(1.0, 1.0, 1.0, 1.0),    // Blanc en haut
                            //     Color::srgba(0.92, 0.88, 0.82, 1.0), // Beige en bas
                            // )
                            // .with_background(background_image),
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
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("CALENDAR"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        });
}
