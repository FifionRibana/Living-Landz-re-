use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use shared::atlas::GaugeAtlas;

use crate::ui::{
    components::{
        CellDetailsActionStatusText, CellDetailsActionTypeText, CellDetailsBiomeText, CellDetailsBuildingImage, CellDetailsPanelMarker, CellDetailsQualityGaugeContainer, CellDetailsTitleText
    },
    debug::HoveredCellInfoText,
};

pub fn setup_cell_details_panel(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &Res<AssetServer>,
    gauge_atlas: &Res<GaugeAtlas>,
) {
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

    let blacksmith_image = asset_server.load("sprites/buildings/blacksmith_01.png");

    parent
        .spawn((
            ImageNode {
                image: paper_panel_image.clone(),
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
            CellDetailsPanelMarker,
            Interaction::None,
            Visibility::Hidden,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|background_parent| {
            // Title
            background_parent
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
                .with_children(|frame_parent| {
                    frame_parent.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(223, 210, 194)),
                        CellDetailsTitleText,
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: false,
                        },
                    ));
                });

            // Content
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
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(67, 60, 37)),
                        Node { ..default() },
                        CellDetailsBiomeText,
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
                        CellDetailsBuildingImage,
                        // Visibility::Inherited,
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
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            CellDetailsQualityGaugeContainer,
                            // Visibility::Inherited,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: false,
                            },
                        ))
                        .with_children(|quality_parent| {
                            for position in -4i32..=4i32 {
                                let light = match position.unsigned_abs() {
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
                                            .get_handles(position.unsigned_abs(), light)
                                            .expect("")
                                            .clone(),
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
                        CellDetailsActionStatusText,
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
                        CellDetailsActionTypeText,
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
}
