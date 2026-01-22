use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::services::auth_service::{self, AuthError};
use crate::db::queries;
use crate::models::{UserId, UserData};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: UserId,
    pub username: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate new user ID
    let user_id = auth_service::generate_user_id();

    // Create user in database
    match queries::create_user(
        state.db.pool(),
        &user_id,
        &payload.username,
        &payload.password,
    )
    .await
    {
        Ok(_) => {
            // Also add user to in-memory state
            let user_data = UserData::new(payload.username.clone());
            let mut inner_state = state.inner.write().await;
            inner_state.users.insert(user_id.clone(), user_data);
            drop(inner_state);

            Ok(Json(AuthResponse {
                user_id,
                username: payload.username,
            }))
        }
        Err(AuthError::UserAlreadyExists) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Username already exists".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            }),
        )),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match queries::verify_user_credentials(state.db.pool(), &payload.username, &payload.password)
        .await
    {
        Ok(user_id) => Ok(Json(AuthResponse {
            user_id,
            username: payload.username,
        })),
        Err(AuthError::InvalidCredentials) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Login failed: {}", e),
            }),
        )),
    }
}

#[derive(Serialize)]
pub struct UserInfoResponse {
    pub user_id: UserId,
    pub username: String,
    pub cash_balance: f64,
}

pub async fn get_me(
    State(state): State<AppState>,
    user_id: String,
) -> Result<Json<UserInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.get_user(&user_id).await {
        Some(user) => Ok(Json(UserInfoResponse {
            user_id,
            username: user.username,
            cash_balance: user.cash_balance,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
            }),
        )),
    }
}
