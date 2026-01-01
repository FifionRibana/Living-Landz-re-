use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use shared::grid::GridCell;
// use shared::grid::GridCell;
use crate::state::resources::{UnitsCache, UnitsDataCache, WorldCache};
use crate::ui::components::{
    ActionBarMarker, ActionsPanelMarker, CellDetailsPanelMarker, CellViewBackButton,
    CellViewBackgroundImage, CellViewContainer, ChatPanelMarker, SlotGridContainer, SlotIndicator,
    SlotState, SlotUnitPortrait, SlotUnitSprite, TopBarMarker,
};
use crate::ui::resources::CellViewState;
use crate::ui::systems::{PendingHexMask, SlotBorderOverlay};
use hexx::HexLayout;
use shared::{BiomeTypeEnum, SlotConfiguration, SlotPosition, SlotType, UnitData};

/// Update cell view and world UI visibility based on CellViewState
pub fn update_cell_view_visibility(
    cell_view_state: Res<CellViewState>,
    mut cell_view_query: Query<&mut Visibility, With<CellViewContainer>>,
    mut world_ui_query: Query<
        &mut Visibility,
        (
            Without<CellViewContainer>,
            Or<(
                With<ActionBarMarker>,
                With<ActionsPanelMarker>,
                With<CellDetailsPanelMarker>,
                With<TopBarMarker>,
                With<ChatPanelMarker>,
            )>,
        ),
    >,
) {
    if !cell_view_state.is_changed() {
        return;
    }

    // Show/hide cell view container
    for mut visibility in &mut cell_view_query {
        *visibility = if cell_view_state.is_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Hide world UI when in cell view mode (keep top bar and chat visible)
    for mut visibility in &mut world_ui_query {
        *visibility = if cell_view_state.is_active {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// Update cell view content when the viewed cell changes
pub fn update_cell_view_content(
    cell_view_state: Res<CellViewState>,
    world_cache: Res<WorldCache>,
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewContainer>>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
) {
    // Only rebuild content when the viewed cell actually changes, not on every state change
    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        // Cell view closed - reset state
        if last_viewed_cell.is_some() {
            *last_viewed_cell = None;
        }
        return;
    };

    // Check if this is the same cell we're already displaying
    if *last_viewed_cell == Some(viewed_cell) {
        return;
    }

    // Cell changed - update the display
    *last_viewed_cell = Some(viewed_cell);

    // Get cell data
    let cell_data = world_cache.get_cell(&viewed_cell);
    let building = world_cache.get_building(&viewed_cell);

    let biome = cell_data
        .map(|c| c.biome)
        .unwrap_or(BiomeTypeEnum::Undefined);

    // Determine slot configuration based on building type or terrain
    let slot_config = if let Some(building_data) = building {
        // Try to get slot config from building type first
        if let Some(building_type) = building_data.to_building_type() {
            SlotConfiguration::for_building_type(building_type)
        } else {
            // Fallback to terrain type for trees or unknown buildings
            SlotConfiguration::for_terrain_type(biome)
        }
    } else {
        // No building, use terrain type
        SlotConfiguration::for_terrain_type(biome)
    };

    // Clear existing content in container
    for container_entity in &container_query {
        if let Ok(children) = children_query.get(container_entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // Rebuild content
        commands.entity(container_entity).with_children(|parent| {
            // 1. Terrain background (full screen, behind everything)
            let terrain_bg = super::load_terrain_background(&asset_server, biome);
            parent.spawn((
                ImageNode {
                    image: terrain_bg,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                CellViewBackgroundImage,
            ));

            // 2. Building background and separators container (if building exists)
            if let Some(building_data) = building {
                let building_bg = super::load_building_background(&asset_server, building_data);

                // Check if building has interior slots
                if slot_config.has_interior() {
                    // Buildings WITH interior: square 1:1 ratio with separators
                    let (left_separator, right_separator) =
                        super::load_separators(&asset_server, Some(building_data));

                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },))
                        .with_children(|building_container| {
                            // Left separator
                            building_container.spawn((
                                ImageNode {
                                    image: left_separator,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    width: Val::Auto,
                                    ..default()
                                },
                            ));

                            // Building background (square 1:1 ratio, height constrained)
                            building_container.spawn((
                                ImageNode {
                                    image: building_bg,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    aspect_ratio: Some(1.0), // Force 1:1 ratio
                                    ..default()
                                },
                            ));

                            // Right separator
                            building_container.spawn((
                                ImageNode {
                                    image: right_separator,
                                    ..default()
                                },
                                Node {
                                    height: Val::Percent(100.0),
                                    width: Val::Auto,
                                    ..default()
                                },
                            ));
                        });
                } else {
                    // Buildings WITHOUT interior (only exterior): 16:9 ratio, no separators
                    parent.spawn((
                        ImageNode {
                            image: building_bg,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                }
            }

            // Main content container (on top of background)
            parent
                .spawn((Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(85.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    position_type: PositionType::Relative,
                    ..default()
                },))
                .with_children(|content| {
                    // Title
                    content.spawn((
                        Text::new(format!("Cell: q={}, r={}", viewed_cell.q, viewed_cell.r)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Unified slot container - Interior at center, Exterior around
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|section| {
                            // Create HexLayout for slot positioning (70.0 for 112px slots)
                            let slot_hex_layout = HexLayout::pointy().with_hex_size(70.0);

                            // Load hex images
                            let interior_hex_image = asset_server.load("ui/ui_hex_normal.png");
                            let exterior_hex_image = asset_server.load("ui/ui_hex_normal.png");
                            
                            // Get all occupied slots for the current cell
                            let occupied_slots = units_cache.get_occupied_slots(&viewed_cell);


                            // Unified container with absolute positioning
                            let container_size = Vec2::new(900.0, 700.0);
                            section
                                .spawn((
                                    Node {
                                        position_type: PositionType::Relative,
                                        width: Val::Px(container_size.y),
                                        height: Val::Px(container_size.y),
                                        padding: UiRect::all(Val::Px(16.0)),
                                        ..default()
                                    },
                                    // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
                                    // BorderRadius::all(Val::Px(8.0)),
                                ))
                                .with_children(|container| {
                                    // Generate and place INTERIOR slots at CENTER
                                    if slot_config.has_interior() {
                                        let interior_positions = slot_config
                                            .interior_layout
                                            .generate_positions(container_size, &slot_hex_layout);

                                        for (index, pos) in interior_positions.iter().enumerate() {
                                            let slot_indicator =
                                                SlotIndicator::new(SlotPosition::interior(index));
                                            let opacity = slot_indicator
                                                .state
                                                .get_opacity(slot_indicator.is_occupied());

                                            container
                                                .spawn((
                                                    Button,
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: Val::Px(pos.x - 56.0), // Center (112px / 2)
                                                        top: Val::Px(pos.y - 65.0), // Center (130px / 2)
                                                        width: Val::Px(112.0),
                                                        height: Val::Px(130.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        ..default()
                                                    },
                                                    ImageNode {
                                                        image: interior_hex_image.clone(),
                                                        color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                        ..default()
                                                    },
                                                    slot_indicator,
                                                    Interaction::None,
                                                ))
                                                .observe(on_cell_slot_hover)
                                                .observe(on_cell_slot_leave)
                                                .observe(on_cell_slot_click)
                                                .observe(on_cell_slot_start_drag)
                                                .observe(on_cell_slot_drag)
                                                .observe(on_cell_slot_end_drag)
                                                .observe(on_cell_slot_drag_drop)
                                                .observe(on_cell_slot_enter_drag)
                                                .observe(on_cell_slot_over_drag)
                                                .observe(on_cell_slot_leave_drag);
                                            // .observe(on_cell_slot_hover);
                                        }
                                    }

                                    // Generate and place EXTERIOR slots AROUND the center
                                    let exterior_positions = slot_config
                                        .exterior_layout
                                        .generate_positions(container_size, &slot_hex_layout);

                                    for (index, pos) in exterior_positions.iter().enumerate() {
                                        let slot_indicator =
                                            SlotIndicator::new(SlotPosition::exterior(index));
                                        let opacity = slot_indicator.state.get_opacity(false); //slot_indicator.is_occupied());

                                        container
                                            .spawn((
                                                Button,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Px(pos.x - 56.0), // Center (112px / 2)
                                                    top: Val::Px(pos.y - 65.0), // Center (130px / 2)
                                                    width: Val::Px(112.0),
                                                    height: Val::Px(130.0),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },
                                                ImageNode {
                                                    image: exterior_hex_image.clone(),
                                                    color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                    ..default()
                                                },
                                                slot_indicator,
                                                Interaction::None,
                                            ))
                                            .observe(on_cell_slot_hover)
                                            .observe(on_cell_slot_leave)
                                            .observe(on_cell_slot_click)
                                            .observe(on_cell_slot_start_drag)
                                            .observe(on_cell_slot_drag)
                                            .observe(on_cell_slot_end_drag)
                                            .observe(on_cell_slot_drag_drop)
                                            .observe(on_cell_slot_enter_drag)
                                            .observe(on_cell_slot_over_drag)
                                            .observe(on_cell_slot_leave_drag);
                                    }
                                });
                        });
                });

            // Back button (positioned absolutely)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        left: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.7, 0.2, 0.2, 0.9)),
                    BorderRadius::all(Val::Px(8.0)),
                    CellViewBackButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("← Back"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });

        info!(
            "Cell view content updated for cell: q={}, r={} (interior: {}, exterior: {})",
            viewed_cell.q,
            viewed_cell.r,
            slot_config.interior_slots(),
            slot_config.exterior_slots()
        );
    }
}

pub fn draw_portrait(
    asset_server: Res<AssetServer>,
    mut parent: RelatedSpawnerCommands<ChildOf>,
    slot_position: SlotPosition,
    slot_state: SlotState,
    unit_data: &UnitData,
) {
    let unit_id = unit_data.id;
    parent
        .spawn((
            Node {
                width: Val::Px(112.0),
                height: Val::Px(130.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            SlotUnitPortrait {
                unit_id,
                slot_position,
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .with_children(|container| {
            let portrait_path = unit_data
                .avatar_url.clone()
                .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());
            let portrait_handle: Handle<Image> = asset_server.load(portrait_path);
            let mask_handle: Handle<Image> = asset_server.load("ui/ui_hex_mask.png");

            container.spawn((
                ImageNode {
                    image: portrait_handle.clone(),
                    ..default()
                },
                Node {
                    width: Val::Px(112.0),
                    height: Val::Px(130.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                SlotUnitSprite {
                    unit_id,
                    slot_position,
                },
                PendingHexMask {
                    portrait_handle,
                    mask_handle,
                },
            ));

            let border_sprite_path = slot_state.get_sprite_path(true); // true = occupied
            let opacity = slot_state.get_opacity(true);

            container.spawn((
                ImageNode {
                    image: asset_server.load(&border_sprite_path),
                    color: Color::srgba(1.0, 1.0, 1.0, opacity),
                    ..default()
                },
                Node {
                    width: Val::Px(112.0),
                    height: Val::Px(130.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                SlotBorderOverlay {
                    slot_position,
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
            ));
        });
}

fn on_cell_slot_hover(
    over: On<Pointer<Over>>,
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&mut SlotIndicator, &mut ImageNode)>,
) {
    let is_dragging = slot_query
        .iter()
        .any(|(slot_indicator, _)| slot_indicator.is_dragging);
    if let Ok((mut slot_indicator, mut image_node)) = slot_query.get_mut(over.entity) {
        info!("Cell hover: {:?}", slot_indicator.position);
        return;
        slot_indicator.hovered = true;

        if !is_dragging {
            return;
        }

        // if slot_indicator.state != SlotState::Normal {
        //     return;
        // }

        // Get the appropriate sprite for the current state
        let sprite_path = slot_indicator.state.get_sprite_path(false); //slot_indicator.is_occupied());

        // Load and update the image
        image_node.image = asset_server.load(&sprite_path);

        // Update opacity based on state, BUT respect current interaction state
        // Don't overwrite hover/pressed colors set by update_slot_visual_feedback
        let opacity = slot_indicator.state.get_hover_opacity(false); //slot_indicator.is_occupied());
        image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    }
}

fn on_cell_slot_leave(
    out: On<Pointer<Out>>,
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&mut SlotIndicator, &mut ImageNode)>,
) {
    let is_dragging = slot_query
        .iter()
        .any(|(slot_indicator, _)| slot_indicator.is_dragging());
    if let Ok((mut slot_indicator, mut image_node)) = slot_query.get_mut(out.entity) {
        info!("Cell unhovered: {:?}", slot_indicator.position);

        slot_indicator.hovered = false;

        if !is_dragging {
            return;
        }

        // Get the appropriate sprite for the current state
        let sprite_path = slot_indicator.state.get_sprite_path(false); //slot_indicator.is_occupied());

        // Load and update the image
        image_node.image = asset_server.load(&sprite_path);

        // Update opacity based on state, BUT respect current interaction state
        // Don't overwrite hover/pressed colors set by update_slot_visual_feedback
        let opacity = slot_indicator.state.get_opacity(false); //slot_indicator.is_occupied());
        image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    }
}

fn on_cell_slot_click(
    click: On<Pointer<Click>>,
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&mut SlotIndicator, &mut ImageNode)>,
    mut cell_view_state: ResMut<CellViewState>,
) {
    if let Ok((mut slot_indicator, mut image_node)) = slot_query.get_mut(click.entity) {
        info!("Cell clicked: {:?}", slot_indicator.position);
        return;

        if !slot_indicator.is_occupied() || slot_indicator.is_dragging() {
            return;
        }

        match slot_indicator.state {
            SlotState::Normal => {
                // Deselect any previously selected slot first
                slot_indicator.state = SlotState::Selected;
                cell_view_state.select_slot(slot_indicator.position);
                info!("Slot selected: {:?}", slot_indicator.position);
            }
            SlotState::Selected => {
                slot_indicator.state = SlotState::Normal;
                cell_view_state.deselect_slot();
                info!("Slot deselected: {:?}", slot_indicator.position);
            }
            _ => {
                // Can't select disabled/invalid slots
                warn!("Cannot select slot in state: {:?}", slot_indicator.state);
            }
        }

        // Get the appropriate sprite for the current state
        // let sprite_path = slot_indicator
        //     .state
        //     .get_sprite_path(slot_indicator.is_occupied());

        // // Load and update the image
        // image_node.image = asset_server.load(&sprite_path);

        // let opacity = slot_indicator
        //     .state
        //     .get_hover_opacity(slot_indicator.is_occupied());

        // image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
        // info!("Cell click {:?}", slot_indicator);
    }
}

fn on_cell_slot_start_drag(drag_start: On<Pointer<DragStart>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag_start.entity) {
        info!("Cell drag started from: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_drag(drag: On<Pointer<Drag>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag.entity) {
        info!("Cell dragged: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_end_drag(drag_end: On<Pointer<DragEnd>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag_end.entity) {
        info!("Cell drag ended: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_drag_drop(drag_drop: On<Pointer<DragDrop>>, slot_query: Query<&SlotIndicator>) {
    if let Ok([slot_indicator_from, slot_indicator_to]) =
        slot_query.get_many([drag_drop.dropped, drag_drop.event_target()])
    {
        info!(
            "Cell drag drop from: {:?} to {:?}",
            slot_indicator_from.position, slot_indicator_to.position
        );
    }
}

fn on_cell_slot_enter_drag(drag_enter: On<Pointer<DragEnter>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag_enter.event_target()) {
        info!("Cell drag entered: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_over_drag(drag_over: On<Pointer<DragOver>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag_over.event_target()) {
        info!("Cell drag overed: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_leave_drag(drag_leave: On<Pointer<DragLeave>>, slot_query: Query<&SlotIndicator>) {
    if let Ok(slot_indicator) = slot_query.get(drag_leave.event_target()) {
        info!("Cell drag left: {:?}", slot_indicator.position);
    }
}
