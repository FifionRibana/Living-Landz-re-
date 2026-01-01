use std::collections::HashSet;

use bevy::{prelude::*, window::PrimaryWindow};
use hexx::*;
use shared::{SlotPosition, protocol::ClientMessage};

use crate::{
    networking::client::NetworkClient,
    state::resources::{UnitsCache, UnitsDataCache},
    ui::{
        components::{
            ExteriorSlotContainer, InteriorSlotContainer, PendingHexMask, SlotBorderOverlay,
            SlotIndicator, SlotState, SlotUnitPortrait, SlotUnitSprite,
        },
        resources::{CellState, DragInfo, DragState, PanelEnum, UIState},
        systems::panels::components::CellViewPanel,
    },
};

// ═══════════════════════════════════════════════════════════════
// RELATION SLOT ↔ UNITÉ (one-to-one strict)
// Un slot contient au plus une unité, une unité occupe au plus un slot
// ═══════════════════════════════════════════════════════════════

/// L'unité déclare dans quel slot elle est positionnée.
/// Optionnel : une unité peut être sur une case sans occuper de slot spécifique.
#[derive(Component, Debug)]
#[relationship(relationship_target = SlotOccupant)]
pub struct InSlot(pub Entity);

/// Le slot connaît l'unité qui l'occupe (une seule).
/// Mis à jour automatiquement. Si on assigne une nouvelle unité au slot,
/// l'ancienne perd son composant InSlot.
#[derive(Component, Debug)]
#[relationship_target(relationship = InSlot)]
pub struct SlotOccupant(Entity);

impl SlotOccupant {
    pub fn get(&self) -> Entity {
        self.0
    }
}

