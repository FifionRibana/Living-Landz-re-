use bevy::prelude::*;
use crate::ui::components::{SlotIndicator, SlotUnitSprite};
use crate::ui::resources::CellViewState;
use crate::state::resources::{UnitsCache, UnitsDataCache};

/// Update unit sprites displayed on slots
pub fn update_unit_sprites(
    mut commands: Commands,
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
    asset_server: Res<AssetServer>,
    // Query existing unit sprites to remove them
    existing_sprites: Query<(Entity, &SlotUnitSprite)>,
    // Query slot indicators to attach sprites to them
    slot_query: Query<(Entity, &SlotIndicator)>,
    // Track the last viewed cell to detect when we switch to a different cell
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
    // Track if we need to retry on the next frame (when UI isn't ready yet)
    mut needs_retry: Local<bool>,
) {
    // Only update when cell view is active
    if !cell_view_state.is_active {
        *last_viewed_cell = None;
        *needs_retry = false;
        return;
    }

    // Don't update during drag & drop to avoid flickering
    if cell_view_state.dragging_unit.is_some() {
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
        occupied_slots.len(), slot_count, viewed_cell.q, viewed_cell.r
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

                // Spawn unit sprite as child of the slot button
                commands.entity(slot_entity).with_children(|parent| {
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load(portrait_path),
                            ..default()
                        },
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        SlotUnitSprite {
                            unit_id: *unit_id,
                            slot_position: slot_indicator.position,
                        },
                    ));
                });
            }
        }
    }
}

/// Update SlotIndicator occupied_by field based on UnitsCache
pub fn update_slot_occupancy(
    cell_view_state: Res<CellViewState>,
    units_cache: Res<UnitsCache>,
    mut slot_query: Query<&mut SlotIndicator>,
) {
    // Only update when cell view is active and changed
    if !cell_view_state.is_active {
        return;
    }

    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    // Update each slot indicator
    for mut slot_indicator in &mut slot_query {
        slot_indicator.occupied_by = units_cache.get_unit_at_slot(&viewed_cell, &slot_indicator.position);
    }
}
