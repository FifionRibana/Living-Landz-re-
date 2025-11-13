use std::io::Cursor;

use bevy::{asset::RenderAssetUsages, prelude::*};

use crate::{
    grid::{components::HexSelectIndicator, resources::SelectedHexes},
    ui::debug::HoveredCellInfoText,
};

#[derive(Component)]
pub struct UIFrameMarker;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let frame = asset_server.load("ui/wood_and_leather_frame_4_05x.png");

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

    let slicer = TextureSlicer {
        border: BorderRect {
            left: 42.,
            right: 42.,
            top: 41.,
            bottom: 41.,
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
            UIFrameMarker,
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageNode {
                        image: frame.clone(),
                        // image: frame_handler.clone(),
                        image_mode: NodeImageMode::Sliced(slicer.clone()),
                        ..default()
                    },
                    Node {
                        width: px(400.),
                        height: px(600.),
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        right: Val::Px(20.0),
                        margin: UiRect::all(px(20.)),
                        ..default()
                    },
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
                                height: percent(100.),
                                position_type: PositionType::Absolute,
                                margin: UiRect::all(px(20.)),
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
                                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                Node {
                                    ..default()
                                },
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
                                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                Node {
                                    ..default()
                                },
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
                                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                Node {
                                    ..default()
                                },
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
                                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                Node {
                                    ..default()
                                },
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
                                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                Node {
                                    ..default()
                                },
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
