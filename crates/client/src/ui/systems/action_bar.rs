use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::ui::components::{ActionBarMarker, ActionCategory, ActionCategoryButton};

pub fn setup_action_bar(parent: &mut RelatedSpawnerCommands<ChildOf>, asset_server: &Res<AssetServer>) {
    let categories = [
        (ActionCategory::Roads, "ui/icons/road.png", "Chemin"),
        (ActionCategory::Buildings, "ui/icons/village.png", "Constructions"),
        (ActionCategory::Production, "ui/icons/cog.png", "Production"),
        (ActionCategory::Management, "ui/icons/bookmarklet.png", "Gestion"),
        (ActionCategory::Entertainment, "ui/icons/laurels-trophy.png", "Divertissement"),
    ];

    parent
        .spawn((
            Node {
                width: px(60.),
                position_type: PositionType::Absolute,
                left: px(10.),
                bottom: px(170.),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.),
                ..default()
            },
            ActionBarMarker,
            Visibility::Hidden,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|action_bar| {
            for (category, icon_path, _label) in categories.iter() {
                action_bar
                    .spawn((
                        Button,
                        Node {
                            width: px(48.),
                            height: px(48.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        BorderColor::all(Color::srgb_u8(150, 130, 100)),
                        BorderRadius::all(px(8.)),
                        ActionCategoryButton { category: *category },
                        Pickable {
                            should_block_lower: true,
                            is_hoverable: true,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            ImageNode {
                                image: asset_server.load(*icon_path),
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
                    });
            }
        });
}

pub fn handle_action_category_button_interactions(
    mut query: Query<
        (&ActionCategoryButton, &Interaction),
        Changed<Interaction>,
    >,
    mut action_state: ResMut<crate::ui::resources::ActionState>,
) {
    for (category_button, interaction) in &mut query {
        if *interaction == Interaction::Pressed {
            action_state.select_category(category_button.category);
            info!("Action category toggled: {:?}", category_button.category);
        }
    }
}

pub fn update_action_category_button_appearance(
    mut query: Query<(&ActionCategoryButton, &mut BackgroundColor, &Interaction)>,
    action_state: Res<crate::ui::resources::ActionState>,
) {
    if action_state.is_changed() {
        for (category_button, mut background_color, interaction) in &mut query {
            let is_checked = action_state.is_category_checked(category_button.category);

            if is_checked {
                *background_color = BackgroundColor(Color::srgba(0.3, 0.5, 0.7, 0.9));
            } else {
                match *interaction {
                    Interaction::Hovered => {
                        *background_color = BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9));
                    }
                    _ => {
                        *background_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
                    }
                }
            }
        }
    }
}

pub fn update_action_bar_visibility(
    selected_hexes: Res<crate::grid::resources::SelectedHexes>,
    mut action_bar_query: Query<&mut Visibility, With<ActionBarMarker>>,
) {
    if selected_hexes.is_changed() {
        let is_selected = !selected_hexes.ids.is_empty();
        for mut visibility in action_bar_query.iter_mut() {
            *visibility = if is_selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
