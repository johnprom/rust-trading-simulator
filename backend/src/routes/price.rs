use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub asset: String,
    pub price: f64,
}

#[derive(Serialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
}

#[derive(Serialize)]
pub struct PriceHistoryResponse {
    pub asset: String,
    pub prices: Vec<PricePoint>,
}

pub async fn get_price(State(state): State<AppState>) -> Json<PriceResponse> {
    let price = state.get_latest_price("BTC").await.unwrap_or(0.0);
    Json(PriceResponse {
        asset: "BTC".to_string(),
        price,
    })
}

pub async fn get_price_history(State(state): State<AppState>) -> Json<PriceHistoryResponse> {
    let price_window = state.get_price_window("BTC", 720).await; // Last hour (720 points at 5s intervals)

    let prices: Vec<PricePoint> = price_window
        .iter()
        .map(|p| PricePoint {
            timestamp: p.timestamp.timestamp(),
            price: p.price,
        })
        .collect();

    Json(PriceHistoryResponse {
        asset: "BTC".to_string(),
        prices,
    })
}
