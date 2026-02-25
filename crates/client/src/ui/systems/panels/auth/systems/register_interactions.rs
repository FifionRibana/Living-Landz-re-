/// Interaction systems for the register panel
use bevy::prelude::*;
use bevy_ui_text_input::TextInputBuffer;

use crate::{
    networking::client::NetworkClient,
    ui::{
        resources::{PanelEnum, UIState},
        systems::panels::auth::components::*,
    },
};
use shared::protocol::{ClientMessage, ServerMessage};

/// System to handle register button click
pub fn handle_register_button_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterSubmitButton>)>,
    family_name_query: Query<&TextInputBuffer, With<RegisterFamilyNameInput>>,
    password_query: Query<
        &TextInputBuffer,
        (
            With<RegisterPasswordInput>,
            Without<RegisterFamilyNameInput>,
            Without<RegisterPasswordConfirmInput>,
        ),
    >,
    confirm_password_query: Query<
        &TextInputBuffer,
        (
            With<RegisterPasswordConfirmInput>,
            Without<RegisterFamilyNameInput>,
            Without<RegisterPasswordInput>,
        ),
    >,
    mut error_text_query: Query<
        (&mut Text, &mut Visibility),
        (With<RegisterErrorText>, Without<RegisterSuccessText>),
    >,
    mut success_text_query: Query<
        (&mut Text, &mut Visibility),
        (With<RegisterSuccessText>, Without<RegisterErrorText>),
    >,
    mut network_client: ResMut<NetworkClient>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Hide previous messages
            if let Ok((_, mut visibility)) = error_text_query.single_mut() {
                *visibility = Visibility::Hidden;
            }
            if let Ok((_, mut visibility)) = success_text_query.single_mut() {
                *visibility = Visibility::Hidden;
            }

            // Get input values
            let family_name = family_name_query
                .single()
                .map(|b| b.get_text().to_string())
                .unwrap_or_default();

            let password = password_query
                .single()
                .map(|b| b.get_text().to_string())
                .unwrap_or_default();

            let confirm_password = confirm_password_query
                .single()
                .map(|b| b.get_text().to_string())
                .unwrap_or_default();

            // Validate inputs
            if family_name.trim().is_empty() {
                if let Ok((mut text, mut visibility)) = error_text_query.single_mut() {
                    **text = "Le nom de famille est requis".to_string();
                    *visibility = Visibility::Visible;
                }
                return;
            }

            if password.is_empty() {
                if let Ok((mut text, mut visibility)) = error_text_query.single_mut() {
                    **text = "Le mot de passe est requis".to_string();
                    *visibility = Visibility::Visible;
                }
                return;
            }

            // Validate password match
            if password != confirm_password {
                if let Ok((mut text, mut visibility)) = error_text_query.single_mut() {
                    **text = "Les mots de passe ne correspondent pas".to_string();
                    *visibility = Visibility::Visible;
                }
                return;
            }

            // Client-side validation
            if let Err(e) = shared::auth::validate_family_name(&family_name) {
                if let Ok((mut text, mut visibility)) = error_text_query.single_mut() {
                    **text = e;
                    *visibility = Visibility::Visible;
                }
                return;
            }

            let requirements = shared::auth::PasswordRequirements::default();
            if let Err(e) = shared::auth::validate_password(&password, &requirements) {
                if let Ok((mut text, mut visibility)) = error_text_query.single_mut() {
                    **text = e;
                    *visibility = Visibility::Visible;
                }
                return;
            }

            // Send register message to server
            let message = ClientMessage::RegisterAccount {
                family_name,
                password,
            };

            network_client.send_message(message);
            info!("Registration request sent to server");
        }
    }
}

/// System to handle back button click
pub fn handle_back_button_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterBackButton>)>,
    mut ui_state: ResMut<UIState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Switching back to login panel");
            ui_state.switch_to(PanelEnum::LoginPanel);
        }
    }
}

// Note: Registration response handling is now in handle_server_message in networking/client/handlers.rs

/// System to handle button hover effects for register panel
pub fn handle_register_button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            Or<(With<RegisterSubmitButton>, With<RegisterBackButton>)>,
        ),
    >,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgb(0.25, 0.4, 0.2).into(); // Darker green on press
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.4, 0.55, 0.35).into(); // Lighter green on hover
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.35, 0.5, 0.3).into(); // Default green
            }
        }
    }
}
