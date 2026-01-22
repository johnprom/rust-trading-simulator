use crate::state::AppState;
use axum::{extract::{State, Query}, Json};
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub struct AssetQuery {
    pub asset: Option<String>,
}

pub async fn get_price(
    State(state): State<AppState>,
    Query(query): Query<AssetQuery>,
) -> Json<PriceResponse> {
    let asset = query.asset.unwrap_or_else(|| "BTC".to_string());
    let price = state.get_latest_price(&asset).await.unwrap_or(0.0);
    Json(PriceResponse {
        asset: asset.clone(),
        price,
    })
}

pub async fn get_price_history(
    State(state): State<AppState>,
    Query(query): Query<AssetQuery>,
) -> Json<PriceHistoryResponse> {
    let asset = query.asset.unwrap_or_else(|| "BTC".to_string());
    let price_window = state.get_price_window(&asset, 720).await; // Last hour (720 points at 5s intervals)

    let prices: Vec<PricePoint> = price_window
        .iter()
        .map(|p| PricePoint {
            timestamp: p.timestamp.timestamp(),
            price: p.price,
        })
        .collect();

    Json(PriceHistoryResponse {
        asset: asset.clone(),
        prices,
    })
}
