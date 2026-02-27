use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::states::AppState;
use crate::ui::{
    components::{ActionMenuMarker, PanelContainer, TopBarMarker},
    resources::{CellState, PanelEnum, UIState},
};

pub fn update_top_bar_visibility(
    app_state: Res<State<AppState>>,
    mut top_bar_query: Query<(&mut Visibility, &TopBarMarker)>,
) {
    let Ok((mut visibility, _)) = top_bar_query.single_mut() else {
        return;
    };

    let should_be_visible = *app_state.get() == AppState::InGame;

    let new_visibility = if should_be_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    if *visibility != new_visibility {
        *visibility = new_visibility;
    }
}

pub fn update_panel_visibility(
    ui_state: Res<UIState>,
    mut cell_state: ResMut<CellState>,
    panel_query: Query<(&mut Visibility, &PanelContainer), Without<ActionMenuMarker>>,
    mut action_menu_query: Query<(&mut Visibility, &ActionMenuMarker), Without<PanelContainer>>,
    mut previous_panel: Local<Option<PanelEnum>>,
    mut input_focus: ResMut<InputFocus>,
) {
    let panel_changed = *previous_panel != Some(ui_state.panel_state);

    if !panel_changed {
        return;
    }

    // Clear any text input focus when switching panels to avoid blocking camera/keyboard input
    input_focus.0 = None;

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