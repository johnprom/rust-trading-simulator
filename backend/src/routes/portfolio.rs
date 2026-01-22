use crate::{models::UserData, state::AppState};
use axum::{extract::{State, Query}, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PortfolioQuery {
    pub user_id: String,
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Query(query): Query<PortfolioQuery>,
) -> Json<UserData> {
    let user = state
        .get_user(&query.user_id)
        .await
        .unwrap_or_else(|| UserData::new("Unknown".to_string()));
    Json(user)
}
