use crate::networking::client::NetworkClient;
use crate::state::resources::{UnitsCache, UnitsDataCache};
use crate::ui::components::{SlotIndicator, SlotState, SlotUnitPortrait, SlotUnitSprite};
use crate::ui::resources::{CellState, CellViewState, PanelEnum, UIState};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use shared::SlotPosition;
use shared::protocol::ClientMessage;

/// DONE : Utilisation du mécanisme d'observer plutôt que Interaction.
/// TODO : Drag => drop:
/// TODO :    - Detect target slot: Add "In drag event" / hover enabled dans ce cas
/// TODO :    - Set une propriété: Target Slot
/// TODO :    - Valid   => Message server pour déplacer
/// TODO :    - Invalid => Rendre dirty la vue et redraw les unités
///
/// TODO : Gérer la sélection comme emplacements sélectionné dans la vue
/// TODO : Si déplacement unité sélectionnée alors => sélection bouge
/// TODO : Cliquer sur une unité => Toggle de la sélection
/// TODO : La vue de l'unité doit se mettre à jour

/// Update unit sprites displayed on slots
pub fn update_unit_sprites(
    mut commands: Commands,
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    asset_server: Res<AssetServer>,
    // Query existing unit sprites to remove them
    existing_sprites: Query<(Entity, &SlotUnitSprite)>,
    // Query existing border overlays to remove them
    existing_borders: Query<(Entity, &SlotBorderOverlay)>,
    // Query slot indicators to attach sprites to them
    slot_query: Query<(Entity, &SlotIndicator)>,
    // Track the last viewed cell to detect when we switch to a different cell
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
    // Track if we need to retry on the next frame (when UI isn't ready yet)
    mut needs_retry: Local<bool>,
    // Cache for hex mask
    mut hex_mask_handle: Local<Option<Handle<Image>>>,
) {
    // Only update when cell view is active
    if !cell_view_state.is_active {
        *last_viewed_cell = None;
        *needs_retry = false;
        return;
    }

    // Don't update during drag & drop (or potential drag) to avoid flickering
    if cell_view_state.dragging_unit.is_some() || cell_view_state.potential_drag.is_some() {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        *last_viewed_cell = None;
        *needs_retry = false;
        return;
    };

    // Check if we just switched to a different cell
    let cell_just_changed = *last_viewed_cell != Some(viewed_cell);
    if cell_just_changed {
        *last_viewed_cell = Some(viewed_cell);
        *needs_retry = true; // Will retry next frame to ensure UI is ready
        info!(
            "Cell view opened for cell ({}, {}), will update sprites",
            viewed_cell.q, viewed_cell.r
        );
    }

    // Only update if:
    // 1. We just opened/switched to this cell, OR
    // 2. We need to retry from last frame, OR
    // 3. The units cache changed (units added/moved/removed)
    if !cell_just_changed && !*needs_retry && !units_cache.is_changed() {
        return;
    }

    // Get all occupied slots for the current cell
    let occupied_slots = units_cache.get_occupied_slots(&viewed_cell);

    // Count how many slot entities we can actually find
    let slot_count = slot_query.iter().count();

    info!(
        "Cell view update: Found {} occupied slots, {} slot UI elements for cell ({}, {})",
        occupied_slots.len(),
        slot_count,
        viewed_cell.q,
        viewed_cell.r
    );

    // If no slots exist yet, the UI hasn't been created - skip this frame but keep retry flag
    if slot_count == 0 && !occupied_slots.is_empty() {
        warn!("Slot UI not ready yet, skipping sprite update (will retry next frame)");
        // Keep needs_retry = true so we'll try again next frame
        return;
    }

    // If we successfully processed, we can turn off retry
    if *needs_retry && slot_count > 0 {
        info!("Slot UI ready, processing unit sprites");
        *needs_retry = false;
    }

    // Build a map of current unit sprites
    let mut existing_sprites_map = std::collections::HashMap::new();
    for (entity, sprite) in &existing_sprites {
        existing_sprites_map.insert((sprite.slot_position, sprite.unit_id), entity);
    }

    // Build a map of current border overlays
    let mut existing_borders_map = std::collections::HashMap::new();
    for (entity, border) in &existing_borders {
        existing_borders_map.insert(border.slot_position, entity);
    }

    // Build a map of what should exist
    let mut should_exist = std::collections::HashMap::new();
    for (slot_pos, unit_id) in &occupied_slots {
        should_exist.insert((*slot_pos, *unit_id), true);
    }

    // Remove sprites that shouldn't exist anymore
    for ((slot_pos, unit_id), entity) in &existing_sprites_map {
        if !should_exist.contains_key(&(*slot_pos, *unit_id)) {
            commands.entity(*entity).despawn();
        }
    }

    // Remove borders for empty slots
    for (slot_pos, entity) in &existing_borders_map {
        let is_occupied = occupied_slots.iter().any(|(pos, _)| pos == slot_pos);
        if !is_occupied {
            commands.entity(*entity).despawn();
        }
    }

    // Add sprites that don't exist yet
    for (slot_entity, slot_indicator) in &slot_query {
        // Check if this slot is occupied
        if let Some((_, unit_id)) = occupied_slots
            .iter()
            .find(|(slot_pos, _)| *slot_pos == slot_indicator.position)
        {
            // Only spawn if sprite doesn't already exist
            if !existing_sprites_map.contains_key(&(slot_indicator.position, *unit_id)) {
                // Get unit data to load the correct portrait
                let portrait_path = units_data_cache
                    .get_unit(*unit_id)
                    .and_then(|unit_data| unit_data.avatar_url.clone())
                    .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

                // Load hex mask once (cached in Local)
                if hex_mask_handle.is_none() {
                    *hex_mask_handle = Some(asset_server.load("ui/ui_hex_mask.png"));
                }

                // Load the portrait image
                let portrait_handle: Handle<Image> = asset_server.load(portrait_path);
                let mask_handle = hex_mask_handle.clone().unwrap();

                // Spawn unit portrait AND border as children of the slot button
                // Portrait first (rendered below), then border (rendered on top)
                commands.entity(slot_entity).with_children(|parent| {
                    // 1. Portrait (will be masked with hex shape)
                    parent.spawn((
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
                            unit_id: *unit_id,
                            slot_position: slot_indicator.position,
                        },
                        PendingHexMask {
                            portrait_handle,
                            mask_handle,
                        },
                    ));

                    // 2. Border overlay (hex _empty sprite on top of portrait)
                    let border_sprite_path = slot_indicator.state.get_sprite_path(true); // true = occupied
                    let opacity = slot_indicator.state.get_opacity(true);

                    parent.spawn((
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
                            slot_position: slot_indicator.position,
                        },
                    ));
                });

                info!(
                    "Spawned portrait for unit {} with PendingHexMask and border overlay",
                    unit_id
                );
            }
        }
    }
}

