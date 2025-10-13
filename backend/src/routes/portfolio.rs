use crate::{models::UserData, state::AppState};
use axum::{extract::State, Json};

pub async fn get_portfolio(State(state): State<AppState>) -> Json<UserData> {
    let user = state
        .get_user(&"demo_user".to_string())
        .await
        .unwrap_or_else(|| UserData::new("Demo User".to_string()));
    Json(user)
}
