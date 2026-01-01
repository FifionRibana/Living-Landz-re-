use bevy::prelude::*;

use crate::ui::{
    components::{ActionMenuMarker, PanelContainer},
    resources::{CellState, PanelEnum, UIState},
};

pub fn update_panel_visibility(
    ui_state: Res<UIState>,
    mut cell_state: ResMut<CellState>,
    panel_query: Query<(&mut Visibility, &PanelContainer), Without<ActionMenuMarker>>,
    mut action_menu_query: Query<(&mut Visibility, &ActionMenuMarker), Without<PanelContainer>>,
    mut previous_panel: Local<Option<PanelEnum>>,
) {
    let panel_changed = *previous_panel != Some(ui_state.panel_state);

    if !panel_changed {
        return;
    }

    if ui_state.panel_state != PanelEnum::CellView && *previous_panel == Some(PanelEnum::CellView) {
        info!("Exit cell view");
        cell_state.exit_view();
    }

    for (mut visibility, container) in panel_query {
        *visibility = Visibility::Hidden;

        if container.panel == ui_state.panel_state {
            *visibility = Visibility::Visible;
        }
    }
    action_menu_query
        .iter_mut()
        .for_each(|(mut action_menu_visibility, _)| {
            *action_menu_visibility = Visibility::Hidden;
        });

    if matches!(
        ui_state.panel_state,
        PanelEnum::MapView | PanelEnum::SearchView
    ) {
        action_menu_query
            .iter_mut()
            .for_each(|(mut action_menu_visibility, _)| {
                *action_menu_visibility = Visibility::Visible;
            });
        // continue;
    }

    *previous_panel = Some(ui_state.panel_state);
}
