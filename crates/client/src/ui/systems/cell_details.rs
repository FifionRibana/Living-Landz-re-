use bevy::prelude::*;
use shared::{
    atlas::{BuildingAtlas, GaugeAtlas, MoonAtlas},
    grid::GridCell,
};

use crate::{
    grid::resources::SelectedHexes,
    state::resources::WorldCache,
    ui::components::{
        CellDetailsBiomeText, CellDetailsBuildingImage, CellDetailsPanelMarker,
        CellDetailsQualityGaugeContainer, CellDetailsTitleText,
    },
};

use super::{
    action_bar::setup_action_bar, action_panel_setup::setup_action_panel,
    cell_details_panel::setup_cell_details_panel, chat_panel_setup::setup_chat_panel,
    top_bar::setup_top_bar,
};

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gauge_atlas: Res<GaugeAtlas>,
    moon_atlas: Res<MoonAtlas>,
) {
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
            // Top bar
            setup_top_bar(parent, &asset_server, &moon_atlas);

            // Cell details panel (right side)
            setup_cell_details_panel(parent, &asset_server, &gauge_atlas);

            // Action bar (left sidebar)
            setup_action_bar(parent, &asset_server);

            // Action panel (bottom, with tabs)
            setup_action_panel(parent, &asset_server);

            // Chat panel and icon
            setup_chat_panel(parent, &asset_server);
        });
}

pub fn update_cell_details_visibility(
    mut query: Query<&mut Visibility, With<CellDetailsPanelMarker>>,
    selected: Res<SelectedHexes>,
) {
    if selected.is_changed() {
        let is_selected = !selected.ids.is_empty();

        for mut visibility in query.iter_mut() {
            *visibility = if is_selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn update_cell_details_content(
    selected: Res<SelectedHexes>,
    world_cache: Res<WorldCache>,
    building_atlas: Res<BuildingAtlas>,
    mut text_query: Query<(
        &mut Text,
        Option<&CellDetailsTitleText>,
        Option<&CellDetailsBiomeText>,
    )>,
    mut building_image_query: Query<
        (&mut ImageNode, &mut Visibility),
        (
            With<CellDetailsBuildingImage>,
            Without<CellDetailsQualityGaugeContainer>,
        ),
    >,
    mut gauge_query: Query<
        &mut Visibility,
        (
            With<CellDetailsQualityGaugeContainer>,
            Without<CellDetailsBuildingImage>,
        ),
    >,
) {
    if !selected.is_changed() {
        return;
    }

    // Get the first selected cell (assuming single selection for now)
    let Some(selected_hex) = selected.ids.iter().next() else {
        return;
    };

    let grid_cell = GridCell::from_hex(selected_hex);

    // Get cell data for biome
    let cell_data = world_cache.get_cell(&grid_cell);
    let building_data = world_cache.get_building(&grid_cell);

    // Update title, biome text, building image, and gauge based on building presence
    if let Some(building) = building_data {
        // Get building type and variant
        let (building_type, variant) = match &building.specific_data {
            shared::BuildingSpecific::ManufacturingWorkshop(data) => {
                (data.workshop_type.to_building_type(), data.variant as usize)
            }
            shared::BuildingSpecific::Agriculture(data) => (
                data.agriculture_type.to_building_type(),
                data.variant as usize,
            ),
            shared::BuildingSpecific::AnimalBreeding(data) => {
                (data.animal_type.to_building_type(), data.variant as usize)
            }
            shared::BuildingSpecific::Entertainment(data) => (
                data.entertainment_type.to_building_type(),
                data.variant as usize,
            ),
            shared::BuildingSpecific::Cult(data) => {
                (data.cult_type.to_building_type(), data.variant as usize)
            }
            shared::BuildingSpecific::Commerce(data) => {
                (data.commerce_type.to_building_type(), data.variant as usize)
            }
            shared::BuildingSpecific::Tree(_) => {
                // Trees don't have a specific building type in BuildingTypeEnum
                // Just show "Tree" as title
                for (mut text, title_query, biome_query) in &mut text_query {
                    if title_query.is_some() {
                        **text = "Tree".to_string();
                    }

                    if biome_query.is_some() {
                        if let Some(cell) = cell_data {
                            **text = format!("{:?}", cell.biome);
                        }
                    }
                }
                // let mut text = query.single_mut();
                // if let Ok(mut text, (title_query, biome_query)) = query.single_mut() {
                //     **text = "Tree".to_string();
                // }
                // if let Ok(mut text) = biome_query.single_mut() {
                //     if let Some(cell) = cell_data {
                //         **text = format!("{:?}", cell.biome);
                //     }
                // }
                // Hide building image and gauge for trees
                // TODO: Hide if the panel is hidden
                if let Ok((_, mut visibility)) = building_image_query.single_mut() {
                    *visibility = Visibility::Hidden;
                }
                if let Ok(mut visibility) = gauge_query.single_mut() {
                    *visibility = Visibility::Hidden;
                }
                return;
            }
            shared::BuildingSpecific::Unknown() => {
                // Unknown building, don't show anything special
                return;
            }
        };

        // Update title with building type
        for (mut text, title_query, biome_query) in &mut text_query {
            if title_query.is_some() {
                **text = format!("{:?}", building_type);
            }

            if biome_query.is_some() {
                if let Some(cell) = cell_data {
                    **text = format!("{:?}", cell.biome);
                }
            }
        }

        // if let Ok(mut text) = title_query.single_mut() {
        //     **text = format!("{:?}", building_type);
        // }

        // // Update biome text
        // if let Ok(mut text) = biome_query.single_mut() {
        //     if let Some(cell) = cell_data {
        //         **text = format!("{:?}", cell.biome);
        //     }
        // }

        // Update building image
        if let Ok((mut image_node, mut visibility)) = building_image_query.single_mut() {
            if let Some(sprite_handle) = building_atlas.get_sprite(building_type, variant) {
                image_node.image = sprite_handle.clone();
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }

        // Show quality gauge
        if let Ok(mut visibility) = gauge_query.single_mut() {
            *visibility = Visibility::Visible;
        }
    } else {
        // No building: show only biome

        for (mut text, title_query, biome_query) in &mut text_query {
            if title_query.is_some() {
                **text = String::new();
            }

            if biome_query.is_some() {
                if let Some(cell) = cell_data {
                    **text = format!("{:?}", cell.biome);
                } else {
                    **text = String::new();
                }
            }
        }

        // if let Ok(mut text) = title_query.single_mut() {
        //     **text = String::new();
        // }

        // if let Ok(mut text) = biome_query.single_mut() {
        //     if let Some(cell) = cell_data {
        //         **text = format!("{:?}", cell.biome);
        //     } else {
        //         **text = String::new();
        //     }
        // }

        // Hide building image and gauge
        if let Ok((_, mut visibility)) = building_image_query.single_mut() {
            *visibility = Visibility::Hidden;
        }

        if let Ok(mut visibility) = gauge_query.single_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
