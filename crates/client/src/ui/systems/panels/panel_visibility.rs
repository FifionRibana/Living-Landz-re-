use bevy::prelude::*;

use crate::states::GameView;
use crate::ui::components::ActionMenuMarker;
use crate::ui::resources::UnitSelectionState;

/// Show action menu on Map/Search views, or whenever units are selected.
pub fn update_action_menu_visibility(
    game_view: Option<Res<State<GameView>>>,
    unit_selection: Res<UnitSelectionState>,
    mut action_menu_query: Query<&mut Visibility, With<ActionMenuMarker>>,
) {
    let Some(gv) = game_view else { return };
    let in_map_or_search = matches!(gv.get(), GameView::Map | GameView::Search);
    let has_selection = unit_selection.has_selection();
    let should_show = in_map_or_search || has_selection;
    let new_vis = if should_show { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in &mut action_menu_query {
        if *vis != new_vis {
            *vis = new_vis;
        }
    }
}
