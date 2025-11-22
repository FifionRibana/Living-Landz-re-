use std::io::Cursor;

use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy_ui_text_input::{TextInputBuffer, TextInputNode, TextInputQueue, TextInputStyle};
use shared::atlas::{GaugeAtlas, MoonAtlas};

use crate::{
    grid::{components::HexSelectIndicator, resources::SelectedHexes},
    ui::{
        components::{
            ActionButtonMarker, ActionDescriptionText, ActionTitleText, ActionsPanelMarker,
            CharacterNameText, ChatIconButton, ChatInputContainer, ChatInputField,
            ChatMessagesContainer, ChatNotificationBadge, ChatPanelMarker, ChatSendButton,
            ChatToggleButton, ClockText, DateText, MenuButton, MoonPhaseImage, MoonText,
            PlayerNameText,
        },
        debug::HoveredCellInfoText,
        resources::ChatState,
    },
};

#[derive(Component)]
pub struct UIFrameMarker;

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gauge_atlas: Res<GaugeAtlas>,
    moon_atlas: Res<MoonAtlas>,
) {
    // let frame = asset_server.load("ui/wood_and_leather_frame_4_05x.png");
    let top_bar_image = asset_server.load("ui/ui_top_bar.png");
    let paper_panel_image = asset_server.load("ui/ui_paper_panel.png");
    let wood_panel_image = asset_server.load("ui/ui_wood_panel.png");

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
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

    let wood_panel_slicer = TextureSlicer {
        border: BorderRect {
            left: 42.,
            right: 42.,
            top: 42.,
            bottom: 42.,
        },
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    };

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
                    Button,
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
                        is_hoverable: true,
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
                    top_bar_parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center, // horizontal
                                align_items: AlignItems::Center,         // vertical
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
                    // Moon phase display with UI overlay
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
                    Interaction::None,
                    Pickable {
                        should_block_lower: true,
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
                                should_block_lower: true,
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
                                    should_block_lower: true,
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
                                should_block_lower: true,
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
                                    should_block_lower: true,
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
                                    should_block_lower: true,
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
                                        should_block_lower: true,
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
                                                should_block_lower: true,
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
                                    should_block_lower: true,
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
                                    should_block_lower: true,
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
                                    should_block_lower: true,
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
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });
                });

            // Actions panel at the bottom
            parent
                .spawn((
                    ImageNode {
                        image: wood_panel_image.clone(),
                        image_mode: NodeImageMode::Sliced(wood_panel_slicer.clone()),
                        ..default()
                    },
                    Node {
                        height: px(150.),
                        position_type: PositionType::Absolute,
                        bottom: px(0.0),
                        left: px(0.0),
                        right: px(0.0),
                        margin: UiRect::all(px(10.)),
                        padding: UiRect::all(px(16.)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ActionsPanelMarker,
                    UIFrameMarker,
                    Interaction::None,
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|actions_panel| {
                    // Title
                    actions_panel.spawn((
                        Text::new("Actions"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        Node {
                            margin: UiRect::bottom(px(8.)),
                            ..default()
                        },
                        ActionTitleText,
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));

                    // Actions container (horizontal scroll)
                    actions_panel
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(100.),
                                flex_direction: FlexDirection::Row,
                                column_gap: px(8.),
                                ..default()
                            },
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|actions_container| {
                            // Example action button: Build Road
                            actions_container
                                .spawn((
                                    Button,
                                    Node {
                                        width: px(100.),
                                        height: px(80.),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(px(8.)),
                                        border: UiRect::all(px(2.)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgb_u8(150, 130, 100)),
                                    BackgroundColor(Color::srgb_u8(100, 90, 70)),
                                    ActionButtonMarker {
                                        action_type: "build_road".to_string(),
                                    },
                                    Pickable {
                                        should_block_lower: true,
                                        is_hoverable: true,
                                    },
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new("Build\nRoad"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(223, 210, 194)),
                                        // TextLayout::new_with_justify(Justify::Center),
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: false,
                                        },
                                    ));
                                });

                            // Example action button: Build Building
                            actions_container
                                .spawn((
                                    Button,
                                    Node {
                                        width: px(100.),
                                        height: px(80.),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(px(8.)),
                                        border: UiRect::all(px(2.)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgb_u8(150, 130, 100)),
                                    BackgroundColor(Color::srgb_u8(100, 90, 70)),
                                    ActionButtonMarker {
                                        action_type: "build_building".to_string(),
                                    },
                                    Pickable {
                                        should_block_lower: true,
                                        is_hoverable: true,
                                    },
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new("Build\nBuilding"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(223, 210, 194)),
                                        // TextLayout::new_with_justify(Justify::Center),
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: false,
                                        },
                                    ));
                                });
                        });
                });

            // Chat panel at the bottom left
            parent
                .spawn((
                    ImageNode {
                        image: paper_panel_image.clone(),
                        image_mode: NodeImageMode::Sliced(paper_panel_slicer.clone()),
                        ..default()
                    },
                    Node {
                        width: px(350.),
                        height: px(250.),
                        position_type: PositionType::Absolute,
                        bottom: px(0.0),
                        left: px(0.0),
                        margin: UiRect::all(px(10.)),
                        padding: UiRect {
                            left: px(16.),
                            right: px(16.),
                            top: px(4.),
                            bottom: px(16.),
                        },
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ChatPanelMarker,
                    Interaction::None,
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|chat_panel| {
                    // Title bar with toggle button
                    chat_panel
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: px(36.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|title_bar| {
                            // Title
                            title_bar.spawn((
                                Text::new("Chat"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });

                    // Toggle button
                    chat_panel
                        .spawn((
                            Button,
                            Node {
                                width: px(20.),
                                height: px(20.),
                                top: px(6.),
                                right: px(12.),
                                position_type: PositionType::Absolute,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(1.)),
                                ..default()
                            },
                            BorderColor::all(Color::srgb_u8(150, 130, 100)),
                            BackgroundColor(Color::srgb_u8(100, 90, 70)),
                            ChatToggleButton,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: true,
                            },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("-"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });

                    // Messages container (scrollable area)
                    chat_panel
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(100.),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip_y(),
                                margin: UiRect::bottom(px(8.)),
                                ..default()
                            },
                            ChatMessagesContainer,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|messages| {
                            // Example messages
                            messages.spawn((
                                Text::new("Player1: Hello!"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Node {
                                    margin: UiRect::bottom(px(4.)),
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                            messages.spawn((
                                Text::new("Player2: Hi there!"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Node {
                                    margin: UiRect::bottom(px(4.)),
                                    ..default()
                                },
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });

                    // Input container
                    chat_panel
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: px(32.),
                                flex_direction: FlexDirection::Row,
                                column_gap: px(8.),
                                ..default()
                            },
                            ChatInputContainer,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|input_container| {
                            // Input field with bevy_ui_text_input
                            input_container.spawn((
                                TextInputNode {
                                    clear_on_submit: true,
                                    ..default()
                                },
                                TextInputBuffer::default(),
                                TextInputQueue::default(),
                                TextInputStyle::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(223, 210, 194)),
                                Node {
                                    flex_grow: 1.,
                                    height: px(32.),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(px(8.)),
                                    ..default()
                                },
                                BorderColor::all(Color::srgb_u8(150, 130, 100)),
                                BackgroundColor(Color::srgb_u8(80, 70, 50)),
                                ChatInputField,
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: true,
                                },
                            ));

                            // Send button
                            input_container
                                .spawn((
                                    Button,
                                    Node {
                                        width: px(60.),
                                        height: px(32.),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(px(8.)),
                                        border: UiRect::all(px(2.)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgb_u8(150, 130, 100)),
                                    BackgroundColor(Color::srgb_u8(100, 90, 70)),
                                    ChatSendButton,
                                    Pickable {
                                        should_block_lower: true,
                                        is_hoverable: true,
                                    },
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new("Send"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(223, 210, 194)),
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: false,
                                        },
                                    ));
                                });
                        });
                });

            // Chat icon button (when chat is collapsed)
            parent
                .spawn((
                    Button,
                    Node {
                        width: px(48.),
                        height: px(48.),
                        position_type: PositionType::Absolute,
                        bottom: px(10.),
                        left: px(10.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    BorderColor::all(Color::srgb_u8(150, 130, 100)),
                    BorderRadius::all(px(24.)),
                    ChatIconButton,
                    Visibility::Hidden, // Initially hidden, shown when chat is collapsed
                    Interaction::None,
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: true,
                    },
                ))
                .with_children(|icon_button| {
                    // Chat icon
                    icon_button.spawn((
                        ImageNode {
                            image: asset_server.load("ui/icons/envelope.png"),
                            ..default()
                        },
                        Node {
                            width: px(30.),
                            height: px(30.),
                            ..default()
                        },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));

                    // Notification badge
                    icon_button
                        .spawn((
                            Node {
                                width: px(20.),
                                height: px(20.),
                                position_type: PositionType::Absolute,
                                top: px(-4.),
                                right: px(-4.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                            BorderRadius::all(px(10.)),
                            ChatNotificationBadge,
                            Visibility::Hidden, // Only visible when there are unread messages
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new("0"),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Pickable {
                                    should_block_lower: true,
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