/// Component to mark portraits that need hex masking
#[derive(Component)]
pub struct PendingHexMask {
    pub portrait_handle: Handle<Image>,
    pub mask_handle: Handle<Image>,
}

/// Component to mark border overlays that should be displayed on top of portraits
#[derive(Component)]
pub struct SlotBorderOverlay {
    pub slot_position: shared::SlotPosition,
}


/// Update border overlays when slot state changes
pub fn update_border_overlays(
    asset_server: Res<AssetServer>,
    // Query slots to check their state
    slot_query: Query<(&SlotIndicator, &Children), Changed<SlotIndicator>>,
    // Query borders to update them
    mut border_query: Query<(&SlotBorderOverlay, &mut ImageNode)>,
) {
    for (slot_indicator, children) in &slot_query {
        // Only update if slot is occupied
        if !slot_indicator.is_occupied() {
            continue;
        }

        // Find the border overlay child
        for child in children.iter() {
            if let Ok((border, mut image_node)) = border_query.get_mut(child) {
                if border.slot_position == slot_indicator.position {
                    // Update border sprite based on new state
                    let border_sprite_path = slot_indicator.state.get_sprite_path(true);
                    image_node.image = asset_server.load(&border_sprite_path);
                }
            }
        }
    }
}

/*
pub fn update_unit_portraits(
    mut commands: Commands,
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    asset_server: Res<AssetServer>,
    // Query existing unit sprites to remove them
    existing_sprites: Query<(Entity, &SlotUnitSprite)>,
    // Query existing border overlays to remove them
    existing_borders: Query<(Entity, &SlotBorderOverlay)>,
    // Query slot indicators to attach sprites to them
    slot_query: Query<(Entity, &SlotIndicator)>,
    // Track the last viewed cell to detect when we switch to a different cell
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
    // Track if we need to retry on the next frame (when UI isn't ready yet)
    mut needs_retry: Local<bool>,
    // Cache for hex mask
    mut hex_mask_handle: Local<Option<Handle<Image>>>,
) {
    // Only update when cell view is active
    if !cell_view_state.is_active {
        *last_viewed_cell = None;
        *needs_retry = false;
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        *last_viewed_cell = None;
        *needs_retry = false;
        return;
    };

    // Check if we just switched to a different cell
    let cell_just_changed = *last_viewed_cell != Some(viewed_cell);
    if cell_just_changed {
        *last_viewed_cell = Some(viewed_cell);
        *needs_retry = true; // Will retry next frame to ensure UI is ready
        info!(
            "Cell view opened for cell ({}, {}), will update sprites",
            viewed_cell.q, viewed_cell.r
        );
    }

    // Only update if:
    // 1. We just opened/switched to this cell, OR
    // 2. We need to retry from last frame, OR
    // 3. The units cache changed (units added/moved/removed)
    if !cell_just_changed && !*needs_retry && !units_cache.is_changed() {
        return;
    }

    // Get all occupied slots for the current cell
    let occupied_slots = units_cache.get_occupied_slots(&viewed_cell);

    // Count how many slot entities we can actually find
    let slot_count = slot_query.iter().count();

    info!(
        "Cell view update: Found {} occupied slots, {} slot UI elements for cell ({}, {})",
        occupied_slots.len(),
        slot_count,
        viewed_cell.q,
        viewed_cell.r
    );

    // If no slots exist yet, the UI hasn't been created - skip this frame but keep retry flag
    if slot_count == 0 && !occupied_slots.is_empty() {
        warn!("Slot UI not ready yet, skipping sprite update (will retry next frame)");
        // Keep needs_retry = true so we'll try again next frame
        return;
    }

    // If we successfully processed, we can turn off retry
    if *needs_retry && slot_count > 0 {
        info!("Slot UI ready, processing unit sprites");
        *needs_retry = false;
    }

    // Build a map of current unit sprites
    let mut existing_sprites_map = std::collections::HashMap::new();
    for (entity, sprite) in &existing_sprites {
        existing_sprites_map.insert((sprite.slot_position, sprite.unit_id), entity);
    }

    // Build a map of current border overlays
    let mut existing_borders_map = std::collections::HashMap::new();
    for (entity, border) in &existing_borders {
        existing_borders_map.insert(border.slot_position, entity);
    }

    // Build a map of what should exist
    let mut should_exist = std::collections::HashMap::new();
    for (slot_pos, unit_id) in &occupied_slots {
        should_exist.insert((*slot_pos, *unit_id), true);
    }

    // Remove sprites that shouldn't exist anymore
    for ((slot_pos, unit_id), entity) in &existing_sprites_map {
        if !should_exist.contains_key(&(*slot_pos, *unit_id)) {
            commands.entity(*entity).despawn();
        }
    }

    // Remove borders for empty slots
    for (slot_pos, entity) in &existing_borders_map {
        let is_occupied = occupied_slots.iter().any(|(pos, _)| pos == slot_pos);
        if !is_occupied {
            commands.entity(*entity).despawn();
        }
    }

    // Add sprites that don't exist yet
    for (slot_entity, slot_indicator) in &slot_query {
        // Check if this slot is occupied
        if let Some((_, unit_id)) = occupied_slots
            .iter()
            .find(|(slot_pos, _)| *slot_pos == slot_indicator.position)
        {
            // Only spawn if sprite doesn't already exist
            if !existing_sprites_map.contains_key(&(slot_indicator.position, *unit_id)) {
                // Get unit data to load the correct portrait
                let portrait_path = units_data_cache
                    .get_unit(*unit_id)
                    .and_then(|unit_data| unit_data.avatar_url.clone())
                    .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

                // Load hex mask once (cached in Local)
                if hex_mask_handle.is_none() {
                    *hex_mask_handle = Some(asset_server.load("ui/ui_hex_mask.png"));
                }

                // Load the portrait image
                let portrait_handle: Handle<Image> = asset_server.load(portrait_path);
                let mask_handle = hex_mask_handle.clone().unwrap();

                // Spawn unit portrait AND border as children of the slot button
                // Portrait first (rendered below), then border (rendered on top)
                commands.entity(slot_entity).with_children(|parent| {
                    let mut portrait_container = parent.spawn((
                        Node {
                            width: Val::Px(112.0),
                            height: Val::Px(130.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        slot_indicator.clone(),
                        SlotUnitPortrait {
                            unit_id: *unit_id,
                            slot_position: slot_indicator.position,
                        },
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: false,
                        },
                        GlobalZIndex(0),
                    ));
                    portrait_container
                        .with_children(|container_parent| {
                            // 1. Portrait (will be masked with hex shape)
                            container_parent.spawn((
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
                                    unit_id: *unit_id,
                                    slot_position: slot_indicator.position,
                                },
                                PendingHexMask {
                                    portrait_handle,
                                    mask_handle,
                                },
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: true,
                                },
                            ));

                            // 2. Border overlay (hex _empty sprite on top of portrait)
                            let border_sprite_path = slot_indicator.state.get_sprite_path(true); // true = occupied
                            let opacity = slot_indicator.state.get_opacity(true);

                            container_parent.spawn((
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
                                    slot_position: slot_indicator.position,
                                },
                                Pickable {
                                    should_block_lower: false,
                                    is_hoverable: true,
                                },
                            ));
                            // .observe(on_cell_slot_hover)
                            // .observe(on_cell_slot_leave);
                            // .observe(on_cell_slot_click);
                        })
                        // .observe(on_cell_slot_start_drag)
                        // .observe(on_cell_slot_drag)
                        // .observe(on_cell_slot_drag_end)
                        // .observe(on_cell_slot_drag_drop);
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
                });

                info!(
                    "Spawned portrait for unit {} with PendingHexMask and border overlay",
                    unit_id
                );
            }
        }
    }
}
*/
// fn on_cell_slot_hover(
//     over: On<Pointer<Over>>,
//     asset_server: Res<AssetServer>,
//     slot_query: Query<&SlotIndicator, Without<SlotUnitPortrait>>,
//     mut border_query: Query<(&SlotBorderOverlay, &mut ImageNode)>,
// ) {
//     if let Ok((border_overlay, mut image_node)) = border_query.get_mut(over.entity)
//         && let Some(slot_indicator) = slot_query
//             .iter()
//             .find(|&slot_indicator| slot_indicator.position == border_overlay.slot_position)
//     {
//         if slot_indicator.state != SlotState::Normal {
//             return;
//         }

