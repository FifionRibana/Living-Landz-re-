use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::resources::SceneRenderTarget;
use crate::states::GameView;
use crate::ui::carousel::components::{Carousel, CarouselAlpha, CarouselItem};
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::systems::panels::components::MessagesPanel;

pub fn setup_messages_panel(
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
            DespawnOnExit(GameView::Messages),
            MessagesPanel,
        ))
        .with_children(|parent| {
            let items = vec![
                "Item 1", "Item 2", "Item 3", "Item 4", "Item 5", "Item 6", "Item 7",
            ];
            let item_width = 150.0;
            let spacing = 25.0;

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(250.),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::horizontal(Val::Px(50.0)),
                        ..default()
                    },
                    Carousel {
                        scroll_offset: 0.0,
                        item_width,
                        spacing,
                        total_items: items.len(),
                        current_scroll: 0.0,
                        target_scroll: 0.0,
                        lerp_speed: 10.0,
                        snap_timer: 0.0
                    },
                ))
                .with_children(|carousel| {
                    for (i, name) in items.iter().enumerate() {
                        carousel
                            .spawn((
                                CarouselItem { index: i },
                                MaterialNode(
                                    materials.add(FrostedGlassMaterial::from(
                                        FrostedGlassConfig::card()
                                            .with_border_radius(12.0)
                                            .with_scene_texture(render_target.0.clone()),
                                    )),
                                ),
                                Node {
                                    width: Val::Px(item_width),
                                    height: Val::Px(200.0),
                                    ..default()
                                },
                                BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
                                BorderRadius::all(Val::Px(12.0)),
                            ))
                            .with_children(|carousel_item| {
                                carousel_item.spawn((
                                    Text::new(*name),
                                    BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                                    CarouselAlpha::new(1.0),
                                ));
                            });
                    }
                });
            // parent
            //     .spawn(Node {
            //         width: Val::Percent(100.0),
            //         height: Val::Px(200.0),
            //         flex_direction: FlexDirection::Row,
            //         align_items: AlignItems::Center,
            //         justify_content: JustifyContent::Center,
            //         column_gap: Val::Px(20.0),
            //         ..default()
            //     })
            //     .with_children(|container| {
            //         // 1. Left card
            //         container.spawn((
            //             MaterialNode(
            //                 materials.add(FrostedGlassMaterial::from(
            //                     FrostedGlassConfig::card_fading(FadeDirection::Left)
            //                         .with_border_radius(12.0)
            //                         .with_scene_texture(render_target.0.clone()),
            //                 )),
            //             ),
            //             Node {
            //                 width: Val::Px(150.0),
            //                 height: Val::Px(180.0),
            //                 ..default()
            //             },
            //             BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            //             BorderRadius::all(Val::Px(12.0)),
            //         ));

            //         // 2. Mid cards
            //         for _ in 0..3 {
            //             container.spawn((
            //                 MaterialNode(
            //                     materials.add(FrostedGlassMaterial::from(
            //                         FrostedGlassConfig::card()
            //                             .with_border_radius(12.0)
            //                             .with_scene_texture(render_target.0.clone()),
            //                     )),
            //                 ),
            //                 Node {
            //                     width: Val::Px(150.0),
            //                     height: Val::Px(200.0),
            //                     ..default()
            //                 },
            //                 BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            //                 BorderRadius::all(Val::Px(12.0)),
            //             ));
            //         }

            //         // 3. Right card
            //         container.spawn((
            //             MaterialNode(
            //                 materials.add(FrostedGlassMaterial::from(
            //                     FrostedGlassConfig::card_fading(FadeDirection::Right)
            //                         .with_border_radius(12.0)
            //                         .with_scene_texture(render_target.0.clone()),
            //                 )),
            //             ),
            //             Node {
            //                 width: Val::Px(150.0),
            //                 height: Val::Px(180.0),
            //                 ..default()
            //             },
            //             BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            //             BorderRadius::all(Val::Px(12.0)),
            //         ));
            //     });

            // Login form container (centered box)
            // parent
            //     .spawn((
            //         Node {
            //             width: Val::Px(450.0),
            //             padding: UiRect::all(Val::Px(40.0)),
            //             flex_direction: FlexDirection::Column,
            //             row_gap: Val::Px(20.0),
            //             border: UiRect::all(Val::Px(2.0)),
            //             ..default()
            //         },
            //         MaterialNode(materials.add(material)),
            //         BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
            //         BorderRadius::all(Val::Px(8.0)),
            //     ))
            //     .with_children(|panel| {
            //         panel.spawn((
            //             Text::new("MESSAGES"),
            //             TextFont {
            //                 font_size: 28.0,
            //                 ..default()
            //             },
            //             TextColor(Color::BLACK),
            //         ));
            //     });
        });
}
