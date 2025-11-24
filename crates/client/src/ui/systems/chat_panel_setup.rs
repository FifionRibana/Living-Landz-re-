use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_ui_text_input::{TextInputBuffer, TextInputNode, TextInputQueue, TextInputStyle};

use crate::ui::components::{
    ChatIconButton, ChatInputContainer, ChatInputField, ChatMessagesContainer,
    ChatNotificationBadge, ChatPanelMarker, ChatSendButton, ChatToggleButton,
};

pub fn setup_chat_panel(parent: &mut RelatedSpawnerCommands<ChildOf>, asset_server: &Res<AssetServer>) {
    let paper_panel_image = asset_server.load("ui/ui_paper_panel.png");
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
}