pub fn setup_cell_slots(
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    ui_state: Res<UIState>,
    units_cache: Res<UnitsCache>,
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewPanel>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    slots_query: Query<Entity, With<SlotIndicator>>,
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
) {
    if ui_state.panel_state != PanelEnum::CellView {
        if last_viewed_cell.is_some() {
            // Despawn slots
            for slot_entity in slots_query.iter() {
                commands.entity(slot_entity).despawn();
            }
            *last_viewed_cell = None;
        }
        return;
    }

    let window = if let Ok(window) = window_query.single() {
        window
    } else {
        return;
    };

    // Only rebuild content when the viewed cell actually changes, not on every state change
    let Some(viewed_cell) = &cell_state.cell() else {
        return;
    };

    // Check if this is the same cell we're already displaying
    if *last_viewed_cell == Some(*viewed_cell) {
        return;
    }

    // Cell changed - update the display
    *last_viewed_cell = Some(*viewed_cell);

    // Get all occupied slots for the current cell
    let occupied_slots = units_cache.get_occupied_slots(viewed_cell);

    info!(
        "Occupied slots from cache for cell {:?}: {:?}",
        viewed_cell, occupied_slots
    );

    for container_entity in &container_query {
        commands.entity(container_entity).with_children(|parent| {
            // Create HexLayout for slot positioning (70.0 for 112px slots)
            info!("Spawning slots for cell: {:?}", viewed_cell);
            let slot_hex_layout = HexLayout::pointy().with_hex_size(70.0);
            let slot_image = asset_server.load("ui/ui_hex_normal.png");

            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    // height: Val::Px(window.height() - 64.), // Top bar size
                    // width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    top: Val::Px(64.),
                    left: Val::Px(0.),
                    right: Val::Px(0.),
                    bottom: Val::Px(0.),
                    margin: UiRect::all(Val::Px(10.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(50.),
                    ..default()
                },))
                .with_children(|container| {
                    if cell_state.has_interior() {
                        // Left exterior slots
                        container
                            .spawn((
                                Node {
                                    // width: Val::Px((window.width() - (window.height() - 64.)) / 2.),
                                    margin: UiRect::all(Val::Px(10.)),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(8.)),
                                ExteriorSlotContainer,
                            ))
                            .with_children(|exterior| {
                                let side = window.height() - 64. - 4. * 10.;
                                let container_size =
                                    Vec2::new((window.width() - side - 50. - 2. * 10.) / 2., side);
                                let exterior_positions = cell_state
                                    .slot_configuration()
                                    .exterior_layout
                                    .generate_positions(container_size, &slot_hex_layout);

                                // let offset = if matches!(
                                //     cell_state.slot_configuration().exterior_layout.layout_type,
                                //     SlotLayoutType::HexLine
                                // ) {
                                //     56.0
                                // } else {
                                //     0.0
                                // };
                                let offset = 56.0;

                                for (index, pos) in exterior_positions.iter().enumerate() {
                                    let slot_indicator =
                                        SlotIndicator::new(SlotPosition::exterior(index));
                                    let opacity = SlotState::Normal.get_opacity(false);

                                    exterior
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(pos.x - 56.0 - offset),
                                                top: Val::Px(pos.y - 65.0),
                                                width: Val::Px(112.0),
                                                height: Val::Px(130.0),
                                                ..default()
                                            },
                                            ImageNode {
                                                image: slot_image.clone(),
                                                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                ..default()
                                            },
                                            GlobalZIndex(0),
                                            slot_indicator,
                                        ))
                                        .observe(on_slot_drag_start)
                                        .observe(on_slot_drag)
                                        .observe(on_slot_drag_end)
                                        .observe(on_slot_drag_drop)
                                        .observe(on_slot_drag_enter)
                                        .observe(on_slot_drag_leave);
                                }
                            });

                        // Interior slots
                        container
                            .spawn((
                                Node {
                                    width: Val::Px(window.height() - 64.),
                                    margin: UiRect::all(Val::Px(10.)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(8.)),
                                InteriorSlotContainer,
                            ))
                            .with_children(|interior| {
                                let side = window.height() - 64. - 4. * 10.;
                                let container_size = Vec2::new(side, side);
                                let interior_positions = cell_state
                                    .slot_configuration()
                                    .interior_layout
                                    .generate_positions(container_size, &slot_hex_layout);

                                for (index, pos) in interior_positions.iter().enumerate() {
                                    let slot_indicator =
                                        SlotIndicator::new(SlotPosition::interior(index));
                                    let opacity = SlotState::Normal.get_opacity(false);

                                    interior
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(pos.x - 56.0),
                                                top: Val::Px(pos.y - 65.0),
                                                width: Val::Px(112.0),
                                                height: Val::Px(130.0),
                                                ..default()
                                            },
                                            ImageNode {
                                                image: slot_image.clone(),
                                                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                ..default()
                                            },
                                            GlobalZIndex(0),
                                            slot_indicator,
                                        ))
                                        .observe(on_slot_drag_start)
                                        .observe(on_slot_drag)
                                        .observe(on_slot_drag_end)
                                        .observe(on_slot_drag_drop)
                                        .observe(on_slot_drag_enter)
                                        .observe(on_slot_drag_leave);
                                }
                            });

                        // Right exterior slots
                        container
                            .spawn((
                                Node {
                                    // width: Val::Px((window.width() - (window.height() - 64.)) / 2.),
                                    margin: UiRect::all(Val::Px(10.)),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(8.)),
                                ExteriorSlotContainer,
                            ))
                            .with_children(|exterior| {
                                let side = window.height() - 64. - 4. * 10.;
                                let container_size =
                                    Vec2::new((window.width() - side - 50. - 2. * 10.) / 2., side);
                                let exterior_positions = cell_state
                                    .slot_configuration()
                                    .exterior_layout
                                    .generate_positions(container_size, &slot_hex_layout);

                                for (index, pos) in exterior_positions.iter().enumerate() {
                                    let slot_indicator = SlotIndicator::new(
                                        SlotPosition::exterior(index + exterior_positions.len()),
                                    );
                                    let opacity = SlotState::Normal.get_opacity(false);

                                    exterior
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(pos.x - 56.0),
                                                top: Val::Px(pos.y - 65.0),
                                                width: Val::Px(112.0),
                                                height: Val::Px(130.0),
                                                ..default()
                                            },
                                            ImageNode {
                                                image: slot_image.clone(),
                                                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                ..default()
                                            },
                                            GlobalZIndex(0),
                                            slot_indicator,
                                        ))
                                        .observe(on_slot_drag_start)
                                        .observe(on_slot_drag)
                                        .observe(on_slot_drag_end)
                                        .observe(on_slot_drag_drop)
                                        .observe(on_slot_drag_enter)
                                        .observe(on_slot_drag_leave);
                                }
                            });
                    } else {
                        // Exterior only cell
                        container
                            .spawn((
                                Node {
                                    // width: Val::Px((window.width() - (window.height() - 64.)) / 2.),
                                    margin: UiRect::all(Val::Px(10.)),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(8.)),
                                ExteriorSlotContainer,
                            ))
                            .with_children(|exterior| {
                                let container_size = Vec2::new(
                                    window.width() - 64. - 4. * 10.,
                                    window.height() - 64. - 4. * 10.,
                                );
                                let exterior_positions = cell_state
                                    .slot_configuration()
                                    .exterior_layout
                                    .generate_positions(container_size, &slot_hex_layout);

                                for (index, pos) in exterior_positions.iter().enumerate() {
                                    let slot_indicator =
                                        SlotIndicator::new(SlotPosition::exterior(index));

                                    let opacity = SlotState::Normal.get_opacity(false);

                                    exterior
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(pos.x - 56.0),
                                                top: Val::Px(pos.y - 65.0),
                                                width: Val::Px(112.0),
                                                height: Val::Px(130.0),
                                                ..default()
                                            },
                                            ImageNode {
                                                image: slot_image.clone(),
                                                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                                ..default()
                                            },
                                            GlobalZIndex(0),
                                            slot_indicator,
                                        ))
                                        .observe(on_slot_drag_start)
                                        .observe(on_slot_drag)
                                        .observe(on_slot_drag_end)
                                        .observe(on_slot_drag_drop)
                                        .observe(on_slot_drag_enter)
                                        .observe(on_slot_drag_leave);
                                }
                            });
                    }
                });
        });
    }
}

