use bevy::prelude::*;
use crate::ui::components::{SlotIndicator, SlotUnitSprite};
use crate::ui::resources::{CellViewState, UnitSelectionState};
use crate::ui::components::SlotBorderOverlay;
use crate::state::resources::UnitsCache;

// ─── Selection-aware colors ──────────────────────────────────

const SELECTED_BORDER_COLOR: Color = Color::srgba(0.4, 1.0, 0.4, 0.9);
const NORMAL_BORDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.75);
const SELECTED_PORTRAIT_TINT: Color = Color::srgba(0.7, 0.85, 1.0, 1.0);
const NORMAL_PORTRAIT_TINT: Color = Color::WHITE;

// ─── Helpers ────────────────────────────────────────────────

fn is_slot_unit_selected(
    cell: &shared::grid::GridCell,
    slot_pos: &shared::SlotPosition,
    units_cache: &UnitsCache,
    unit_selection: &UnitSelectionState,
) -> bool {
    units_cache
        .get_unit_at_slot(cell, slot_pos)
        .map(|uid| unit_selection.is_selected(uid))
        .unwrap_or(false)
}

// ─── Base slot hex image ────────────────────────────────────

/// Update the base hex slot image based on interaction state.
pub fn update_slot_visual_feedback(
    cell_view_state: Res<CellViewState>,
    mut slot_query: Query<(&Interaction, &SlotIndicator, &mut ImageNode), Changed<Interaction>>,
) {
    let is_dragging = cell_view_state.is_dragging();

    for (interaction, slot_indicator, mut image_node) in &mut slot_query {
        let is_occupied = slot_indicator.occupied_by.is_some();

        match interaction {
            Interaction::Hovered => {
                if is_dragging {
                    if is_occupied {
                        image_node.color = Color::srgba(1.0, 0.5, 0.5, 0.6);
                    } else {
                        image_node.color = Color::srgba(0.5, 1.0, 0.5, 0.6);
                    }
                } else {
                    let hover_opacity = slot_indicator.state.get_hover_opacity(slot_indicator.is_occupied());
                    image_node.color = Color::srgba(1.0, 1.0, 1.0, hover_opacity);
                }
            }
            Interaction::Pressed => {
                let opacity = slot_indicator.state.get_opacity(slot_indicator.is_occupied());
                image_node.color = Color::srgba(1.2, 1.2, 1.2, opacity.min(1.0));
            }
            Interaction::None => {
                let opacity = slot_indicator.state.get_opacity(slot_indicator.is_occupied());
                image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
            }
        };
    }
}

// ─── Border overlay (on top of portrait) ────────────────────

/// Update the border overlay color based on interaction state.
/// Selection-aware: in Interaction::None, uses green tint if unit is selected.
pub fn update_slot_overlay_visual_feedback(
    cell_view_state: Res<CellViewState>,
    unit_selection: Res<UnitSelectionState>,
    units_cache: Res<UnitsCache>,
    mut slot_query: Query<(&Interaction, &SlotBorderOverlay, &mut ImageNode), Changed<Interaction>>,
) {
    let is_dragging = cell_view_state.is_dragging();
    let viewed_cell = cell_view_state.viewed_cell;

    for (interaction, border, mut image_node) in &mut slot_query {
        let is_selected = viewed_cell
            .as_ref()
            .map(|cell| is_slot_unit_selected(cell, &border.slot_position, &units_cache, &unit_selection))
            .unwrap_or(false);

        match interaction {
            Interaction::Hovered => {
                if is_dragging {
                    image_node.color = Color::srgba(1.0, 0.5, 0.5, 0.6);
                } else {
                    image_node.color = Color::srgba(1.0, 1.0, 1.0, 0.9);
                }
            }
            Interaction::Pressed => {
                image_node.color = Color::srgba(1.2, 1.2, 1.2, 1.0);
            }
            Interaction::None => {
                image_node.color = if is_selected {
                    SELECTED_BORDER_COLOR
                } else {
                    NORMAL_BORDER_COLOR
                };
            }
        };
    }
}

/// Refresh border overlays every frame for non-hovered/pressed slots.
/// Ensures selection state is always visually in sync, including on state entry.
pub fn refresh_overlay_on_selection_change(
    cell_view_state: Res<CellViewState>,
    unit_selection: Res<UnitSelectionState>,
    units_cache: Res<UnitsCache>,
    mut border_query: Query<(&Interaction, &SlotBorderOverlay, &mut ImageNode)>,
) {
    let Some(viewed_cell) = cell_view_state.viewed_cell else {
        return;
    };

    for (interaction, border, mut image_node) in &mut border_query {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            continue;
        }

        let is_selected = is_slot_unit_selected(
            &viewed_cell,
            &border.slot_position,
            &units_cache,
            &unit_selection,
        );

        let target = if is_selected {
            SELECTED_BORDER_COLOR
        } else {
            NORMAL_BORDER_COLOR
        };

        // Only write if different to avoid unnecessary change detection
        if image_node.color != target {
            image_node.color = target;
        }
    }
}

// ─── Portrait sprite tint ───────────────────────────────────

/// Apply a light blue tint on portrait sprites of selected units.
/// Runs every frame to ensure consistency on state entry.
pub fn update_unit_selection_portrait_tint(
    unit_selection: Res<UnitSelectionState>,
    mut sprite_query: Query<(&SlotUnitSprite, &mut ImageNode)>,
) {
    for (sprite, mut image_node) in &mut sprite_query {
        let target = if unit_selection.is_selected(sprite.unit_id) {
            SELECTED_PORTRAIT_TINT
        } else {
            NORMAL_PORTRAIT_TINT
        };

        if image_node.color != target {
            image_node.color = target;
        }
    }
}
