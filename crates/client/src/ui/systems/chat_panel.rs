use bevy::{prelude::*, input::keyboard::{Key, KeyboardInput}};

use crate::{
    grid::resources::SelectedHexes,
    ui::{
        components::{ChatInputContainer, ChatInputField, ChatInputState, ChatInputText, ChatMessagesContainer, ChatPanelMarker, ChatSendButton, ChatToggleButton},
        resources::ChatState,
    },
};

pub fn handle_chat_send_button(
    mut query: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<ChatSendButton>)>,
    mut input_query: Query<&mut ChatInputState, With<ChatInputField>>,
) {
    for (mut background_color, interaction) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb_u8(80, 70, 50));

                // Send message and clear input
                if let Ok(mut input_state) = input_query.single_mut() {
                    if !input_state.text.is_empty() {
                        info!("Sending chat message: {}", input_state.text);
                        // TODO: Actually send the message to the server
                        input_state.text.clear();
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
    mut panel_query: Query<&mut Node, With<ChatPanelMarker>>,
    mut messages_query: Query<&mut Visibility, (With<ChatMessagesContainer>, Without<ChatInputContainer>)>,
    mut input_query: Query<&mut Visibility, (With<ChatInputContainer>, Without<ChatMessagesContainer>)>,
) {
    let is_actions_panel_visible = !selected_hexes.ids.is_empty();

    if chat_state.is_changed() || selected_hexes.is_changed() {
        for mut node in panel_query.iter_mut() {
            // Adjust height based on chat state
            if chat_state.is_expanded {
                node.height = px(250.);
            } else {
                node.height = px(40.); // Juste le titre
            }

            // Adjust bottom position based on actions panel visibility
            if is_actions_panel_visible {
                // Actions panel is 150px high + 10px bottom margin
                // Chat has its own 10px bottom margin, so we position at 160px
                // This creates a 10px total gap between the panels
                node.bottom = px(160.);
            } else {
                node.bottom = px(0.);
            }
        }

        for mut visibility in messages_query.iter_mut() {
            *visibility = if chat_state.is_expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        // Hide input container when collapsed
        for mut visibility in input_query.iter_mut() {
            *visibility = if chat_state.is_expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// Handle focus when clicking on the input field
pub fn handle_chat_input_focus(
    mut input_query: Query<(&mut ChatInputState, &Interaction), (Changed<Interaction>, With<ChatInputField>)>,
) {
    for (mut input_state, interaction) in &mut input_query {
        match *interaction {
            Interaction::Pressed => {
                input_state.is_focused = true;
            }
            _ => {}
        }
    }
}

// Handle keyboard input for the chat
pub fn handle_chat_input_keyboard(
    mut input_query: Query<&mut ChatInputState, With<ChatInputField>>,
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut input_state) = input_query.single_mut() {
        if !input_state.is_focused {
            return;
        }

        for event in keyboard_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match &event.logical_key {
                Key::Character(char) => {
                    // Add character to text (limit to 200 chars)
                    if input_state.text.len() < 200 {
                        input_state.text.push_str(char);
                    }
                }
                Key::Space => {
                    if input_state.text.len() < 200 {
                        input_state.text.push(' ');
                    }
                }
                Key::Backspace => {
                    input_state.text.pop();
                }
                Key::Enter => {
                    // Send message
                    if !input_state.text.is_empty() {
                        info!("Sending chat message: {}", input_state.text);
                        // TODO: Actually send the message to the server
                        input_state.text.clear();
                    }
                }
                Key::Escape => {
                    // Unfocus
                    input_state.is_focused = false;
                }
                _ => {}
            }
        }
    }
}

// Update the displayed text in the input field
pub fn update_chat_input_display(
    input_query: Query<&ChatInputState, (Changed<ChatInputState>, With<ChatInputField>)>,
    mut text_query: Query<&mut Text, With<ChatInputText>>,
) {
    if let Ok(input_state) = input_query.single() {
        for mut text in &mut text_query {
            let display_text = if input_state.is_focused {
                format!("{}|", input_state.text)
            } else if input_state.text.is_empty() {
                "Tapez votre message...".to_string()
            } else {
                input_state.text.clone()
            };
            **text = display_text;
        }
    }
}
