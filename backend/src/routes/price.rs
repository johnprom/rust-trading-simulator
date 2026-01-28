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
    pub timeframe: Option<String>, // "1h", "8h", or "24h"
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
    let timeframe = query.timeframe.as_deref().unwrap_or("1h");

    tracing::info!(
        "Price history request: asset={}, timeframe={}",
        asset,
        timeframe
    );

    // For 1h: use high-frequency 5-second data (720 points)
    // For 8h/24h: use low-frequency 5-minute candles (96 or 288 points)
    let prices: Vec<PricePoint> = match timeframe {
        "1h" => {
            let price_window = state.get_price_window(&asset, 720).await;
            price_window
                .iter()
                .map(|p| PricePoint {
                    timestamp: p.timestamp.timestamp(),
                    price: p.price,
                })
                .collect()
        }
        "8h" => {
            // 8 hours of 5-minute candles = 96 candles
            let candle_window = state.get_candle_window(&asset, 96).await;
            candle_window
                .iter()
                .map(|p| PricePoint {
                    timestamp: p.timestamp.timestamp(),
                    price: p.price,
                })
                .collect()
        }
        "24h" => {
            // 24 hours of 5-minute candles = 288 candles
            let candle_window = state.get_candle_window(&asset, 288).await;
            candle_window
                .iter()
                .map(|p| PricePoint {
                    timestamp: p.timestamp.timestamp(),
                    price: p.price,
                })
                .collect()
        }
        _ => {
            // Default to 1h
            let price_window = state.get_price_window(&asset, 720).await;
            price_window
                .iter()
                .map(|p| PricePoint {
                    timestamp: p.timestamp.timestamp(),
                    price: p.price,
                })
                .collect()
        }
    };

    tracing::info!(
        "Returning {} data points for {}/{}",
        prices.len(),
        asset,
        timeframe
    );

    Json(PriceHistoryResponse {
        asset: asset.clone(),
        prices,
    })
}
