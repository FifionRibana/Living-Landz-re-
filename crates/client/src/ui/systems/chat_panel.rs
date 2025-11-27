use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputQueue, actions::TextInputAction};

use crate::{
    grid::resources::SelectedHexes,
    ui::{
        components::{ChatIconButton, ChatInputContainer, ChatInputField, ChatMessagesContainer, ChatNotificationBadge, ChatNotificationBadgeText, ChatPanelMarker, ChatSendButton, ChatToggleButton},
        resources::ChatState,
    },
};

pub fn handle_chat_send_button(
    mut query: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<ChatSendButton>)>,
    mut input_query: Query<(&TextInputBuffer, &mut TextInputQueue), With<ChatInputField>>,
) {
    for (mut background_color, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 70, 50));

                // Send message and trigger submit (which will clear if clear_on_submit is true)
                if let Ok((buffer, mut queue)) = input_query.single_mut() {
                    let text = buffer.get_text();
                    if !text.is_empty() {
                        info!("Sending chat message: {}", text);
                        // TODO: Actually send the message to the server
                        queue.add(TextInputAction::Submit);
                    }
                }
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

pub fn handle_chat_toggle_button(
    mut query: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<ChatToggleButton>)>,
    mut chat_state: ResMut<ChatState>,
) {
    for (mut background_color, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 70, 50));
                chat_state.toggle();
                info!("Chat toggled: expanded = {}", chat_state.is_expanded);
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

pub fn update_chat_visibility(
    chat_state: Res<ChatState>,
    selected_hexes: Res<SelectedHexes>,
    mut panel_query: Query<&mut Visibility, (With<ChatPanelMarker>, Without<ChatIconButton>)>,
    mut icon_query: Query<&mut Visibility, (With<ChatIconButton>, Without<ChatPanelMarker>)>,
    mut panel_node_query: Query<&mut Node, (With<ChatPanelMarker>, Without<ChatIconButton>)>,
    mut icon_node_query: Query<&mut Node, (With<ChatIconButton>, Without<ChatPanelMarker>)>,
    mut messages_query: Query<&mut Visibility, (With<ChatMessagesContainer>, Without<ChatInputContainer>, Without<ChatPanelMarker>, Without<ChatIconButton>)>,
    mut input_query: Query<&mut Visibility, (With<ChatInputContainer>, Without<ChatMessagesContainer>, Without<ChatPanelMarker>, Without<ChatIconButton>)>,
) {
    let is_actions_panel_visible = !selected_hexes.ids.is_empty();

    if chat_state.is_changed() || selected_hexes.is_changed() {
        // Toggle between panel and icon
        for mut panel_visibility in panel_query.iter_mut() {
            *panel_visibility = if chat_state.is_expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        for mut icon_visibility in icon_query.iter_mut() {
            *icon_visibility = if chat_state.is_expanded {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }

        // Adjust panel position based on actions panel visibility
        for mut node in panel_node_query.iter_mut() {
            if is_actions_panel_visible {
                node.bottom = px(160.);
            } else {
                node.bottom = px(0.);
            }
        }

        // Adjust icon position based on actions panel visibility
        for mut node in icon_node_query.iter_mut() {
            if is_actions_panel_visible {
                node.bottom = px(170.);
            } else {
                node.bottom = px(10.);
            }
        }

        for mut visibility in messages_query.iter_mut() {
            *visibility = if chat_state.is_expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        for mut visibility in input_query.iter_mut() {
            *visibility = if chat_state.is_expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn handle_chat_icon_button(
    mut query: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<ChatIconButton>)>,
    mut chat_state: ResMut<ChatState>,
) {
    for (mut background_color, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8));
                chat_state.toggle();
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            }
        }
    }
}

pub fn update_chat_notification_badge(
    chat_state: Res<ChatState>,
    mut badge_query: Query<(&mut Visibility, &Children), With<ChatNotificationBadge>>,
    mut text_query: Query<&mut Text, With<ChatNotificationBadgeText>>,
) {
    if chat_state.is_changed() {
        for (mut visibility, children) in badge_query.iter_mut() {
            if chat_state.unread_messages > 0 && !chat_state.is_expanded {
                *visibility = Visibility::Visible;
                // Update badge text
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = chat_state.unread_messages.to_string();
                    }
                }
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