pub fn update_unit_portraits(
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    ui_state: Res<UIState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    mut commands: Commands,
    slot_query: Query<(Entity, &SlotIndicator)>,
    spawned_units_query: Query<(Entity, &SlotUnitPortrait)>,
    mut hex_mask_handle: Local<Option<Handle<Image>>>,
    mut pending_spawns: Local<HashSet<u64>>,
) {
    // info!("UPDATE UNIT PORTRAITS");
    if ui_state.panel_state != PanelEnum::CellView {
        return;
    }

    // Only rebuild content when the viewed cell actually changes, not on every state change
    let Some(viewed_cell) = &cell_state.cell() else {
        return;
    };

    // Get all occupied slots for the current cell
    let occupied_slots = units_cache.get_occupied_slots(viewed_cell);

    // Construire un set des unit_ids qui DOIVENT être présentes
    let expected_units: HashSet<u64> = occupied_slots.iter().map(|(_, id)| *id).collect();

    // Nettoyer le pending_spawns : si une unité pending est maintenant visible
    // dans la query, elle n'est plus "pending"
    pending_spawns.retain(|unit_id| {
        let now_exists = spawned_units_query
            .iter()
            .any(|(_, slot)| slot.unit_id == *unit_id);
        !now_exists // On garde seulement celles qui ne sont pas encore apparues
    });

    // Despawn uniquement les unités qui ne devraient plus être là
    // ET qui ne sont pas en cours de spawn
    for (entity, portrait) in spawned_units_query.iter() {
        if !expected_units.contains(&portrait.unit_id) {
            commands.entity(entity).despawn();
        }
    }

    // Spawning missing units
    for (slot_position, unit_id) in occupied_slots {
        // Déjà spawnée ?
        let already_spawned = spawned_units_query
            .iter()
            .any(|(_, slot)| slot.unit_id == unit_id);

        // En cours de spawn ?
        let is_pending = pending_spawns.contains(&unit_id);

        if already_spawned || is_pending {
            continue;
        }

        // Find the slot where to spawn the unit
        if let Some((slot_entity, slot_indicator)) = slot_query
            .iter()
            .find(|(_, slot_indicator)| slot_indicator.position == slot_position)
        {
            // Marquer comme "en transit" AVANT de lancer le spawn
            pending_spawns.insert(unit_id);

            // Get unit data to load the correct portrait
            let portrait_path = units_data_cache
                .get_unit(unit_id)
                .and_then(|unit_data| unit_data.avatar_url.clone())
                .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

            // Load hex mask once (cached in Local)
            if hex_mask_handle.is_none() {
                *hex_mask_handle = Some(asset_server.load("ui/ui_hex_mask.png"));
            }

            // Load the portrait image
            let portrait_handle: Handle<Image> = asset_server.load(portrait_path);
            let mask_handle = hex_mask_handle.clone().unwrap();

            commands
                .entity(slot_entity)
                .with_children(|slot_container| {
                    slot_container
                        .spawn((
                            InSlot(slot_entity), // Setting the relationship
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(112.0),
                                height: Val::Px(130.0),
                                ..default()
                            },
                            GlobalZIndex(10),
                            SlotUnitPortrait {
                                unit_id,
                                slot_position: slot_indicator.position,
                            },
                            Pickable::IGNORE,
                            // Pickable {
                            //     should_block_lower: false,
                            //     is_hoverable: false,
                            // },
                        ))
                        .with_children(|portrait| {
                            // 1. Portrait (will be masked with hex shape)
                            portrait.spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    ..default()
                                },
                                ImageNode {
                                    image: portrait_handle.clone(),
                                    ..default()
                                },
                                SlotUnitSprite {
                                    unit_id,
                                    slot_position: slot_indicator.position,
                                },
                                PendingHexMask {
                                    portrait_handle,
                                    mask_handle,
                                },
                                Pickable::IGNORE,
                                // Pickable {
                                //     should_block_lower: false,
                                //     is_hoverable: true,
                                // },
                            ));

                            // 2. Border overlay (hex _empty sprite on top of portrait)
                            let border_sprite_path = slot_indicator.state.get_sprite_path(true); // true = occupied
                            let opacity = slot_indicator.state.get_opacity(true);

                            portrait.spawn((
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
                                Pickable::IGNORE,
                                // Pickable {
                                //     should_block_lower: false,
                                //     is_hoverable: true,
                                // },
                            ));
                        });
                });

            info!(
                "Spawned portrait for unit {} with PendingHexMask and border overlay",
                unit_id
            );
        } else {
            warn!(
                "No valid slot corresponding to {:?} position for unit {}",
                slot_position, unit_id
            );
        }
    }
}

