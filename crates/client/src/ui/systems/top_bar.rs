use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use shared::atlas::MoonAtlas;

use crate::ui::{
    components::{
        ActionMenuMarker, ActionModeMenuButton, ActionModeMenuIcon, CharacterNameText, ClockText,
        DateText, MenuButton, MoonPhaseImage, PlayerNameText, TopBarMarker,
    },
    resources::{ActionModeEnum, PanelEnum, UIState},
    systems::{CLICK_COLOR, HOVER_COLOR, NORMAL_COLOR},
};

const TEXT_LIGHT_PRIMARY: Color = Color::srgb_u8(243, 217, 175);
const TEXT_LIGHT_SECONDARY: Color = Color::srgb_u8(152, 121, 94);

pub fn setup_top_bar(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &Res<AssetServer>,
    moon_atlas: &Res<MoonAtlas>,
) {
    let top_bar_image = asset_server.load("ui/ui_top_bar_3.png");
    let top_bar_slicer = TextureSlicer {
        border: BorderRect {
            left: 24.,
            right: 24.,
            top: 19.,
            bottom: 49.,
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
    let world_map_image = asset_server.load("ui/icons/compass.png");
    let village_image = asset_server.load("ui/icons/village.png");

    let sub_tab_normal_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_normal.png");
    let sub_tab_hovered_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_hovered.png");
    let sub_tab_selected_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_selected.png");
    let sub_tab_disabled_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_disabled.png");

    let road_image = asset_server.load("ui/icons/stone-path.png");
    let training_image = asset_server.load("ui/icons/graduate-cap.png");
    let diplomacy_image = asset_server.load("ui/icons/shaking-hands.png");

    let menu_images = [
        (world_map_image, PanelEnum::MapView),
        (griffin_shield_image, PanelEnum::ManagementPanel),
        (bookmarklet_image, PanelEnum::RecordsPanel),
        (envelope_image, PanelEnum::MessagesPanel),
        (laurels_trophy_image, PanelEnum::RankingPanel),
        (calendar_image, PanelEnum::CalendarPanel),
        (search_image, PanelEnum::SearchView),
        (cog_image.clone(), PanelEnum::SettingsView),
    ];

    let sub_menu_images = [
        (road_image, ActionModeEnum::RoadActionMode),
        (village_image, ActionModeEnum::BuildingActionMode),
        (cog_image, ActionModeEnum::ProductionActionMode),
        (training_image, ActionModeEnum::TrainingActionMode),
        (diplomacy_image, ActionModeEnum::DiplomacyActionMode),
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
                height: px(91.),
                position_type: PositionType::Absolute,
                top: px(0.0),
                left: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            TopBarMarker,
            GlobalZIndex(1000),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|top_bar_parent| {
            top_bar_parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(61.),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: true,
                        is_hoverable: false,
                    },
                ))
                .with_children(|top_bar_content| {
                    // Menu buttons
                    top_bar_content
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
                            for (index, (menu_image, panel)) in menu_images.iter().enumerate() {
                                menu_bar_parent
                                    .spawn((
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
                                        MenuButton {
                                            button_id: index,
                                            panel: *panel,
                                        },
                                    ))
                                    .observe(recolor_menu_button_on::<Pointer<Over>>(HOVER_COLOR))
                                    .observe(recolor_menu_button_on::<Pointer<Out>>(NORMAL_COLOR))
                                    .observe(recolor_menu_button_on::<Pointer<Click>>(CLICK_COLOR))
                                    .observe(on_menu_button_click);
                            }
                        });

                    // Spacer
                    top_bar_content.spawn((
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
                    top_bar_content
                        .spawn((
                            Node {
                                width: Val::Px(246.),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|middle_content| {
                            middle_content.spawn(Node {
                                width: Val::Px(108.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            });
                            middle_content
                                .spawn((
                                    Node {
                                        flex_grow: 1.,
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
                                        TextColor(TEXT_LIGHT_PRIMARY),
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
                                        TextColor(TEXT_LIGHT_PRIMARY),
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
                                        TextColor(TEXT_LIGHT_PRIMARY),
                                        ClockText,
                                        Node { ..default() },
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: false,
                                        },
                                    ));
                                });
                        });

                    // Spacer
                    top_bar_content.spawn((
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
                    top_bar_content
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
                    top_bar_content.spawn((
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
                    top_bar_content
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
                                TextColor(TEXT_LIGHT_PRIMARY),
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
                                TextColor(TEXT_LIGHT_SECONDARY),
                                CharacterNameText,
                                Node { ..default() },
                                Pickable {
                                    should_block_lower: true,
                                    is_hoverable: false,
                                },
                            ));
                        });
                });
        });

    parent
        .spawn((
            Node {
                width: percent(100),
                height: Val::Px(44.),
                position_type: PositionType::Absolute,
                top: Val::Px(60.),
                left: Val::Px(20.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            GlobalZIndex(1001),
            ActionMenuMarker,
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|sub_bar_parent| {
            for (index, (sub_menu_image, action_mode)) in sub_menu_images.iter().enumerate() {
                sub_bar_parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(48.),
                            height: Val::Px(44.),
                            margin: UiRect::horizontal(Val::Px(2.)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ImageNode {
                            image: if index == 0 {
                                sub_tab_selected_image.clone()
                            } else {
                                sub_tab_normal_image.clone()
                            },
                            image_mode: NodeImageMode::Auto,
                            ..default()
                        },
                        ActionModeMenuButton {
                            action_mode: *action_mode,
                        },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: true,
                        },
                    ))
                    .observe(on_action_button_hovered::<Pointer<Over>>(true))
                    .observe(on_action_button_hovered::<Pointer<Out>>(false))
                    .observe(on_action_menu_button_click)
                    .with_children(|button_parent| {
                        button_parent.spawn((
                            Node {
                                width: Val::Px(28.),
                                height: Val::Px(28.),
                                ..default()
                            },
                            ImageNode {
                                image: sub_menu_image.clone(),
                                image_mode: NodeImageMode::Auto,
                                color: Color::srgb_u8(157, 136, 93),
                                ..default()
                            },
                            ActionModeMenuIcon {
                                action_mode: *action_mode,
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                    });
            }
        });
}

pub fn recolor_menu_button_on<E: EntityEvent>(
    color: Color,
) -> impl Fn(On<E>, Query<(&MenuButton, &mut ImageNode)>) {
    move |event, mut menu_button_query| {
        if let Ok((_, mut image_node)) = menu_button_query.get_mut(event.event_target()) {
            image_node.color = color;
        }
    }
}

pub fn on_action_button_hovered<E: EntityEvent>(
    state: bool,
) -> impl Fn(On<E>, ResMut<UIState>, Query<&ActionModeMenuButton>) {
    move |event, mut ui_state, action_button_query| {
        if let Ok(action_button) = action_button_query.get(event.event_target()) {
            ui_state.set_action_mode_hovered(action_button.action_mode, state);
        }
    }
}

pub fn on_menu_button_click(
    event: On<Pointer<Click>>,
    menu_button_query: Query<&MenuButton>,
    mut ui_state: ResMut<UIState>,
) {
    if let Ok(menu_button) = menu_button_query.get(event.entity) {
        info!("Switching to {:?}", menu_button.panel);
        ui_state.switch_to(menu_button.panel);
    }
}

pub fn on_action_menu_button_click(
    event: On<Pointer<Click>>,
    menu_button_query: Query<&ActionModeMenuButton>,
    mut ui_state: ResMut<UIState>,
) {
    if let Ok(menu_button) = menu_button_query.get(event.entity) {
        if Some(menu_button.action_mode) == ui_state.action_mode {
            info!("Reset action mode");
            ui_state.reset_action_mode();
        } else {
            info!("Set action mode to: {:?}", menu_button.action_mode);
            ui_state.set_action_mode(menu_button.action_mode);
        }
    }
}
