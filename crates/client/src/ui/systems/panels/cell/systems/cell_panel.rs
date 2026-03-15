use std::collections::HashSet;

use bevy::{prelude::*, window::PrimaryWindow};
use hexx::*;
use shared::{SlotPosition, protocol::ClientMessage};

use crate::camera::resources::CELL_SCENE_LAYER;
use crate::ui::components::{CellSceneSlotSprite, CellSceneVisual, Slot};
use crate::ui::resources::CellViewState;
use crate::{
    networking::client::NetworkClient,
    state::resources::{UnitsCache, UnitsDataCache},
    ui::{
        components::{
            ExteriorSlotContainer, InteriorSlotContainer, PendingHexMask, PendingLayerComposition,
            SlotBorderOverlay, SlotIndicator, SlotState, SlotUnitPortrait, SlotUnitSprite,
        },
        resources::{CellState, DragInfo, DragState, UnitSelectionState},
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
    units_cache: Res<UnitsCache>,
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewPanel>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    slots_query: Query<Entity, With<SlotIndicator>>,
) {
    let window = if let Ok(window) = window_query.single() {
        window
    } else {
        return;
    };

    let Some(viewed_cell) = &cell_state.cell() else {
        return;
    };

    // Despawn existing slots (needed when switching cells while in CellView)
    for slot_entity in slots_query.iter() {
        commands.entity(slot_entity).despawn();
    }

    // Get all occupied slots for the current cell
    let occupied_slots = units_cache.get_occupied_slots(viewed_cell);

    info!(
        "Occupied slots from cache for cell {:?}: {:?}",
        viewed_cell, occupied_slots
    );

    for container_entity in &container_query {
        commands.entity(container_entity).with_children(|parent| {
            parent
                .spawn((
                    Node {
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
                    },
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ))
                .with_children(|container| {
                    if cell_state.has_interior() {
                        // Left exterior slots
                        /*container
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
                                        Sprite {
                                            image: slot_image.clone(),
                                            custom_size: Some(Vec2::new(112.0, 130.0)),
                                            color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                            ..default()
                                        },
                                        Transform::from_translation(Vec3::new(
                                            pos.x - 56.0 - offset,
                                            pos.y - 65.0,
                                            5.0,
                                        )),
                                        Pickable::default(),
                                        slot_indicator,
                                        CellSceneVisual,
                                        CELL_SCENE_LAYER,
                                    ))
                                    .observe(on_slot_drag_start)
                                    .observe(on_slot_drag)
                                    .observe(on_slot_drag_end)
                                    .observe(on_slot_drag_drop)
                                    .observe(on_slot_drag_enter)
                                    .observe(on_slot_drag_leave)
                                    .observe(on_slot_click);
                            }
                        });*/

                        // Interior slots
                        container.spawn((
                            Node {
                                width: Val::Px(window.height() - 64.),
                                margin: UiRect::all(Val::Px(10.)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            BorderRadius::all(Val::Px(8.)),
                            InteriorSlotContainer,
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                        // .with_children(|interior| {

                        // });

                        // Right exterior slots
                        /*container
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
                                        Sprite {
                                            image: slot_image.clone(),
                                            custom_size: Some(Vec2::new(112.0, 130.0)),
                                            color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                            ..default()
                                        },
                                        Transform::from_translation(Vec3::new(
                                            pos.x - 56.0,
                                            pos.y - 65.0,
                                            5.0,
                                        )),
                                        Pickable::default(),
                                        slot_indicator,
                                        CellSceneVisual,
                                        CELL_SCENE_LAYER,
                                    ))
                                    .observe(on_slot_drag_start)
                                    .observe(on_slot_drag)
                                    .observe(on_slot_drag_end)
                                    .observe(on_slot_drag_drop)
                                    .observe(on_slot_drag_enter)
                                    .observe(on_slot_drag_leave)
                                    .observe(on_slot_click);
                            }
                        });*/
                    } else {
                        // Exterior only cell
                        /*container
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
                                        Sprite {
                                            image: slot_image.clone(),
                                            custom_size: Some(Vec2::new(112.0, 130.0)),
                                            color: Color::srgba(1.0, 1.0, 1.0, opacity),
                                            ..default()
                                        },
                                        Transform::from_translation(Vec3::new(
                                            pos.x - 56.0,
                                            pos.y - 65.0,
                                            5.0,
                                        )),
                                        Pickable::default(),
                                        slot_indicator,
                                        CellSceneVisual,
                                        CELL_SCENE_LAYER,
                                    ))
                                    .observe(on_slot_drag_start)
                                    .observe(on_slot_drag)
                                    .observe(on_slot_drag_end)
                                    .observe(on_slot_drag_drop)
                                    .observe(on_slot_drag_enter)
                                    .observe(on_slot_drag_leave)
                                    .observe(on_slot_click);
                            }
                        });*/
                    }
                });
        });
    }

    // Create HexLayout for slot positioning (70.0 for 112px slots)
    let slot_hex_layout = HexLayout::pointy().with_hex_size(70.0);
    let slot_image = asset_server.load("ui/ui_hex_normal.png");
    info!("Spawning slots for cell: {:?}", viewed_cell);

    let side = window.height() - 64. - 4. * 10.;
    let container_size = Vec2::new(side, side);
    let interior_positions = cell_state
        .slot_configuration()
        .interior_layout
        .generate_positions(container_size, &slot_hex_layout);

    info!(
        "Values:\n\tSide={}\n\tWindow={}x{}",
        side,
        window.width(),
        window.height()
    );

    // commands.spawn(
    //     (Node {
    //         width: Val::Px(container_size.x),
    //         height: Val::Px(container_size.y),
    //         position_type: PositionType::Absolute,

    //         ..Default::default() },
    //     BackgroundColor(Color::srgba_u8(255, 0, 0, 50)),
    //     UiTransform::from_translation(Val2::px(window.width() / 2. - side / 2., 64.0 + 2. * 10.)),
    // ));
    for (index, pos) in interior_positions.iter().enumerate() {
        let slot_indicator = SlotIndicator::new(SlotPosition::interior(index));
        let opacity = SlotState::Normal.get_opacity(false);

        info!(
            "Slot {}: Spawning {:?} at pos {:?}",
            index, slot_indicator, pos
        );
        // Compute the real position. For this, we need to consider multiple things.
        //  ________ _ ________________ _ ________
        // |        | |                | |        |
        // |        | |                | |        |
        // |  Left  | |    Interior    | |  Right |
        // |  side  | |                | |  side  |
        // |        | |                | |        |
        // |        | |                | |        |
        // |________|_|________________|_|________|
        //
        let real_pos = Vec2::new(
            container_size.x / 2. - pos.x,
            container_size.y / 2. - pos.y - 32.0,
        );
        commands
            .spawn((
                Sprite {
                    image: slot_image.clone(),
                    custom_size: Some(Vec2::new(112.0, 130.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, opacity),
                    ..default()
                },
                Transform::from_translation(Vec3::new(real_pos.x, real_pos.y, 5.0)),
                Pickable::default(),
                slot_indicator,
                Slot,
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ))
            .observe(on_slot_hover_enter)
            .observe(on_slot_hover_leave)
            .observe(on_slot_drag_start)
            .observe(on_slot_drag)
            .observe(on_slot_drag_end)
            .observe(on_slot_drag_drop)
            .observe(on_slot_drag_enter)
            .observe(on_slot_drag_leave)
            .observe(on_slot_click);
    }
}

pub fn update_unit_portraits(
    asset_server: Res<AssetServer>,
    cell_state: Res<CellState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    mut commands: Commands,
    slot_query: Query<(Entity, &SlotIndicator, &Transform), Without<SlotUnitPortrait>>,
    spawned_units_query: Query<(Entity, &SlotUnitPortrait)>,
    mut hex_mask_handle: Local<Option<Handle<Image>>>,
    mut pending_spawns: Local<HashSet<u64>>,
) {
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
        spawned_units_query
            .iter()
            .any(|(_, slot)| slot.unit_id == *unit_id)
            .then_some(())
            .is_none()
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
        let Some((slot_entity, slot_indicator, slot_transform)) = slot_query
            .iter()
            .find(|(_, slot_indicator, _)| slot_indicator.position == slot_position)
        else {
            warn!(
                "No valid slot corresponding to {:?} position for unit {}",
                slot_position, unit_id
            );
            continue;
        };

        // Marquer comme "en transit" AVANT de lancer le spawn
        pending_spawns.insert(unit_id);

        // Get unit data to load the correct portrait
        let unit_data = units_data_cache.get_unit(unit_id);
        let layers = unit_data.and_then(|u| u.parse_portrait_layers());

        // Load hex mask once (cached in Local)
        if hex_mask_handle.is_none() {
            *hex_mask_handle = Some(asset_server.load("ui/ui_hex_mask.png"));
        }
        let mask_handle = hex_mask_handle.clone().unwrap();

        let container_pos = Vec3::new(
            slot_transform.translation.x,
            slot_transform.translation.y,
            10.,
        );

        // ── Container (holds portrait + border) ──
        let container = commands
            .spawn((
                // Sprite::default(),
                Transform::from_translation(container_pos),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                SlotUnitPortrait {
                    unit_id,
                    slot_position: slot_indicator.position,
                },
                InSlot(slot_entity),
                Pickable::IGNORE,
                CellSceneVisual,
                CELL_SCENE_LAYER,
            ))
            .id();

        // 1. Portrait (will be masked with hex shape)
        let portrait = if let Some([bust, face, clothes, hair]) = layers {
            // LORD — composite 4 layers
            let layer_handles = [
                asset_server.load(format!(
                    "sprites/character/layers/bust/bust_{:02}.png",
                    bust + 1
                )),
                asset_server.load(format!(
                    "sprites/character/layers/face/face_{:02}.png",
                    face + 1
                )),
                asset_server.load(format!(
                    "sprites/character/layers/clothes/clothes_{:02}.png",
                    clothes + 1
                )),
                asset_server.load(format!(
                    "sprites/character/layers/hair/hair_{:02}.png",
                    hair + 1
                )),
            ];

            commands
                .spawn((
                    Sprite {
                        image: mask_handle.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0), // invisible until composed
                        ..default()
                    },
                    Transform::from_translation(Vec3::ZERO),
                    SlotUnitSprite {
                        unit_id,
                        slot_position: slot_indicator.position,
                    },
                    PendingLayerComposition {
                        layer_handles,
                        mask_handle: Some(mask_handle.clone()),
                    },
                    Pickable::IGNORE,
                    CELL_SCENE_LAYER,
                ))
                .id()
        } else {
            // NPC — single avatar_url
            let portrait_path = unit_data
                .and_then(|u| u.avatar_url.clone())
                .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());
            let portrait_handle: Handle<Image> = asset_server.load(portrait_path);

            commands
                .spawn((
                    Sprite {
                        image: portrait_handle.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::ZERO),
                    SlotUnitSprite {
                        unit_id,
                        slot_position: slot_indicator.position,
                    },
                    PendingHexMask {
                        portrait_handle,
                        mask_handle: mask_handle.clone(),
                    },
                    Pickable::IGNORE,
                    CELL_SCENE_LAYER,
                ))
                .id()
        };

        // 2. Border overlay (hex _empty sprite on top of portrait)
        let border_sprite_path = slot_indicator.state.get_sprite_path(true); // true = occupied
        let border_opacity = slot_indicator.state.get_opacity(true);

        let border = commands
            .spawn((
                Sprite {
                    image: asset_server.load(&border_sprite_path),
                    custom_size: Some(Vec2::new(112.0, 130.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, border_opacity),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                SlotBorderOverlay {
                    slot_position: slot_indicator.position,
                },
                Pickable::IGNORE,
                CELL_SCENE_LAYER,
            ))
            .id();

        // Parent-child
        commands.entity(container).add_child(portrait);
        commands.entity(container).add_child(border);

        info!(
            "Spawned portrait for unit {} with PendingHexMask and border overlay",
            unit_id
        );
    }
}

pub fn sync_slot_hierarchy_on_relation_change(
    changed_units: Query<(Entity, &InSlot), Changed<InSlot>>,
    slot_query: Query<(&SlotIndicator, &Transform), With<Slot>>,
    mut portrait_query: Query<(&mut SlotUnitPortrait, &mut Transform), Without<Slot>>,
    // mut transform_query: Query<&mut Transform>,
) {
    for (unit_entity, in_slot) in changed_units.iter() {
        info!("Entity {:?} changed slot", unit_entity);
        let new_slot_entity = in_slot.0;

        // Mettre à jour le slot_position dans SlotUnitPortrait si nécessaire
        if let Ok((slot_indicator, slot_transform)) = slot_query.get(new_slot_entity)
            && let Ok((mut portrait, mut transform)) = portrait_query.get_mut(unit_entity)
        {
            info!("New slot: {:?}", slot_indicator.position);
            portrait.slot_position = slot_indicator.position;
            transform.translation.x = slot_transform.translation.x;
            transform.translation.y = slot_transform.translation.y;
            transform.translation.z = 10.0;
        }
    }
}

fn on_slot_hover_enter(
    event: On<Pointer<Over>>,
    drag_state: Res<DragState>,
    mut slot_query: Query<(&SlotIndicator, &mut Sprite, &mut Transform), With<Slot>>,
    mut portrait_sprite_query: Query<(&SlotUnitPortrait, &mut Transform), Without<Slot>>,
) {
    let slot_entity = event.event_target();
    let Ok((indicator, mut slot_sprite, mut slot_transform)) = slot_query.get_mut(slot_entity)
    else {
        return;
    };

    let is_occupied = indicator.occupied_by.is_some();
    let is_dragging = drag_state.active.is_some();

    info!(
        "On hover: slot {:?} occupied: {:?} / is dragging: {:?}",
        slot_entity, is_occupied, is_dragging
    );

    if is_dragging {
        slot_sprite.color = if is_occupied {
            // Force update z to 12 to ensure that when is_dragging the color is correctly displayed
            info!("  Dragging and occupied");
            slot_transform.translation.z = 12.;
            Color::srgba(1.0, 0.5, 0.5, 0.6) // red = invalid drop target
        } else {
            info!("  Dragging and empty");
            Color::srgba(0.5, 1.0, 0.5, 0.6) // green = valid drop target
        };
        if let Some(unit_id) = indicator.occupied_by {
            for (unit_sprite, mut transform) in portrait_sprite_query.iter_mut() {
                if unit_sprite.unit_id == unit_id {
                    transform.translation.z = 10.;
                    info!(
                        "Slot {} / portrait {}",
                        slot_transform.translation.z, transform.translation.z
                    );
                    //             sprite.color = Color::srgb(1.0, 0.0, 0.0);
                    break;
                }
            }
        }
    } else if is_occupied {
        info!("  Simply occupied");
        // Force update z to 5 to ensure it is ALWAYS below the portrait.
        slot_transform.translation.z = 5.;

        // Reset color and opacity to default
        let hover_opacity = indicator.state.get_hover_opacity(true);
        slot_sprite.color = Color::srgba(1.0, 1.0, 1.0, hover_opacity);

        if let Some(unit_id) = indicator.occupied_by {
            for (unit_sprite, mut transform) in portrait_sprite_query.iter_mut() {
                if unit_sprite.unit_id == unit_id {
                    transform.translation.z = 10.;
                    info!(
                        "Slot {} / portrait {}",
                        slot_transform.translation.z, transform.translation.z
                    );
                    //             sprite.color = Color::srgb(1.0, 0.0, 0.0);
                    break;
                }
            }
        }
    }
}

fn on_slot_hover_leave(
    event: On<Pointer<Out>>,
    drag_state: Res<DragState>,
    mut slot_query: Query<(&SlotIndicator, &mut Sprite, &mut Transform), With<Slot>>,
    mut portrait_sprite_query: Query<(&SlotUnitSprite, &mut Sprite), Without<Slot>>,
) {
    let slot_entity = event.event_target();
    let Ok((indicator, mut slot_sprite, mut slot_transform)) = slot_query.get_mut(slot_entity)
    else {
        return;
    };

    info!(
        "On leave: slot {:?} occupied: {:?} / is dragging: {:?}",
        slot_entity,
        indicator.occupied_by.is_some(),
        drag_state.active.is_some()
    );

    // Force z = 5 to ensure when leaving it is always below the portrait
    slot_transform.translation.z = 5.;
    // Reset color and opacity to default
    let opacity = indicator.state.get_opacity(indicator.is_occupied());
    slot_sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);

    // if let Some(unit_id) = indicator.occupied_by {
    //     for (unit_sprite, mut sprite) in portrait_sprite_query.iter_mut() {
    //         if unit_sprite.unit_id == unit_id {
    //             sprite.color = Color::WHITE;
    //             break;
    //         }
    //     }
    // }
}

fn on_slot_drag_start(
    event: On<Pointer<DragStart>>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&SlotIndicator, Option<&SlotOccupant>), With<Slot>>,
    container_query: Query<&Transform, With<SlotUnitPortrait>>,
) {
    let slot_entity = event.event_target();

    let Ok((slot_indicator, maybe_occupant)) = slot_query.get(slot_entity) else {
        return;
    };

    let Some(occupant) = maybe_occupant else {
        return;
    };

    let unit_entity = occupant.get();

    let origin = container_query
        .get(unit_entity)
        .map(|t| t.translation.truncate())
        .unwrap_or_default();

    drag_state.active = Some(DragInfo {
        source_slot: slot_entity,
        unit_entity,
        source_position: slot_indicator.position,
        origin,
    });
}

fn on_slot_drag(
    event: On<Pointer<Drag>>,
    drag_state: Res<DragState>,
    mut container_query: Query<&mut Transform, With<SlotUnitPortrait>>,
) {
    let Some(drag_info) = &drag_state.active else {
        return;
    };

    if let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity) {
        transform.translation.x = drag_info.origin.x + event.distance.x;
        transform.translation.y = drag_info.origin.y - event.distance.y;
        transform.translation.z = 100.0;
    }
}

fn on_slot_drag_end(
    _event: On<Pointer<DragEnd>>,
    mut drag_state: ResMut<DragState>,
    mut container_query: Query<&mut Transform, With<SlotUnitPortrait>>,
) {
    let Some(drag_info) = drag_state.active.take() else {
        return;
    };

    let target_slot = drag_state.hovered_slot.take();

    if let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity) {
        transform.translation.x = drag_info.origin.x;
        transform.translation.y = drag_info.origin.y;
        transform.translation.z = 10.0;
    } else {
        warn!("Can't reset slot position");
    }

    if target_slot.is_none()
        && let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity)
    {
        transform.translation.x = drag_info.origin.x;
        transform.translation.y = drag_info.origin.y;
        transform.translation.z = 10.0;
    }
}

