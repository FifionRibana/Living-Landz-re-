use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use shared::atlas::MoonAtlas;

use crate::ui::components::{
    ClockText, DateText, MenuButton, MoonPhaseImage, PlayerNameText, CharacterNameText, TopBarMarker,
};

pub fn setup_top_bar(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &Res<AssetServer>,
    moon_atlas: &Res<MoonAtlas>,
) {
    let top_bar_image = asset_server.load("ui/ui_top_bar.png");
    let top_bar_slicer = TextureSlicer {
        border: BorderRect {
            left: 24.,
            right: 24.,
            top: 24.,
            bottom: 24.,
        },
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    let bookmarklet_image = asset_server.load("ui/icons/bookmarklet.png");
    let calendar_image = asset_server.load("ui/icons/calendar.png");
    let cog_image = asset_server.load("ui/icons/cog.png");
    let envelope_image = asset_server.load("ui/icons/envelope.png");
    let griffin_shield_image = asset_server.load("ui/icons/griffin-shield.png");
    let laurels_trophy_image = asset_server.load("ui/icons/laurels-trophy.png");
    let search_image = asset_server.load("ui/icons/search.png");
    let village_image = asset_server.load("ui/icons/village.png");

    let menu_images = [
        village_image,
        griffin_shield_image,
        bookmarklet_image,
        envelope_image,
        laurels_trophy_image,
        calendar_image,
        search_image,
        cog_image,
    ];

    parent
        .spawn((
            Button,
            ImageNode {
                image: top_bar_image.clone(),
                image_mode: NodeImageMode::Sliced(top_bar_slicer.clone()),
                ..default()
            },
            Node {
                width: percent(100),
                height: px(64.),
                position_type: PositionType::Absolute,
                top: px(0.0),
                left: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            TopBarMarker,
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|top_bar_parent| {
            // Menu buttons
            top_bar_parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(16.)),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: true,
                    },
                ))
                .with_children(|menu_bar_parent| {
                    for (index, menu_image) in menu_images.iter().enumerate() {
                        menu_bar_parent.spawn((
                            Button,
                            Node {
                                width: px(32.),
                                height: px(32.),
                                margin: UiRect::horizontal(px(8.)),
                                ..default()
                            },
                            ImageNode {
                                image: menu_image.clone(),
                                image_mode: NodeImageMode::Auto,
                                color: Color::srgb_u8(157, 136, 93),
                                ..default()
                            },
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: true,
                            },
                            MenuButton { button_id: index },
                        ));
                    }
                });

            // Spacer
            top_bar_parent.spawn((
                Node {
                    flex_grow: 1.,
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
            ));

            // Date display
            top_bar_parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|date_node| {
                    date_node.spawn((
                        Text::new("Year 24 AF"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        Node { ..default() },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                    date_node.spawn((
                        Text::new("November 14th"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        DateText,
                        Node { ..default() },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                    date_node.spawn((
                        Text::new("13:37:23"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        ClockText,
                        Node { ..default() },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                });

            // Spacer
            top_bar_parent.spawn((
                Node {
                    flex_grow: 1.,
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
            ));

            // Moon phase display
            top_bar_parent
                .spawn((
                    Node {
                        width: px(48.),
                        height: px(48.),
                        align_self: AlignSelf::Center,
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|moon_container| {
                    // Moon phase image
                    if let Some(moon_image) = moon_atlas.get_handle(3) {
                        moon_container.spawn((
                            ImageNode {
                                image: moon_image.clone(),
                                ..default()
                            },
                            Node {
                                width: px(28.),
                                height: px(28.),
                                position_type: PositionType::Absolute,
                                left: px(10.),
                                top: px(10.),
                                ..default()
                            },
                            MoonPhaseImage,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ));
                    }
                    // UI overlay (moon ring)
                    moon_container.spawn((
                        ImageNode {
                            image: asset_server.load("ui/moon_ring_brass.png"),
                            ..default()
                        },
                        Node {
                            width: px(48.),
                            height: px(48.),
                            position_type: PositionType::Absolute,
                            left: px(0.),
                            top: px(0.),
                            ..default()
                        },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                });

            // Spacer
            top_bar_parent.spawn((
                Node {
                    width: px(32.),
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
            ));

            // Player and Character info
            top_bar_parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        padding: UiRect::horizontal(px(16.)),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|player_info_parent| {
                    player_info_parent.spawn((
                        Text::new("--"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        PlayerNameText,
                        Node { ..default() },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                    player_info_parent.spawn((
                        Text::new("--"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(200, 187, 171)),
                        CharacterNameText,
                        Node { ..default() },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                });
        });
}
