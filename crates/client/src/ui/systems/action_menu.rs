use bevy::prelude::*;

use crate::state::resources::UnitsDataCache;
use crate::ui::{
    components::{ActionModeDisabled, ActionModeMenuButton, ActionModeMenuIcon, ActionModeTooltip},
    resources::{UIState, UnitSelectionState},
};

/// Update action mode button visuals (selected/hovered/normal/disabled).
pub fn update_action_menu_visual(
    asset_server: Res<AssetServer>,
    ui_state: Res<UIState>,
    menu_button_query: Query<(
        &ActionModeMenuButton,
        &mut ImageNode,
        Option<&ActionModeDisabled>,
    ), Without<ActionModeMenuIcon>>,
    mut menu_icon_query: Query<
        (&ActionModeMenuIcon, &mut ImageNode),
        Without<ActionModeMenuButton>,
    >,
) {
    let sub_tab_normal_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_normal.png");
    let sub_tab_selected_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_selected.png");
    let sub_tab_hovered_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_2_hovered.png");

    let disabled_tint = Color::srgba(0.5, 0.5, 0.5, 0.4);
    let disabled_icon_tint = Color::srgba(0.4, 0.4, 0.4, 0.4);

    for (menu_button, mut image_node, disabled) in menu_button_query {
        let is_disabled = disabled.is_some();

        if is_disabled {
            // Disabled: greyed out
            image_node.image = sub_tab_normal_image.clone();
            image_node.color = disabled_tint;

            if let Some((_icon, mut icon_image)) = menu_icon_query
                .iter_mut()
                .find(|(icon, _)| icon.action_mode == menu_button.action_mode)
            {
                icon_image.color = disabled_icon_tint;
            }
        } else if let Some(action_mode) = ui_state.action_mode
            && action_mode == menu_button.action_mode
        {
            // Selected mode
            image_node.image = sub_tab_selected_image.clone();
            image_node.color = Color::WHITE;

            if let Some((_icon, mut icon_image)) = menu_icon_query
                .iter_mut()
                .find(|(icon, _)| icon.action_mode == menu_button.action_mode)
            {
                icon_image.color = Color::srgb_u8(157, 136, 93);
            }
        } else {
            // Normal / hovered
            image_node.image = if let Some(hovered) = ui_state.hovered_action_mode
                && hovered == menu_button.action_mode
            {
                sub_tab_hovered_image.clone()
            } else {
                sub_tab_normal_image.clone()
            };
            image_node.color = Color::WHITE;

            if let Some((_icon, mut icon_image)) = menu_icon_query
                .iter_mut()
                .find(|(icon, _)| icon.action_mode == menu_button.action_mode)
            {
                icon_image.color = Color::srgb_u8(86, 73, 54);
            }
        }
    }
}

/// Update which action mode buttons are enabled/disabled based on selected units' professions.
pub fn update_action_mode_availability(
    mut commands: Commands,
    unit_selection: Res<UnitSelectionState>,
    units_data_cache: Res<UnitsDataCache>,
    mut ui_state: ResMut<UIState>,
    button_query: Query<(Entity, &ActionModeMenuButton, Option<&ActionModeDisabled>)>,
) {
    // Collect professions of all selected units
    let selected_professions: Vec<_> = unit_selection
        .selected_ids()
        .iter()
        .filter_map(|&uid| units_data_cache.get_unit(uid))
        .map(|u| u.profession)
        .collect();

    let has_selection = !selected_professions.is_empty();

    for (entity, menu_button, currently_disabled) in &button_query {
        let should_disable = if !has_selection {
            true
        } else {
            // Check if ANY selected unit has a profession compatible with this action mode
            !selected_professions
                .iter()
                .any(|prof| menu_button.action_mode.is_available_for(prof))
        };

        let is_disabled = currently_disabled.is_some();

        if should_disable && !is_disabled {
            commands.entity(entity).insert(ActionModeDisabled);
            // If the now-disabled mode was active, reset it
            if ui_state.action_mode == Some(menu_button.action_mode) {
                ui_state.reset_action_mode();
            }
        } else if !should_disable && is_disabled {
            commands.entity(entity).remove::<ActionModeDisabled>();
        }
    }
}

/// Show/hide tooltips on disabled buttons when hovered.
pub fn update_action_mode_tooltips(
    button_query: Query<
        (&Interaction, Option<&ActionModeDisabled>, &Children),
        With<ActionModeMenuButton>,
    >,
    mut tooltip_query: Query<&mut Visibility, With<ActionModeTooltip>>,
) {
    for (interaction, disabled, children) in &button_query {
        let show_tooltip = disabled.is_some() && matches!(interaction, Interaction::Hovered);

        for child in children.iter() {
            if let Ok(mut vis) = tooltip_query.get_mut(child) {
                let target = if show_tooltip {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }
    }
}