//         // Get the appropriate sprite for the current state
//         let sprite_path = slot_indicator.state.get_sprite_path(true);

//         // Load and update the image
//         image_node.image = asset_server.load(&sprite_path);

//         // Update opacity based on state, BUT respect current interaction state
//         // Don't overwrite hover/pressed colors set by update_slot_visual_feedback
//         let opacity = slot_indicator.state.get_hover_opacity(true);
//         image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);

//         info!("Cell hovered: {:?}", slot_indicator);
//     }
// }

// fn on_cell_slot_leave(
//     out: On<Pointer<Out>>,
//     asset_server: Res<AssetServer>,
//     slot_query: Query<&SlotIndicator, Without<SlotUnitPortrait>>,
//     mut border_query: Query<(&SlotBorderOverlay, &mut ImageNode)>,
// ) {
//     if let Ok((border_overlay, mut image_node)) = border_query.get_mut(out.entity)
//         && let Some(slot_indicator) = slot_query
//             .iter()
//             .find(|&slot_indicator| slot_indicator.position == border_overlay.slot_position)
//     {
//         if slot_indicator.state != SlotState::Normal {
//             return;
//         }

//         // Get the appropriate sprite for the current state
//         let sprite_path = slot_indicator.state.get_sprite_path(true);

//         // Load and update the image
//         image_node.image = asset_server.load(&sprite_path);

