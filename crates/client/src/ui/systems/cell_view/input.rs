use crate::ui::components::CellViewBackButton;
use crate::states::GameView;
use bevy::prelude::*;

/// Handle cell view exit via back button click.
/// ESC is handled globally by handle_escape_key in plugin.rs.
pub fn handle_cell_view_back_button(
    mut next_view: ResMut<NextState<GameView>>,
    back_button_query: Query<&Interaction, (Changed<Interaction>, With<CellViewBackButton>)>,
) {
    let should_exit = back_button_query
        .iter()
        .any(|i| matches!(i, Interaction::Pressed));

    if should_exit {
        info!("Exiting cell view via back button");
        next_view.set(GameView::Map);
    }
}
