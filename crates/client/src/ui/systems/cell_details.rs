use std::io::Cursor;

use bevy::{asset::RenderAssetUsages, prelude::*};
use shared::atlas::GaugeAtlas;

use crate::{
    grid::{components::HexSelectIndicator, resources::SelectedHexes},
    ui::{
        components::{ClockText, DateText, MoonText},
        debug::HoveredCellInfoText,
    },
};

#[derive(Component)]
pub struct UIFrameMarker;

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gauge_atlas: Res<GaugeAtlas>,
) {
    // let frame = asset_server.load("ui/wood_and_leather_frame_4_05x.png");
    let top_bar_image = asset_server.load("ui/ui_top_bar.png");
    let paper_panel_image = asset_server.load("ui/ui_paper_panel.png");

    let blacksmith_image = asset_server.load("sprites/buildings/blacksmith_01.png");

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
    // let bytes = include_bytes!("../../../../../assets/ui/wooden_frame.png");
    // let img = image::ImageReader::new(Cursor::new(bytes))
    //     .unwrap()
    //     .decode()
    //     .unwrap();

    // let scaled = img.resize(
    //     img.width() / 2, // 50% de réduction
    //     img.height() / 2,
    //     image::imageops::FilterType::Lanczos3,
    // );

    // let bevy_image = Image::from_dynamic(scaled, true, RenderAssetUsages::RENDER_WORLD);
    // let frame_handle = images.add(bevy_image);

    // let slicer = TextureSlicer {
    //     border: BorderRect {
    //         left: 42.,
    //         right: 42.,
    //         top: 41.,
    //         bottom: 41.,
    //     },
    //     center_scale_mode: SliceScaleMode::Stretch {},
    //     sides_scale_mode: SliceScaleMode::Stretch {}, //Tile { stretch_value: 1.0 },
    //     max_corner_scale: 1.0,
    // };

    let paper_panel_slicer = TextureSlicer {
        border: BorderRect {
            left: 42.,
            right: 42.,
            top: 76.,
            bottom: 42.,
        },
        center_scale_mode: SliceScaleMode::Stretch {},
        sides_scale_mode: SliceScaleMode::Stretch {}, //Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    let top_bar_slicer = TextureSlicer {
        border: BorderRect {
            left: 24.,
            right: 24.,
            top: 24.,
            bottom: 24.,
        },
        center_scale_mode: SliceScaleMode::Stretch {},
        sides_scale_mode: SliceScaleMode::Stretch {}, //Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageNode {
                        image: top_bar_image.clone(),
                        // image: frame_handler.clone(),
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
                        // margin: UiRect::all(px(20.)),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|top_bar_parent| {
                    top_bar_parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(px(16.)),
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|menu_bar_parent| {
                            for menu_image in menu_images {
                                menu_bar_parent.spawn((
                                    Node {
                                        width: px(32.),
                                        height: px(32.),
                                        margin: UiRect::horizontal(px(8.)),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: menu_image,
                                        image_mode: NodeImageMode::Auto,
                                        color: Color::srgb_u8(157, 136, 93),
                                        ..default()
                                    },
                                    Pickable {
                                        should_block_lower: true,
                                        is_hoverable: true,
                                    },
                                ));
                            }
                        });
                    top_bar_parent.spawn((
                        Node {
                            flex_grow: 1.,
                            ..default()
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                    top_bar_parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center, // horizontal
                                align_items: AlignItems::Center,         // vertical
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
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
                                    should_block_lower: false,
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
                                    should_block_lower: false,
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
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                        });
                    top_bar_parent.spawn((
                        Node {
                            flex_grow: 1.,
                            ..default()
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                    top_bar_parent.spawn((
                        Text::new("--"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        MoonText,
                        Node { ..default() },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                    ));
                });
            parent
                .spawn((
                    ImageNode {
                        image: paper_panel_image.clone(),
                        // image: frame_handler.clone(),
                        image_mode: NodeImageMode::Sliced(paper_panel_slicer.clone()),
                        ..default()
                    },
                    Node {
                        width: px(200.),
                        height: px(400.),
                        position_type: PositionType::Absolute,
                        top: Val::Px(64.),
                        right: Val::Px(0.0),
                        margin: UiRect::all(px(10.)),
                        ..default()
                    },
                    UIFrameMarker,
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ))
                .with_children(|background_parent| {
                    background_parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: px(36.),
                                justify_content: JustifyContent::Center, // horizontal
                                align_items: AlignItems::Center,         // vertical
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|frame_parent| {
                            frame_parent.spawn((
                                Text::new("Blacksmith"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                        });
                    background_parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(100.),
                                position_type: PositionType::Absolute,
                                padding: UiRect {
                                    top: px(42.),
                                    bottom: px(24.),
                                    left: px(24.),
                                    right: px(24.),
                                },
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|frame_parent| {
                            frame_parent.spawn((
                                Text::new("Plain (deciduous forest)"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(67, 60, 37)),
                                Node { ..default() },
                                HoveredCellInfoText,
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                            frame_parent.spawn((
                                Node {
                                    width: px(96.),
                                    height: px(96.),
                                    align_self: AlignSelf::Center,
                                    ..default()
                                },
                                ImageNode {
                                    image: blacksmith_image,
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                            frame_parent
                                .spawn((
                                    Node {
                                        width: percent(100.),
                                        height: px(50.),
                                        align_items: AlignItems::Center, // vertical
                                        ..default()
                                    },
                                    Pickable {
                                        should_block_lower: false,
                                        is_hoverable: false,
                                    },
                                ))
                                .with_children(|quality_parent| {
                                    for position in -4i32..=4i32 {
                                        let light = match position.abs() as u32 {
                                            0 => 3,
                                            1 => 2,
                                            2 => 1,
                                            3.. => 0,
                                        };
                                        quality_parent.spawn((
                                            Node {
                                                margin: UiRect {
                                                    left: px(-6.),
                                                    right: px(-6.),
                                                    top: px(0.),
                                                    bottom: px(0.),
                                                },
                                                ..default()
                                            },
                                            ImageNode {
                                                image: gauge_atlas
                                                    .get_handles(position.abs() as u32, light)
                                                    .expect("")
                                                    .clone(),
                                                // image: frame_handler.clone(),
                                                image_mode: NodeImageMode::Auto,
                                                flip_x: position > 0,
                                                ..default()
                                            },
                                            ZIndex(3 - position.abs()),
                                            Pickable {
                                                should_block_lower: false,
                                                is_hoverable: false,
                                            },
                                        ));
                                    }
                                });
                            frame_parent.spawn((
                                Text::new("Plain (deciduous forest)"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(67, 60, 37)),
                                Node { ..default() },
                                HoveredCellInfoText,
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                            frame_parent.spawn((
                                Text::new("Plain (deciduous forest)"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(67, 60, 37)),
                                Node { ..default() },
                                HoveredCellInfoText,
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                            frame_parent.spawn((
                                Text::new("Plain (deciduous forest)"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(67, 60, 37)),
                                Node { ..default() },
                                HoveredCellInfoText,
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                            frame_parent.spawn((
                                Text::new("Plain (deciduous forest)"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(67, 60, 37)),
                                Node { ..default() },
                                HoveredCellInfoText,
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: false,
                                },
                            ));
                        });
                });
        });
}

pub fn update_ui(
    mut query: Query<&mut Visibility, With<UIFrameMarker>>,
    selected: Res<SelectedHexes>,
) {
    let is_selected = !selected.ids.is_empty();

    for mut visibility in query.iter_mut() {
        *visibility = if is_selected {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