//         // Update opacity based on state, BUT respect current interaction state
//         // Don't overwrite hover/pressed colors set by update_slot_visual_feedback
//         let opacity = slot_indicator.state.get_opacity(true);
//         image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);

//         info!("Cell exited: {:?}", slot_indicator);
//     }
// }

// pub fn update_portraits_visual(
//     asset_server: Res<AssetServer>,
//     slot_query: Query<&SlotIndicator, Without<SlotUnitPortrait>>,
//     border_query: Query<(&SlotBorderOverlay, &mut ImageNode)>,
// ) {
//     for (border_overlay, mut image_node) in border_query {
//         if let Some(slot_indicator) = slot_query
//             .iter()
//             .find(|slot_indicator| slot_indicator.position == border_overlay.slot_position)
//         {
//             // Get the appropriate sprite for the current state
//             let sprite_path = slot_indicator.state.get_sprite_path(true);

//             // Load and update the image
//             image_node.image = asset_server.load(&sprite_path);

//             let opacity = if slot_indicator.is_hovered() {
//                 slot_indicator.state.get_hover_opacity(true)
//             } else {
//                 slot_indicator.state.get_opacity(true)
//             };

//             image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
//         }
//     }
// }

// fn on_cell_slot_click(
//     click: On<Pointer<Click>>,
//     asset_server: Res<AssetServer>,
//     mut slot_query: Query<&mut SlotIndicator, Without<SlotUnitPortrait>>,
//     mut border_query: Query<(&SlotBorderOverlay, &mut ImageNode)>,
//     mut cell_view_state: ResMut<CellViewState>,
// ) {
//     if let Ok((border_overlay, mut image_node)) = border_query.get_mut(click.entity)
//         && let Some(mut slot_indicator) = slot_query
//             .iter_mut()
//             .find(|slot_indicator| slot_indicator.position == border_overlay.slot_position)
//     {
//         // match slot_indicator.state {
//         //     SlotState::Normal => {
//         //         // Deselect any previously selected slot first
//         //         slot_indicator.state = SlotState::Selected;
//         //         cell_view_state.select_slot(slot_indicator.position);
//         //         info!("Slot selected: {:?}", slot_indicator.position);
//         //     }
//         //     SlotState::Selected => {
//         //         slot_indicator.state = SlotState::Normal;
//         //         cell_view_state.deselect_slot();
//         //         info!("Slot deselected: {:?}", slot_indicator.position);
//         //     }
//         //     _ => {
//         //         // Can't select disabled/invalid slots
//         //         warn!("Cannot select slot in state: {:?}", slot_indicator.state);
//         //         return;
//         //     }
//         // }

