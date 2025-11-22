use bevy::prelude::*;

use crate::ui::{
    components::{ActionButtonMarker, ActionsPanelMarker},
    resources::ChatState,
};

pub fn handle_action_button_interactions(
    mut query: Query<
        (&ActionButtonMarker, &mut BackgroundColor, &Interaction),
        Changed<Interaction>,
    >,
) {
    for (action_button, mut background_color, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 70, 50));
                info!("Action button pressed: {}", action_button.action_type);
                // TODO: Trigger the actual action
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb_u8(120, 110, 90));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb_u8(100, 90, 70));
            }
        }
    }
}

pub fn update_actions_panel_layout(
    chat_state: Res<ChatState>,
    mut panel_query: Query<&mut Node, With<ActionsPanelMarker>>,
) {
    if chat_state.is_changed() {
        for mut node in panel_query.iter_mut() {
            if chat_state.is_expanded {
                // When chat is expanded, make room for it (350px + 10px margin on each side)
                node.left = px(370.);
                node.width = Val::Percent(100.0);
                node.right = px(0.);
            } else {
                // When chat is collapsed, use full width
                node.left = px(0.);
                node.width = Val::Percent(100.0);
                node.right = px(0.);
            }
        }
    }
}
