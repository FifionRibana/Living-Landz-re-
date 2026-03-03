use bevy::prelude::*;
use shared::{ActionViewContext, BiomeTypeEnum};

use crate::state::resources::UnitsDataCache;
use crate::states::GameView;
use crate::ui::resources::{ActionContextState, CellState, UnitSelectionState};

/// Recompute the ActionContextState every frame based on game state.
pub fn compute_action_context(
    game_view: Res<State<GameView>>,
    unit_selection: Res<UnitSelectionState>,
    units_data_cache: Res<UnitsDataCache>,
    cell_state: Res<CellState>,
    mut action_context: ResMut<ActionContextState>,
) {
    if !unit_selection.has_selection() {
        if action_context.get().is_some() {
            action_context.clear();
        }
        return;
    }

    // Determine view context
    let view = match game_view.get() {
        GameView::Cell => ActionViewContext::Cell,
        _ => ActionViewContext::Map,
    };

    // Collect professions of selected units
    let professions: Vec<_> = unit_selection
        .selected_ids()
        .iter()
        .filter_map(|&uid| units_data_cache.get_unit(uid))
        .map(|u| u.profession)
        .collect();

    // Get building and terrain from CellState
    let (building, terrain) = if view == ActionViewContext::Cell {
        let building = cell_state
            .building_data
            .as_ref()
            .and_then(|bd| bd.to_building_type());
        let terrain = cell_state.biome();
        (building, terrain)
    } else {
        (None, BiomeTypeEnum::Undefined)
    };

    // TODO: Check adjacent roads from road data when available
    let has_adjacent_road = false;

    action_context.update(view, building, terrain, professions, has_adjacent_road);
}