//         // Get the appropriate sprite for the current state
//         let sprite_path = slot_indicator.state.get_sprite_path(true);

//         // Load and update the image
//         image_node.image = asset_server.load(&sprite_path);

//         let opacity = slot_indicator.state.get_hover_opacity(true);

//         image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);

//         // info!("Cell click {:?}", slot_indicator);
//     }
// }

// fn on_cell_slot_start_drag(
//     drag_start: On<Pointer<DragStart>>,
//     mut slot_query: Query<&mut SlotIndicator, Without<SlotUnitPortrait>>,
//     mut portrait_query: Query<(&SlotUnitPortrait, &mut GlobalZIndex)>,
// ) {
//     if let Ok((portrait, mut global_zindex)) = portrait_query.get_mut(drag_start.entity)
//         && let Some(mut slot_indicator) = slot_query
//             .iter_mut()
//             .find(|slot_indicator| slot_indicator.position == portrait.slot_position)
//     {
//         slot_indicator.is_dragging = true;
//         global_zindex.0 = 20;
//     }
// }

// fn on_cell_slot_drag(
//     drag: On<Pointer<Drag>>,
//     mut transform_query: Query<(&SlotUnitPortrait, &mut UiTransform)>,
// ) {
//     if let Ok((_portrait, mut ui_transform)) = transform_query.get_mut(drag.event_target()) {
//         ui_transform.translation = Val2::px(drag.distance.x, drag.distance.y);
//     }
// }