pub fn sync_slot_hierarchy_on_relation_change(
    mut commands: Commands,
    changed_units: Query<(Entity, &InSlot), Changed<InSlot>>,
    slot_query: Query<&SlotIndicator>,
    mut portrait_query: Query<&mut SlotUnitPortrait>,
    mut unit_transform_query: Query<&mut UiTransform>,
) {
    for (unit_entity, in_slot) in changed_units.iter() {
        info!("Entity {:?} changed slot", unit_entity);
        let new_slot_entity = in_slot.0;

        // Mettre à jour la hiérarchie UI
        commands
            .entity(unit_entity)
            .set_parent_in_place(new_slot_entity);

        // Mettre à jour le slot_position dans SlotUnitPortrait si nécessaire
        if let Ok(slot_indicator) = slot_query.get(new_slot_entity)
            && let Ok(mut portrait) = portrait_query.get_mut(unit_entity)
        {
            info!("New slot: {:?}", slot_indicator.position);
            portrait.slot_position = slot_indicator.position;
        }

        if let Ok(mut ui_transform) = unit_transform_query.get_mut(unit_entity) {
            ui_transform.translation = Val2::px(0.0, 0.0);
        }
    }
}

fn on_slot_drag_start(
    event: On<Pointer<DragStart>>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&SlotIndicator, Option<&SlotOccupant>)>,
    mut unit_style_query: Query<&mut GlobalZIndex>,
) {
    let slot_entity = event.event_target();

    let Ok((slot_indicator, maybe_occupant)) = slot_query.get(slot_entity) else {
        return;
    };

    let Some(occupant) = maybe_occupant else {
        return;
    };

    let unit_entity = occupant.get();

    drag_state.active = Some(DragInfo {
        source_slot: slot_entity,
        unit_entity,
        source_position: slot_indicator.position,
    });

    if let Ok(mut global_z_index) = unit_style_query.get_mut(unit_entity) {
        global_z_index.0 = 100;
    } else {
        warn!("Can't reset slot position");
    }
}

fn on_slot_drag(
    event: On<Pointer<Drag>>,
    drag_state: Res<DragState>,
    mut unit_style_query: Query<&mut UiTransform>,
) {
    let Some(drag_info) = &drag_state.active else {
        return;
    };

    if let Ok(mut ui_transform) = unit_style_query.get_mut(drag_info.unit_entity) {
        ui_transform.translation = Val2::px(event.distance.x, event.distance.y);
    }
}

fn on_slot_drag_end(
    mut _event: On<Pointer<DragEnd>>,
    mut drag_state: ResMut<DragState>,
    mut unit_style_query: Query<(&mut UiTransform, &mut GlobalZIndex)>,
) {
    let Some(drag_info) = drag_state.active.take() else {
        return;
    };

    let target_slot = drag_state.hovered_slot.take();

    if let Ok((_, mut global_z_index)) = unit_style_query.get_mut(drag_info.unit_entity) {
        global_z_index.0 = 10;
    } else {
        warn!("Can't reset slot position");
    }

    if target_slot.is_none()
        && let Ok((mut ui_transform, _)) = unit_style_query.get_mut(drag_info.unit_entity)
    {
        ui_transform.translation = Val2::px(0.0, 0.0);
    }
}

