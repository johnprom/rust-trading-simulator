use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub asset: String,
    pub price: f64,
}

pub async fn get_price(State(state): State<AppState>) -> Json<PriceResponse> {
    let price = state.get_latest_price("BTC").await.unwrap_or(0.0);
    Json(PriceResponse {
        asset: "BTC".to_string(),
        price,
    })
}
