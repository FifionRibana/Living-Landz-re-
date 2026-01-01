use bevy::prelude::*;

use crate::ui::{
    components::{ActionModeMenuButton, ActionModeMenuIcon},
    resources::UIState,
};

pub fn update_action_menu_visual(
    asset_server: Res<AssetServer>,
    ui_state: Res<UIState>,
    menu_button_query: Query<(&ActionModeMenuButton, &mut ImageNode), Without<ActionModeMenuIcon>>,
    mut menu_icon_query: Query<
        (&ActionModeMenuIcon, &mut ImageNode),
        Without<ActionModeMenuButton>,
    >,
) {
    let sub_tab_normal_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_normal.png");
    let sub_tab_selected_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_selected.png");
    let sub_tab_hovered_image: Handle<Image> =
        asset_server.load("ui/ui_sub_top_bar_button_hovered.png");

    for (menu_button, mut image_node) in menu_button_query {
        // Selected mode
        if let Some(action_mode) = ui_state.action_mode
            && action_mode == menu_button.action_mode
        {
            image_node.image = sub_tab_selected_image.clone();
            if let Some((_action_icon, mut icon_image_node)) = menu_icon_query
                .iter_mut()
                .find(|(icon, _)| icon.action_mode == menu_button.action_mode)
            {
                icon_image_node.color = Color::srgb_u8(157, 136, 93);
            }
        // Unselected mode
        } else {
            image_node.image = if let Some(hovered_action_mode) = ui_state.hovered_action_mode
                && hovered_action_mode == menu_button.action_mode
            {
                sub_tab_hovered_image.clone()
            } else {
                sub_tab_normal_image.clone()
            };

            if let Some((_action_icon, mut icon_image_node)) = menu_icon_query
                .iter_mut()
                .find(|(icon, _)| icon.action_mode == menu_button.action_mode)
            {
                icon_image_node.color = Color::srgb_u8(86, 73, 54);
            }
        }
    }
}
