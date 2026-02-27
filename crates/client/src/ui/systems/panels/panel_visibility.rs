use bevy::prelude::*;

use crate::states::GameView;
use crate::ui::components::ActionMenuMarker;

/// Show action menu only on Map and Search views.
pub fn update_action_menu_visibility(
    game_view: Option<Res<State<GameView>>>,
    mut action_menu_query: Query<&mut Visibility, With<ActionMenuMarker>>,
) {
    let Some(gv) = game_view else { return };
    let should_show = matches!(gv.get(), GameView::Map | GameView::Search);
    let new_vis = if should_show { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in &mut action_menu_query {
        if *vis != new_vis {
            *vis = new_vis;
        }
    }
}