fn on_slot_drag_drop(
    mut event: On<Pointer<DragDrop>>,
    cell_state: Res<CellState>,
    mut drag_state: ResMut<DragState>,
    mut network_client: ResMut<NetworkClient>,
    mut slot_query: Query<
        (
            &SlotIndicator,
            Option<&SlotOccupant>,
            &mut Sprite,
            &mut Transform,
        ),
        With<Slot>,
    >,
    mut container_query: Query<&mut Transform, (With<SlotUnitPortrait>, Without<Slot>)>,
) {
    event.propagate(false);

    let Some(drag_info) = drag_state.active.take() else {
        return;
    };

    let target_slot = drag_state.hovered_slot.take();

    // Reset source slot visual + z
    if let Ok((indicator, _, mut sprite, mut transform)) = slot_query.get_mut(drag_info.source_slot)
    {
        transform.translation.z = 5.0;
        info!(
            "Reset source {:?} slot to {}",
            drag_info.source_slot, transform.translation.z
        );
        let opacity = indicator.state.get_opacity(false);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    }

    // Reset target slot visual + z
    if let Some(target) = target_slot
        && let Ok((indicator, _, mut sprite, mut transform)) = slot_query.get_mut(target)
    {
        transform.translation.z = 5.0;
        info!(
            "Reset source {:?} slot to {}",
            target, transform.translation.z
        );
        let opacity = indicator.state.get_opacity(false);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);
    }

    // Try the drop
    let drop_valid = (|| {
        let target_slot_entity = target_slot?;

        if target_slot_entity == drag_info.source_slot {
            return None;
        }

        let (_, maybe_source_occupant, _, _) = slot_query.get(drag_info.source_slot).ok()?;
        maybe_source_occupant?;

        let (target_slot_indicator, maybe_target_occupant, _, _) =
            slot_query.get(target_slot_entity).ok()?;
        if maybe_target_occupant.is_some() {
            return None;
        }

        let viewed_cell = cell_state.cell()?;
        let (source_slot_indicator, _, _, _) = slot_query.get(drag_info.source_slot).ok()?;
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

    // Reset container position if drop invalid
    if let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity) {
        // Whatever happens, reset container z to 10. after the drop
        transform.translation.z = 10.0;
        info!(
            "Reset portrait {:?} to {}",
            drag_info.unit_entity, transform.translation.z
        );

        // If drop is invalid, then reset the container's position to the origin
        if !drop_valid {
            transform.translation.x = drag_info.origin.x;
            transform.translation.y = drag_info.origin.y;
        }
    }
}

