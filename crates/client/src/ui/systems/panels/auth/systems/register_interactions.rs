/// Interaction systems for the register panel
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ui_text_input::TextInputBuffer;

use crate::{
    networking::client::auth_task::{AuthResult, AuthTask},
    states::AuthScreen,
    ui::systems::panels::auth::components::*,
};

/// System to handle register button click.
/// Sends HTTP auth request to register and get a ConnectToken for lightyear connection.
pub fn handle_register_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterSubmitButton>)>,
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
    mut auth_task: ResMut<AuthTask>,
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

            // HTTP auth register (for ConnectToken → lightyear connection)
            let auth_url = std::env::var("AUTH_HTTP_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
            let url = format!("{}/api/auth/register", auth_url);

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
                        .unwrap_or("Registration failed")
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
            info!("🔐 Register + auth request sent to {}", auth_url);
        }
    }
}

/// System to handle back button click
pub fn handle_back_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterBackButton>)>,
    mut next_auth: ResMut<NextState<AuthScreen>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Switching back to login panel");
            next_auth.set(AuthScreen::Login);
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