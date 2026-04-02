/// Interaction systems for the login panel
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ui_text_input::TextInputBuffer;

use crate::{
    networking::client::NetworkClient,
    networking::client::auth_task::{AuthResult, AuthTask},
    states::AuthScreen,
    ui::systems::panels::auth::components::*,
};
use shared::protocol::ClientMessage;

/// System to handle login button click.
/// Sends BOTH tungstenite login (for game data) and HTTP auth (for ConnectToken).
pub fn handle_login_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginSubmitButton>)>,
    family_name_query: Query<&TextInputBuffer, With<LoginFamilyNameInput>>,
    password_query: Query<
        &TextInputBuffer,
        (With<LoginPasswordInput>, Without<LoginFamilyNameInput>),
    >,
    mut error_text_query: Query<(&mut Text, &mut Visibility), With<LoginErrorText>>,
    mut network_client: ResMut<NetworkClient>,
    mut auth_task: ResMut<AuthTask>,
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

            // Path 1: Tungstenite login (for game data, lord spawn, AppState::InGame)
            let message = ClientMessage::LoginWithPassword {
                family_name: family_name.clone(),
                password: password.clone(),
            };
            network_client.send_message(message);
            info!("Login request sent to server (tungstenite)");

            // Path 2: HTTP auth (for ConnectToken → lightyear connection)
            let auth_url = std::env::var("AUTH_HTTP_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
            let url = format!("{}/api/auth/login", auth_url);

            let task = IoTaskPool::get().spawn(async_compat::Compat::new(async move {
                let client = reqwest::Client::new();
                let resp = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "family_name": family_name,
                        "password": password,
                    }))
                    .send()
                    .await
                    .map_err(|e| format!("HTTP error: {}", e))?;

                if !resp.status().is_success() {
                    let error: serde_json::Value = resp
                        .json()
                        .await
                        .unwrap_or(serde_json::json!({"error": "Unknown error"}));
                    return Err(error["error"]
                        .as_str()
                        .unwrap_or("Auth failed")
                        .to_string());
                }

                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("JSON parse error: {}", e))?;

                let player_id = body["player_id"]
                    .as_i64()
                    .ok_or("Missing player_id")?
                    as u64;

                let token_b64 = body["token"].as_str().ok_or("Missing token")?;

                use base64::Engine;
                let token_bytes = base64::engine::general_purpose::STANDARD
                    .decode(token_b64)
                    .map_err(|e| format!("Base64 decode error: {}", e))?;

                use lightyear::netcode::{ConnectToken, CONNECT_TOKEN_BYTES};
                if token_bytes.len() != CONNECT_TOKEN_BYTES {
                    return Err(format!(
                        "Invalid token size: {} (expected {})",
                        token_bytes.len(),
                        CONNECT_TOKEN_BYTES
                    ));
                }

                let connect_token = ConnectToken::try_from_bytes(&token_bytes)
                    .map_err(|e| format!("Token parse error: {}", e))?;

                Ok(AuthResult {
                    player_id,
                    connect_token,
                })
            }));

            auth_task.task = Some(task);
            info!("🔐 Auth request sent to {}", auth_url);
        }
    }
}

/// System to handle "Create account" button click
pub fn handle_to_register_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginToRegisterButton>)>,
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