fn on_slot_drag_enter(
    event: On<Pointer<DragEnter>>,
    mut drag_state: ResMut<DragState>,
    mut slot_query: Query<(&SlotIndicator, &mut Sprite, &mut Transform), With<Slot>>,
) {
    info!("DRAG ENTER");
    if drag_state.active.is_none() {
        return;
    }

    let slot_entity = event.event_target();

    if let Ok((indicator, mut sprite, mut transform)) = slot_query.get_mut(slot_entity) {
        drag_state.hovered_slot = Some(slot_entity);

        let is_valid_target = indicator.occupied_by.is_none();
        info!(
            "On DRAG target {:?} validity: {:?}",
            slot_entity, is_valid_target
        );

        transform.translation.z = 12.;

        sprite.color = if is_valid_target {
            Color::srgba(0.5, 1.0, 0.5, 0.5)
        } else {
            Color::srgba(1.0, 0.5, 0.5, 0.5)
        }
    }
}

fn on_slot_drag_leave(
    event: On<Pointer<DragLeave>>,
    mut drag_state: ResMut<DragState>,
    mut slot_query: Query<(&SlotIndicator, &mut Transform, &mut Sprite), With<Slot>>,
) {
    info!("DRAG LEAVE");
    let slot_entity = event.event_target();

    let Ok((slot_indicator, mut transform, mut sprite)) = slot_query.get_mut(slot_entity) else {
        return;
    };

    info!(
        "On drag leave target {:?} validity {:?}",
        slot_entity,
        slot_indicator.occupied_by.is_some()
    );

    // Always reset visual + z, regardless of hovered_slot state
    transform.translation.z = 5.0;
    let opacity = slot_indicator.state.get_opacity(false);
    sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);

    // Only clear hovered_slot if it's still us
    if drag_state.hovered_slot != Some(slot_entity) {
        drag_state.hovered_slot = None;
    }
}

/// Handle click on a slot — select/deselect the unit occupying it.
/// `Pointer<Click>` fires only when press+release happens without significant
/// movement, so it won't interfere with drag operations.
fn on_slot_click(
    _event: On<Pointer<Click>>,
    slot_query: Query<&SlotIndicator>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut unit_selection: ResMut<UnitSelectionState>,
) {
    info!("CLICK");
    let slot_entity = _event.event_target();
    let Ok(slot_indicator) = slot_query.get(slot_entity) else {
        return;
    };

    info!("slot indicator {:?}", slot_indicator);

    let Some(unit_id) = slot_indicator.occupied_by else {
        // Empty slot clicked — clear selection
        unit_selection.clear();
        return;
    };

    info!("unit id {}", unit_id);

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl {
        unit_selection.toggle(unit_id);
    } else {
        unit_selection.select(unit_id);
    }
}