// fn on_cell_slot_drag_end(
//     drag_end: On<Pointer<DragEnd>>,
//     mut slot_query: Query<&mut SlotIndicator, Without<SlotUnitPortrait>>,
//     mut portrait_query: Query<(&SlotUnitPortrait, &mut UiTransform, &mut GlobalZIndex)>,
// ) {
//     if let Ok((portrait, mut ui_transform, mut global_zindex)) =
//         portrait_query.get_mut(drag_end.entity)
//         && let Some(mut slot_indicator) = slot_query
//             .iter_mut()
//             .find(|slot_indicator| slot_indicator.position == portrait.slot_position)
//     {
//         slot_indicator.is_dragging = false;
//         ui_transform.translation = Val2::ZERO;
//         global_zindex.0 = 0;
//     }
// }

// fn on_cell_slot_drag_drop(
//     drag_drop: On<Pointer<DragDrop>>,
//     mut portrait_query: Query<&SlotUnitPortrait>,
//     slot_query: Query<&SlotIndicator, Without<SlotUnitPortrait>>,
//     units_cache: Res<UnitsCache>,
//     cell_view_state: Res<CellViewState>,
//     mut network_client: ResMut<NetworkClient>,
// ) {
//     info!("Drop from drag!");
//     let Some(viewed_cell) = cell_view_state.viewed_cell else {
//         return;
//     };

//     if let Ok(portrait) = portrait_query.get_mut(drag_drop.entity)
//         && let Some(drop_target) = slot_query
//             .iter()
//             .find(|slot_indicator| slot_indicator.is_hovered())
//         && drop_target.position != portrait.slot_position
//     {
//         // Check if target slot is empty
//         if units_cache.is_slot_occupied(&viewed_cell, &drop_target.position) {
//             warn!(
//                 "Cannot drop unit {} on occupied slot {:?}",
//                 portrait.unit_id, drop_target.position
//             );
//         } else {
//             // Send move request to server
//             info!(
//                 "Moving unit {} from {:?} to {:?}",
//                 portrait.unit_id, portrait.slot_position, drop_target.position
//             );

//             let message = ClientMessage::MoveUnitToSlot {
//                 unit_id: portrait.unit_id,
//                 cell: viewed_cell,
//                 from_slot: portrait.slot_position,
//                 to_slot: drop_target.position,
//             };

//             network_client.send_message(message);

//             // Keep unit selected if it was selected before the drag
//             // if was_selected {
//             //     cell_view_state.selected_unit = Some(dragging_unit.unit_id);
//             // }
//         }
//     }
// }

fn on_cell_slot_hover(
    over: On<Pointer<Over>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(over.event_target()) {
        info!("Cell hover: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_leave(
    out: On<Pointer<Out>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(out.event_target()) {
        info!("Cell unhovered: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_click(
    click: On<Pointer<Click>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(click.event_target()) {
        info!("Cell clicked: {:?}", slot_indicator.position);
    }
}

fn on_cell_slot_start_drag(
    drag_start: On<Pointer<DragStart>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag_start.event_target()) {
        info!("Cell drag started from: {:?}", slot_indicator.position);
        
    }
}

fn on_cell_slot_drag(
    drag: On<Pointer<Drag>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag.event_target()) {
        info!("Cell dragged: {:?}", slot_indicator.position);
        
    }
}

fn on_cell_slot_end_drag(
    drag_end: On<Pointer<DragEnd>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag_end.event_target()) {
        info!("Cell drag ended: {:?}", slot_indicator.position);
        
    }
}

fn on_cell_slot_drag_drop(
    drag_drop: On<Pointer<DragDrop>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok([slot_indicator_from, slot_indicator_to]) = slot_query.get_many([drag_drop.dropped, drag_drop.event_target()]) {
        info!("Cell drag drop from: {:?} to {:?}", slot_indicator_from.position, slot_indicator_to.position);
        
    }
}

fn on_cell_slot_enter_drag(
    drag_enter: On<Pointer<DragEnter>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag_enter.event_target()) {
        info!("Cell drag entered: {:?}", slot_indicator.position);
        
    }
}

fn on_cell_slot_over_drag(
    drag_over: On<Pointer<DragOver>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag_over.event_target()) {
        info!("Cell drag overed: {:?}", slot_indicator.position);
        
    }
}

fn on_cell_slot_leave_drag(
    drag_leave: On<Pointer<DragLeave>>,
    slot_query: Query<&SlotIndicator>,
) {
    if let Ok(slot_indicator) = slot_query.get(drag_leave.event_target()) {
        info!("Cell drag left: {:?}", slot_indicator.position);
        
    }
}