fn on_slot_drag_drop(
    mut event: On<Pointer<DragDrop>>,
    cell_state: Res<CellState>,
    mut drag_state: ResMut<DragState>,
    mut network_client: ResMut<NetworkClient>,
    mut slot_query: Query<(
        &SlotIndicator,
        Option<&SlotOccupant>,
        Option<&mut GlobalZIndex>,
        Option<&mut ImageNode>,
    )>,
    mut unit_style_query: Query<(&mut UiTransform, &mut GlobalZIndex), Without<SlotIndicator>>,
) {
    event.propagate(false);

    let Some(drag_info) = drag_state.active.take() else {
        return;
    };

    let target_slot = drag_state.hovered_slot.take();

    if let Ok((_, mut global_z_index)) = unit_style_query.get_mut(drag_info.unit_entity) {
        global_z_index.0 = 10;
    } else {
        warn!("Can't reset slot position");
    }

    if let Ok((slot_indicator, _, maybe_global_z_index, maybe_image)) =
        slot_query.get_mut(drag_info.source_slot)
        && let Some(mut global_z_index) = maybe_global_z_index
        && let Some(mut image) = maybe_image
    {
        global_z_index.0 = 0;
        let opacity = slot_indicator.state.get_opacity(false);
        image.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    } else {
        warn!("Can't reset slot z index");
    }

    if let Some(target) = target_slot
        && let Ok((slot_indicator, _, maybe_global_z_index, maybe_image)) =
            slot_query.get_mut(target)
        && let Some(mut global_z_index) = maybe_global_z_index
        && let Some(mut image) = maybe_image
    {
        global_z_index.0 = 0;
        let opacity = slot_indicator.state.get_opacity(false);
        image.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    } else {
        warn!("Can't reset slot z index");
    }

    // Tente le drop, retourne true si valide et message envoyé
    let drop_valid = (|| {
        let target_slot_entity = target_slot?;

        if target_slot_entity == drag_info.source_slot {
            return None;
        }

        let (source_slot_indicator, maybe_source_occupant, _, _) =
            slot_query.get(drag_info.source_slot).ok()?;
        maybe_source_occupant?;

        let (target_slot_indicator, maybe_target_occupant, _, _) =
            slot_query.get(target_slot_entity).ok()?;
        if maybe_target_occupant.is_some() {
            return None;
        }

        let viewed_cell = cell_state.cell()?;
        let unit_id = source_slot_indicator.occupied_by?;

        info!("Sending MoveUnitToSlot");
        network_client.send_message(ClientMessage::MoveUnitToSlot {
            unit_id,
            cell: viewed_cell,
            from_slot: source_slot_indicator.position,
            to_slot: target_slot_indicator.position,
        });

        Some(())
    })()
    .is_some();

    // Reset seulement si le drop est invalide
    if !drop_valid
        && let Ok((mut ui_transform, _)) = unit_style_query.get_mut(drag_info.unit_entity)
    {
        ui_transform.translation = Val2::px(0.0, 0.0);
    }
}

fn on_slot_drag_enter(
    event: On<Pointer<DragEnter>>,
    mut drag_state: ResMut<DragState>,
    mut slot_query: Query<(
        &SlotIndicator,
        &mut ImageNode,
        Option<&mut GlobalZIndex>,
        Option<&SlotOccupant>,
    )>,
) {
    info!("DRAG ENTER");
    if drag_state.active.is_none() {
        return;
    }

    let slot_entity = event.event_target();

    if let Ok((_, mut image, global_z_index, maybe_occupant)) = slot_query.get_mut(slot_entity) {
        drag_state.hovered_slot = Some(slot_entity);

        let is_valid_target = maybe_occupant.is_none();

        if let Some(mut z_index) = global_z_index
            && !is_valid_target
        {
            z_index.0 = 11;
        }
        image.color = if is_valid_target {
            Color::srgba(0.5, 1.0, 0.5, 0.5)
        } else {
            Color::srgba(1.0, 0.5, 0.5, 0.5)
        }
    }
}

fn on_slot_drag_leave(
    event: On<Pointer<DragLeave>>,
    mut drag_state: ResMut<DragState>,
    mut slot_query: Query<(&SlotIndicator, Option<&mut GlobalZIndex>, &mut ImageNode)>,
) {
    info!("DRAG LEAVE");
    let slot_entity = event.event_target();

    let Ok((slot_indicator, global_z_index, mut image)) = slot_query.get_mut(slot_entity) else {
        return;
    };

    if drag_state.hovered_slot != Some(slot_entity) {
        return;
    }

    drag_state.hovered_slot = None;

    if let Some(mut z_index) = global_z_index {
        z_index.0 = 0;
    }

    let opacity = slot_indicator.state.get_opacity(false);

    image.color = Color::srgba(1.0, 1.0, 1.0, opacity);
}
