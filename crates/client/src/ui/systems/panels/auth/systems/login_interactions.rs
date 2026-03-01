/// Interaction systems for the login panel
use bevy::prelude::*;
use bevy_ui_text_input::TextInputBuffer;

use crate::{
    networking::client::NetworkClient,
    states::AuthScreen,
    ui::systems::panels::auth::components::*,
};
use shared::protocol::ClientMessage;

/// System to handle login button click
pub fn handle_login_button_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginSubmitButton>)>,
    family_name_query: Query<&TextInputBuffer, With<LoginFamilyNameInput>>,
    password_query: Query<
        &TextInputBuffer,
        (With<LoginPasswordInput>, Without<LoginFamilyNameInput>),
    >,
    mut error_text_query: Query<(&mut Text, &mut Visibility), With<LoginErrorText>>,
    mut network_client: ResMut<NetworkClient>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Get input values
            let family_name = family_name_query
                .single()
                .map(|b| b.get_text().to_string())
                .unwrap_or_default();

            let password = password_query
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

            // Hide error message if validation passed
            if let Ok((_, mut visibility)) = error_text_query.single_mut() {
                *visibility = Visibility::Hidden;
            }

            // Send login message to server
            let message = ClientMessage::LoginWithPassword {
                family_name,
                password,
            };

            network_client.send_message(message);
            info!("Login request sent to server");
        }
    }
}

/// System to handle "Create account" button click
pub fn handle_to_register_button_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginToRegisterButton>)>,
    mut next_auth: ResMut<NextState<AuthScreen>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Switching to register panel");
            next_auth.set(AuthScreen::Register);
        }
    }
}

// Note: Login response handling is now in handle_server_message in networking/client/handlers.rs

/// System to handle button hover effects for login panel
pub fn handle_login_button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            Or<(With<LoginSubmitButton>, With<LoginToRegisterButton>)>,
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

/// Test button: jump directly to character creation screen
pub fn handle_test_character_creation_click(
    query: Query<&Interaction, (Changed<Interaction>, With<TestCharacterCreationButton>)>,
    mut next_app_state: ResMut<NextState<crate::states::AppState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("[TEST] Jumping to character creation");
            next_app_state.set(crate::states::AppState::CharacterCreation);
        }
    }
}

/// Test button: jump directly to coat of arms screen
pub fn handle_test_coat_of_arms_click(
    query: Query<&Interaction, (Changed<Interaction>, With<TestCoatOfArmsButton>)>,
    mut next_app_state: ResMut<NextState<crate::states::AppState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("[TEST] Jumping to coat of arms creation");
            next_app_state.set(crate::states::AppState::CoatOfArmsCreation);
        }
    }
}

/// Hover effect for test buttons
pub fn handle_test_button_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            Or<(With<TestCharacterCreationButton>, With<TestCoatOfArmsButton>)>,
        ),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => {
                BackgroundColor(Color::srgba(0.35, 0.28, 0.16, 0.9))
            }
            Interaction::None => BackgroundColor(Color::srgba(0.25, 0.20, 0.12, 0.8)),
        };
    }
}