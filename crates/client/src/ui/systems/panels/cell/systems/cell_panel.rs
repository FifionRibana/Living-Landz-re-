use std::collections::HashSet;

use bevy::{prelude::*, window::PrimaryWindow};
use hexx::{Rect, *};
use shared::{SlotPosition, protocol::ClientMessage};

use crate::camera;
use crate::camera::resources::CELL_SCENE_LAYER;
use crate::state::resources::UnitWorkState;
use crate::ui::components::{
    CellSceneSlotSprite, CellSceneVisual, DragTargetValidity, Slot, SlotVisualState,
};
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
                        container.spawn((
                            Node {
                                // width: Val::Px(window.height() - 64.),
                                margin: UiRect::all(Val::Px(10.)),
                                flex_grow: 1.0,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            ExteriorSlotContainer,
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));

                        // Interior slots
                        container.spawn((
                            Node {
                                width: Val::Px(window.height() - 64.),
                                margin: UiRect::all(Val::Px(10.)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            InteriorSlotContainer,
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));

                        // Right exterior slots
                        container.spawn((
                            Node {
                                // width: Val::Px(window.height() - 64.),
                                margin: UiRect::all(Val::Px(10.)),
                                flex_grow: 1.0,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            ExteriorSlotContainer,
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                    } else {
                        // Exterior only cell
                        container.spawn((
                            Node {
                                // width: Val::Px(window.height() - 64.),
                                margin: UiRect::all(Val::Px(10.)),
                                flex_grow: 1.0,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            ExteriorSlotContainer,
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                    }
                });
        });
    }

    // Create HexLayout for slot positioning (70.0 for 112px slots)
    let slot_hex_layout = HexLayout::pointy().with_hex_size(70.0);
    let slot_image = asset_server.load("ui/ui_hex_normal.png");
    info!("Spawning slots for cell: {:?}", viewed_cell);

    let screen_w = window.width();
    let screen_h = window.height();
    let top_bar = 64.0;
    let margin = 10.0;
    let gap = 50.0;
    let side = window.height() - top_bar - 4. * margin;
    info!(
        "Values:\n\tSide={}\n\tWindow={}x{}",
        side,
        window.width(),
        window.height()
    );

    // Compute the real position. For this, we need to consider multiple things.
    // We compute the size of panels (without margins)
    // The following sketch does not include margins.
    // - sides + interior are inside a container that has 10px margins.
    // - sides & interior also have 10px margins.
    // - gap between sides & interior is 50px.
    // - top_bar is 64px high.
    //  ________ _ ________________ _ ________
    // |                top_bar               |
    // |________ _ ________________ _ ________|
    // |        | |                | |        |
    // |        | |                | |        |
    // |  Left  | |    Interior    | |  Right |
    // |  side  | |                | |  side  |
    // |        | |                | |        |
    // |        | |                | |        |
    // |________|_|________________|_|________|
    //
    // | 10px | SIDE | 50px | INTERIOR | 50px | SIDE | 10px |
    // ------
    // TOP_BAR:
    // width = SCREEN_W
    // height = 64px
    // x = 0
    // y = 0
    let top_bar_box = bevy::math::Rect::new(0.0, 0.0, screen_w, top_bar);
    // MAIN_CONTAINER_CONTENT:
    // width = screen_w - 2 * margin
    // height = screen_h - TOP_BAR.height - 2 * margin
    // x = margin
    // y = TOP_BAR.bottom + margin
    // left = margin / right = screen_w - margin
    // top = TOP_BAR.bottom + margin / bottom = screen_h - margin
    let main_container_content = bevy::math::Rect::new(
        margin,
        top_bar_box.max.y + margin,
        screen_w - margin,
        screen_h - margin,
    );
    // ------
    // main container margin: 10px
    // ------
    // INTERIOR_CONTAINER
    // ------
    // main container margin: 10px
    // ------
    let containers_height = main_container_content.height() - 2. * margin;
    let containers_content_height = containers_height - 2. * margin;
    //
    // INTERIOR_CONTAINER:
    // height = MAIN_CONTAINER_CONTENT.height - 2 * margin
    // width = height
    //
    let remaining_space =
        main_container_content.width() - containers_height - 2. * gap - 2. * margin;
    // SIDE_CONTAINER(S):
    // width => Remaining space divided by 2 when removing interior container + twice the gap.
    // width = (SCREEN_W - INTERIOR_CONTAINER.width - 2 * gap - 2 * margin) / 2.
    // height = INTERIOR_CONTAINER.height
    let side_container_width = remaining_space / 2.;
    //
    // LEFT:
    // x = margin
    // y = TOP_BAR.height + margin
    let left_side_container = bevy::math::Rect::new(
        main_container_content.min.x + margin,
        main_container_content.min.y + margin,
        main_container_content.min.x + margin + side_container_width,
        main_container_content.max.y - margin,
    );
    //
    // SIDE_CONTAINER_CONTENT(S):
    // width = SIDE_CONTAINER_CONTENT.width - 2 * margin
    // height = SIDE_CONTAINER_CONTENT.height - 2 * margin
    //
    // LEFT:
    // x = LEFT_SIDE_CONTAINER.x + margin
    // y = LEFT_SIDE_CONTAINER.y + margin
    let left_side_container_content = bevy::math::Rect::new(
        left_side_container.min.x + margin,
        left_side_container.min.y + margin,
        left_side_container.max.x - margin,
        left_side_container.max.y - margin,
    );
    info!("Left container: {:?}", left_side_container_content);
    //
    // INTERIOR_CONTAINER:
    // x = SIDE_CONTAINER.right + gap
    // y = TOP_BAR.height + margin
    let interior_container = bevy::math::Rect::new(
        left_side_container.max.x + gap,
        left_side_container.min.y,
        left_side_container.max.x + gap + containers_height,
        left_side_container.max.y,
    );
    //
    // INTERIOR_CONTAINER_CONTENT: INTERIOR_CONTAINER - margins
    // width = INTERIOR_CONTAINER.width - 2 * margin
    // height = INTERIOR_CONTAINER.height - 2 * margin
    // x = INTERIOR_CONTAINER.x + margin
    // y = INTERIOR_CONTAINER.y + margin
    let interior_container_content = bevy::math::Rect::new(
        interior_container.min.x + margin,
        interior_container.min.y + margin,
        interior_container.max.x - margin,
        interior_container.max.y - margin,
    );
    info!("Interior container: {:?}", interior_container_content);
    //
    // RIGHT:
    // x = INTERIOR_CONTAINER.right + gap
    // y = TOP_BAR.height + margin
    let right_side_container = bevy::math::Rect::new(
        interior_container.max.x + gap,
        left_side_container.min.y,
        interior_container.max.x + gap + side_container_width,
        left_side_container.max.y,
    );
    //
    // SIDE_CONTAINER_CONTENT(S):
    // width = SIDE_CONTAINER_CONTENT.width - 2 * margin
    // height = SIDE_CONTAINER_CONTENT.height - 2 * margin
    //
    // RIGHT:
    // x = RIGHT_SIDE_CONTAINER.x + margin
    // y = RIGHT_SIDE_CONTAINER.y + margin
    let right_side_container_content = bevy::math::Rect::new(
        right_side_container.min.x + margin,
        right_side_container.min.y + margin,
        right_side_container.max.x - margin,
        right_side_container.max.y - margin,
    );
    info!("Right container: {:?}", right_side_container_content);
    // Note: Sprite are placed on world position (camera position) whereas UI nodes are on screen position
    //       a conversion is required here.

    // => ext = window_h - 64px - 8*10px
    if cell_state.has_interior() {
        // left exterior slots
        let exterior_positions = cell_state
            .slot_configuration()
            .exterior_layout
            .generate_positions(left_side_container_content.size(), &slot_hex_layout);
        let offset = 56.0; // Short offset as the slot position generation seems weird for linear layouts

        for (index, pos) in exterior_positions.iter().enumerate() {
            let slot_indicator = SlotIndicator::new(SlotPosition::exterior(index));
            let opacity = SlotState::Normal.get_opacity(false);

            info!(
                "Spawn ext l.slot: {} | pos {:?} | container {:?} (center: {:?})",
                index,
                pos,
                left_side_container_content,
                left_side_container_content.center()
            );

            let real_pos = Vec2::new(
                left_side_container_content.min.x + pos.x - offset,
                left_side_container_content.min.y + pos.y,
            );
            info!("  > Pos: {:?}", real_pos);
            commands
                .spawn((
                    Sprite {
                        image: slot_image.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, opacity),
                        ..default()
                    },
                    Transform::from_translation(
                        camera::utils::screen_to_world(real_pos.x, real_pos.y, screen_w, screen_h)
                            .extend(5.0),
                    ),
                    Pickable::default(),
                    slot_indicator,
                    Slot,
                    SlotVisualState::default(),
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

        // Interior slots
        let interior_positions = cell_state
            .slot_configuration()
            .interior_layout
            .generate_positions(interior_container_content.size(), &slot_hex_layout);

        for (index, pos) in interior_positions.iter().enumerate() {
            let slot_indicator = SlotIndicator::new(SlotPosition::interior(index));
            let opacity = SlotState::Normal.get_opacity(false);

            let real_pos = Vec2::new(
                interior_container_content.min.x + pos.x,
                interior_container_content.min.y + pos.y, // - 32.0,
            );
            commands
                .spawn((
                    Sprite {
                        image: slot_image.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, opacity),
                        ..default()
                    },
                    Transform::from_translation(
                        camera::utils::screen_to_world(real_pos.x, real_pos.y, screen_w, screen_h)
                            .extend(5.0),
                    ),
                    // Transform::from_translation(Vec3::new(real_pos.x, -real_pos.y, 5.0)),
                    Pickable::default(),
                    slot_indicator,
                    Slot,
                    SlotVisualState::default(),
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

        // Right exterior slots
        let exterior_positions = cell_state
            .slot_configuration()
            .exterior_layout
            .generate_positions(right_side_container_content.size(), &slot_hex_layout);

        for (index, pos) in exterior_positions.iter().enumerate() {
            let slot_indicator = SlotIndicator::new(SlotPosition::exterior(index));
            let opacity = SlotState::Normal.get_opacity(false);

            info!(
                "Spawn ext r.slot: {} | pos {:?} | container {:?} (center: {:?})",
                index,
                pos,
                right_side_container_content,
                right_side_container_content.center()
            );

            let real_pos = Vec2::new(
                right_side_container_content.min.x + pos.x, // + window.width() / 2.,
                right_side_container_content.min.y + pos.y, // - 32.0,
            );
            info!("  > Pos: {:?}", real_pos);
            commands
                .spawn((
                    Sprite {
                        image: slot_image.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, opacity),
                        ..default()
                    },
                    Transform::from_translation(
                        camera::utils::screen_to_world(real_pos.x, real_pos.y, screen_w, screen_h)
                            .extend(5.0),
                    ),
                    // Transform::from_translation(Vec3::new(real_pos.x, -real_pos.y, 5.0)),
                    Pickable::default(),
                    slot_indicator,
                    Slot,
                    SlotVisualState::default(),
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
    } else {
        // Exterior cells only
        let exterior_positions = cell_state
            .slot_configuration()
            .exterior_layout
            .generate_positions(main_container_content.size(), &slot_hex_layout);

        for (index, pos) in exterior_positions.iter().enumerate() {
            let slot_indicator = SlotIndicator::new(SlotPosition::exterior(index));
            let opacity = SlotState::Normal.get_opacity(false);

            let real_pos = Vec2::new(
                main_container_content.min.x + pos.x,
                main_container_content.min.y + pos.y, // - 32.0,
            );
            commands
                .spawn((
                    Sprite {
                        image: slot_image.clone(),
                        custom_size: Some(Vec2::new(112.0, 130.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, opacity),
                        ..default()
                    },
                    Transform::from_translation(
                        camera::utils::screen_to_world(real_pos.x, real_pos.y, screen_w, screen_h)
                            .extend(5.0),
                    ),
                    // Transform::from_translation(Vec3::new(real_pos.x, -real_pos.y, 5.0)),
                    Pickable::default(),
                    slot_indicator,
                    Slot,
                    SlotVisualState::default(),
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
    changed_units: Query<(Entity, &InSlot, &Children), Changed<InSlot>>,
    slot_query: Query<(&SlotIndicator, &Transform), Without<SlotUnitPortrait>>,
    mut portrait_query: Query<&mut SlotUnitPortrait>,
    mut border_query: Query<&mut SlotBorderOverlay>,
    mut transform_query: Query<&mut Transform, Without<SlotIndicator>>,
) {
    for (unit_entity, in_slot, children) in changed_units.iter() {
        info!("Entity {:?} changed slot", unit_entity);
        let new_slot_entity = in_slot.0;

        if let Ok((slot_indicator, slot_transform)) = slot_query.get(new_slot_entity) {
            // Update container position
            if let Ok(mut transform) = transform_query.get_mut(unit_entity) {
                transform.translation.x = slot_transform.translation.x;
                transform.translation.y = slot_transform.translation.y;
                transform.translation.z = 10.0;
            }

            // Update portrait slot_position
            if let Ok(mut portrait) = portrait_query.get_mut(unit_entity) {
                info!("New slot: {:?}", slot_indicator.position);
                portrait.slot_position = slot_indicator.position;
            }

            // Update border slot_position (child of container)
            for child in children.iter() {
                if let Ok(mut border) = border_query.get_mut(child) {
                    border.slot_position = slot_indicator.position;
                }
            }
        }
    }
}

pub fn sync_slot_visuals(
    drag_state: Res<DragState>,
    unit_selection: Res<UnitSelectionState>,
    cell_state: Res<CellState>,
    units_cache: Res<UnitsCache>,
    unit_work_state: Res<UnitWorkState>,
    // Slot hex sprites
    mut slot_query: Query<(&SlotIndicator, &SlotVisualState, &mut Sprite), With<Slot>>,
    // Unit portrait sprites
    mut portrait_query: Query<(&SlotUnitSprite, &mut Sprite), Without<Slot>>,
    // Border overlay sprites
    mut border_query: Query<
        (&SlotBorderOverlay, &mut Sprite),
        (Without<Slot>, Without<SlotUnitSprite>),
    >,
    container_query: Query<&SlotUnitPortrait>,
) {
    let Some(viewed_cell) = cell_state.cell() else {
        return;
    };

    let is_dragging = drag_state.active.is_some();

    // ── Slot hex visuals ──
    for (indicator, vis_state, mut sprite) in slot_query.iter_mut() {
        sprite.color = if let Some(validity) = vis_state.drag_target {
            // Drag target feedback — highest priority
            match validity {
                DragTargetValidity::Valid => Color::srgba_u8(128, 255, 128, 128),
                DragTargetValidity::Invalid => Color::srgba_u8(255, 128, 128, 128),
            }
        } else {
            // Normal opacity based on occupied state
            let opacity = indicator.state.get_opacity(false); //indicator.is_occupied());
            Color::srgba(1.0, 1.0, 1.0, opacity)
        };
    }

    let dragged_unit_id = drag_state.active.as_ref().and_then(|d| {
        // Find the unit_id of the dragged container
        container_query.get(d.unit_entity).ok().map(|p| p.unit_id)
    });

    // ── Portrait visuals ──
    for (unit_sprite, mut sprite) in portrait_query.iter_mut() {
        let drag_target = slot_query.iter().find_map(|(ind, vis, _)| {
            if ind.occupied_by == Some(unit_sprite.unit_id) {
                vis.drag_target
            } else {
                None
            }
        });

        let is_selected = unit_selection.is_selected(unit_sprite.unit_id);
        let is_working = unit_work_state.is_working(unit_sprite.unit_id);
        let is_being_dragged = dragged_unit_id == Some(unit_sprite.unit_id);

        // TODO: When changing slot, the border is not "selected anymore if the unit is selected
        // Find if this unit's slot is hovered
        let is_hovered = slot_query
            .iter()
            .any(|(ind, vis, _)| ind.occupied_by == Some(unit_sprite.unit_id) && vis.hovered);

        sprite.color = if is_being_dragged {
            Color::WHITE
        } else if drag_target == Some(DragTargetValidity::Invalid) {
            Color::srgba_u8(255, 128, 128, 196)
        } else if is_working {
            Color::srgb_u8(255, 200, 100) // warm orange tint — busy
        } else if is_hovered && !is_dragging {
            Color::srgb_u8(217, 230, 196) // hover highlight
        } else if is_selected {
            Color::srgb_u8(179, 217, 196) // selection tint
        } else {
            Color::WHITE
        };
    }

    // ── Border visuals ──
    for (border, mut sprite) in border_query.iter_mut() {
        let drag_target = slot_query.iter().find_map(|(ind, vis, _)| {
            if ind.position == border.slot_position && ind.occupied_by.is_some() {
                vis.drag_target
            } else {
                None
            }
        });

        let border_unit_id = units_cache.get_unit_at_slot(&viewed_cell, &border.slot_position);
        let is_being_dragged = border_unit_id.is_some() && border_unit_id == dragged_unit_id;

        let is_selected = units_cache
            .get_unit_at_slot(&viewed_cell, &border.slot_position)
            .map(|uid| unit_selection.is_selected(uid))
            .unwrap_or(false);

        sprite.color = if is_being_dragged {
            Color::srgba_u8(255, 255, 255, 191) // normal
        } else if drag_target == Some(DragTargetValidity::Invalid) {
            Color::srgba_u8(255, 128, 128, 204) // red border
        } else if is_selected {
            Color::srgba_u8(179, 217, 255, 230) // selection tint
        } else {
            Color::srgba_u8(255, 255, 255, 191) // normal
        };
    }
}

fn on_slot_hover_enter(
    event: On<Pointer<Over>>,
    mut query: Query<&mut SlotVisualState, With<Slot>>,
) {
    if let Ok(mut state) = query.get_mut(event.event_target()) {
        state.hovered = true;
    }
}

fn on_slot_hover_leave(
    event: On<Pointer<Out>>,
    mut query: Query<&mut SlotVisualState, With<Slot>>,
) {
    if let Ok(mut state) = query.get_mut(event.event_target()) {
        state.hovered = false;
    }
}

fn on_slot_drag_enter(
    event: On<Pointer<DragEnter>>,
    mut drag_state: ResMut<DragState>,
    query: Query<&SlotIndicator, With<Slot>>,
    mut vis_query: Query<&mut SlotVisualState, With<Slot>>,
) {
    if drag_state.active.is_none() {
        return;
    }
    let entity = event.event_target();
    drag_state.hovered_slot = Some(entity);

    if let Ok(indicator) = query.get(entity)
        && let Ok(mut state) = vis_query.get_mut(entity)
    {
        state.drag_target = Some(if indicator.occupied_by.is_none() {
            DragTargetValidity::Valid
        } else {
            DragTargetValidity::Invalid
        });
    }
}

fn on_slot_drag_leave(
    event: On<Pointer<DragLeave>>,
    mut drag_state: ResMut<DragState>,
    mut vis_query: Query<&mut SlotVisualState, With<Slot>>,
) {
    let entity = event.event_target();
    if let Ok(mut state) = vis_query.get_mut(entity) {
        state.drag_target = None;
    }
    if drag_state.hovered_slot == Some(entity) {
        drag_state.hovered_slot = None;
    }
}

fn on_slot_drag_start(
    event: On<Pointer<DragStart>>,
    mut drag_state: ResMut<DragState>,
    unit_work_state: Res<UnitWorkState>,
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

    //? For now it is as is, but tbd if it is the desired behavior.
    //? Perhaps you can drag but can't move the unit outside
    // Get unit_id from the portrait
    // Can't drag a working unit
    if let Some(unit_id) = slot_indicator.occupied_by {
        if unit_work_state.is_working(unit_id) {
            info!("Unit {} is working, can't drag", unit_id);
            return;
        }
    }

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

    let _target_slot = drag_state.hovered_slot.take();

    if let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity) {
        transform.translation.x = drag_info.origin.x;
        transform.translation.y = drag_info.origin.y;
        transform.translation.z = 10.0;
    } else {
        warn!("Can't reset slot position");
    }

    // if target_slot.is_none()
    //     && let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity)
    // {
    //     transform.translation.x = drag_info.origin.x;
    //     transform.translation.y = drag_info.origin.y;
    //     transform.translation.z = 10.0;
    // }
}
fn on_slot_drag_drop(
    mut event: On<Pointer<DragDrop>>,
    cell_state: Res<CellState>,
    mut drag_state: ResMut<DragState>,
    mut network_client: ResMut<NetworkClient>,
    slot_query: Query<(&SlotIndicator, Option<&SlotOccupant>), With<Slot>>,
    mut container_query: Query<&mut Transform, With<SlotUnitPortrait>>,
    mut vis_query: Query<&mut SlotVisualState, With<Slot>>,
) {
    event.propagate(false);

    let Some(drag_info) = drag_state.active.take() else {
        return;
    };

    let drop_target = event.event_target();
    drag_state.hovered_slot = None;

    // Reset all visual states — sync_slot_visuals handles the rest
    for mut state in vis_query.iter_mut() {
        state.drag_target = None;
        state.hovered = false;
    }

    // Try the drop
    let drop_valid = (|| {
        if drop_target == drag_info.source_slot {
            return None;
        }

        let (_, maybe_source_occupant) = slot_query.get(drag_info.source_slot).ok()?;
        maybe_source_occupant?;

        let (target_indicator, maybe_target_occupant) = slot_query.get(drop_target).ok()?;
        if maybe_target_occupant.is_some() {
            return None;
        }

        let viewed_cell = cell_state.cell()?;
        let (source_indicator, _) = slot_query.get(drag_info.source_slot).ok()?;
        let unit_id = source_indicator.occupied_by?;

        info!("Sending MoveUnitToSlot");
        network_client.send_message(ClientMessage::MoveUnitToSlot {
            unit_id,
            cell: viewed_cell,
            from_slot: source_indicator.position,
            to_slot: target_indicator.position,
        });

        Some(())
    })()
    .is_some();

    // Reset container position if drop invalid
    if let Ok(mut transform) = container_query.get_mut(drag_info.unit_entity) {
        // Whatever happens, reset the z position to 10. (a drop can happen outside of any slot)
        transform.translation.z = 10.0;

        if !drop_valid {
            transform.translation.x = drag_info.origin.x;
            transform.translation.y = drag_info.origin.y;
        }
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
