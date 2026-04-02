use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use base64::Engine;
use lightyear::netcode::ConnectToken;
use serde::{Deserialize, Serialize};

use crate::auth::password;
use crate::database::client::DatabaseTables;

// ─── State ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AuthState {
    pub db_tables: Arc<DatabaseTables>,
    pub game_server_addr: std::net::SocketAddr,
}

// ─── Request / Response types ────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub family_name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub family_name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub player_id: i64,
}

#[derive(Serialize)]
pub struct AuthError {
    pub error: String,
}

// ─── Helpers ─────────────────────────────────────────────────────────

type AuthResult = Result<Json<AuthResponse>, (StatusCode, Json<AuthError>)>;

fn auth_error(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<AuthError>) {
    (
        status,
        Json(AuthError {
            error: msg.into(),
        }),
    )
}

fn generate_token(
    game_server_addr: std::net::SocketAddr,
    player_id: i64,
) -> Result<String, (StatusCode, Json<AuthError>)> {
    let token = ConnectToken::build(
        game_server_addr,
        shared::protocol::netcode_config::PROTOCOL_ID,
        player_id as u64,
        shared::protocol::netcode_config::private_key(),
    )
    .generate()
    .map_err(|e| {
        tracing::error!("Failed to generate ConnectToken: {}", e);
        auth_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate authentication token",
        )
    })?;

    let token_bytes = token.try_into_bytes().map_err(|e| {
        tracing::error!("Failed to serialize ConnectToken: {}", e);
        auth_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize authentication token",
        )
    })?;

    Ok(base64::engine::general_purpose::STANDARD.encode(token_bytes))
}

// ─── Login ───────────────────────────────────────────────────────────

pub async fn login(
    State(state): State<AuthState>,
    Json(body): Json<LoginRequest>,
) -> AuthResult {
    // 1. Fetch player by family name
    let player = match shared::types::game::methods::get_player_by_family_name(
        &state.db_tables.pool,
        &body.family_name,
    )
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            tracing::warn!("Login failed: player '{}' not found", body.family_name);
            return Err(auth_error(StatusCode::UNAUTHORIZED, "Invalid credentials"));
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ));
        }
    };

    // 2. Check password hash exists
    let password_hash = match &player.password_hash {
        Some(hash) => hash,
        None => {
            tracing::warn!(
                "Player '{}' has no password hash (account migration required)",
                body.family_name
            );
            return Err(auth_error(
                StatusCode::UNAUTHORIZED,
                "Account requires password migration",
            ));
        }
    };

    // 3. Verify password
    match password::verify_password(&body.password, password_hash) {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!("Login failed: wrong password for '{}'", body.family_name);
            return Err(auth_error(StatusCode::UNAUTHORIZED, "Invalid credentials"));
        }
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password verification error",
            ));
        }
    }

    // 4. Update last_login_at
    if let Err(e) =
        shared::types::game::methods::update_last_login(&state.db_tables.pool, player.id).await
    {
        tracing::warn!(
            "Failed to update last_login_at for player {}: {}",
            player.id,
            e
        );
    }

    // 5. Generate ConnectToken
    let token_b64 = generate_token(state.game_server_addr, player.id)?;

    tracing::info!(
        "Player '{}' (ID: {}) authenticated via HTTP, token issued",
        body.family_name, player.id
    );

    Ok(Json(AuthResponse {
        token: token_b64,
        player_id: player.id,
    }))
}

// ─── Register ────────────────────────────────────────────────────────

pub async fn register(
    State(state): State<AuthState>,
    Json(body): Json<RegisterRequest>,
) -> AuthResult {
    // 1. Validate family name
    if let Err(e) = shared::auth::validate_family_name(&body.family_name) {
        tracing::warn!("Invalid family name during registration: {}", e);
        return Err(auth_error(StatusCode::BAD_REQUEST, e));
    }

    // 2. Validate password
    let requirements = shared::auth::PasswordRequirements::default();
    if let Err(e) = shared::auth::validate_password(&body.password, &requirements) {
        tracing::warn!("Invalid password during registration: {}", e);
        return Err(auth_error(StatusCode::BAD_REQUEST, e));
    }

    // 3. Check if family name already exists
    match shared::types::game::methods::get_player_by_family_name(
        &state.db_tables.pool,
        &body.family_name,
    )
    .await
    {
        Ok(Some(_)) => {
            tracing::warn!(
                "Registration failed: family name already exists: {}",
                body.family_name
            );
            return Err(auth_error(
                StatusCode::CONFLICT,
                "This family name is already taken",
            ));
        }
        Ok(None) => {} // good, name is available
        Err(e) => {
            tracing::error!("Database error during registration: {}", e);
            return Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ));
        }
    }

    // 4. Hash the password
    let password_hash = match password::hash_password(&body.password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password processing error",
            ));
        }
    };

    // 5. Create player
    let player = match shared::types::game::methods::create_player_with_password(
        &state.db_tables.pool,
        &body.family_name,
        1, // Default language_id
        "default_location",
        None, // No motto
        &password_hash,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create player: {}", e);
            return Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create account: {}", e),
            ));
        }
    };

    // 6. Generate ConnectToken
    let token_b64 = generate_token(state.game_server_addr, player.id)?;

    tracing::info!(
        "Account registered: '{}' (ID: {}), token issued",
        body.family_name, player.id
    );

    Ok(Json(AuthResponse {
        token: token_b64,
        player_id: player.id,
    }))
}
