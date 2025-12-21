use bevy::prelude::*;
use crate::ui::components::{UnitDetailsPanelMarker, UnitDetailsNameText, UnitDetailsLevelText, UnitDetailsProfessionText, UnitDetailsCloseButton};
use crate::ui::resources::CellViewState;
use crate::state::resources::UnitsCache;

/// Marker component for the unit details panel container
#[derive(Component)]
pub struct UnitDetailsPanelContainer;

/// Spawn the unit details panel UI (initially hidden)
pub fn setup_unit_details_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    commands
        .spawn((
            ImageNode {
                image: paper_panel_image.clone(),
                image_mode: NodeImageMode::Sliced(paper_panel_slicer.clone()),
                ..default()
            },
            Node {
                width: Val::Px(300.0),
                height: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            UnitDetailsPanelMarker,
            UnitDetailsPanelContainer,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Close button
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                UnitDetailsCloseButton,
            ))
            .with_children(|button_parent| {
                button_parent.spawn((
                    Text::new("X"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Title
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ))
            .with_children(|title_parent| {
                title_parent.spawn((
                    Text::new("Unit Details"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(223, 210, 194)),
                ));
            });

            // Unit name
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ))
            .with_children(|field_parent| {
                field_parent.spawn((
                    Text::new("Name: "),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(67, 60, 37)),
                    UnitDetailsNameText,
                ));
            });

            // Unit level
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ))
            .with_children(|field_parent| {
                field_parent.spawn((
                    Text::new("Level: "),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(67, 60, 37)),
                    UnitDetailsLevelText,
                ));
            });

            // Unit profession
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ))
            .with_children(|field_parent| {
                field_parent.spawn((
                    Text::new("Profession: "),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(67, 60, 37)),
                    UnitDetailsProfessionText,
                ));
            });
        });
}

/// Update the visibility of the unit details panel based on selection state
pub fn update_panel_visibility(
    cell_view_state: Res<CellViewState>,
    mut panel_query: Query<&mut Visibility, With<UnitDetailsPanelMarker>>,
) {
    for mut visibility in &mut panel_query {
        if cell_view_state.is_active && cell_view_state.selected_unit.is_some() {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Update the content of the unit details panel
pub fn update_panel_content(
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    mut name_text_query: Query<&mut Text, (With<UnitDetailsNameText>, Without<UnitDetailsLevelText>, Without<UnitDetailsProfessionText>)>,
    mut level_text_query: Query<&mut Text, (With<UnitDetailsLevelText>, Without<UnitDetailsNameText>, Without<UnitDetailsProfessionText>)>,
    mut profession_text_query: Query<&mut Text, (With<UnitDetailsProfessionText>, Without<UnitDetailsNameText>, Without<UnitDetailsLevelText>)>,
) {
    // Only update when a unit is selected
    let Some(selected_unit_id) = cell_view_state.selected_unit else {
        return;
    };

    // For now, display basic information
    // TODO: Fetch actual unit data from a units resource/cache
    for mut text in &mut name_text_query {
        **text = format!("Name: Unit #{}", selected_unit_id);
    }

    for mut text in &mut level_text_query {
        **text = format!("Level: {}", 1); // TODO: Get actual level
    }

    for mut text in &mut profession_text_query {
        **text = format!("Profession: Worker"); // TODO: Get actual profession
    }
}

/// Handle clicks on the close button
pub fn handle_close_button(
    mut cell_view_state: ResMut<CellViewState>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<UnitDetailsCloseButton>)>,
) {
    for interaction in &button_query {
        if matches!(interaction, Interaction::Pressed) {
            info!("Unit details panel closed via button");
            cell_view_state.selected_unit = None;
        }
    }
}
