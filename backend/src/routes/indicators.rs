use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{indicators::{SMA, EMA}, state::AppState};

#[derive(Deserialize)]
pub struct IndicatorQuery {
    pub asset: String,
    pub timeframe: String,      // "1h", "8h", or "24h"
    pub indicators: String,      // comma-separated: "sma_20,sma_50,ema_12"
}

#[derive(Serialize)]
pub struct IndicatorResponse {
    pub asset: String,
    pub timeframe: String,
    pub timestamps: Vec<i64>,
    pub prices: Vec<f64>,
    pub indicators: HashMap<String, Vec<Option<f64>>>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn get_indicators(
    State(state): State<AppState>,
    Query(query): Query<IndicatorQuery>,
) -> Result<Json<IndicatorResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate timeframe - only 1h is supported for now
    if query.timeframe != "1h" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "Indicators are only supported for 1h timeframe. Requested: {}",
                    query.timeframe
                ),
            }),
        ));
    }

    // Get price data from state (1h = 5-second price_window data)
    let state_lock = state.inner.read().await;
    let price_window = &state_lock.price_window;

    // Filter prices for the requested asset
    let asset_prices: Vec<_> = price_window
        .iter()
        .filter(|p| p.asset == query.asset)
        .collect();

    if asset_prices.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("No price data found for asset: {}", query.asset),
            }),
        ));
    }

    // Extract prices and timestamps
    let prices: Vec<f64> = asset_prices.iter().map(|p| p.price).collect();
    let timestamps: Vec<i64> = asset_prices
        .iter()
        .map(|p| p.timestamp.timestamp())
        .collect();

    // Check if we have enough data for indicators
    if prices.len() < 20 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "Insufficient data for indicators. Need at least 20 points, have {}",
                    prices.len()
                ),
            }),
        ));
    }

    // Parse requested indicators
    let requested: Vec<&str> = query.indicators.split(',').map(|s| s.trim()).collect();
    let mut indicators = HashMap::new();

    for indicator_str in requested {
        // Parse indicator format: "sma_20", "ema_12", etc.
        let parts: Vec<&str> = indicator_str.split('_').collect();
        if parts.len() != 2 {
            continue; // Skip malformed indicator strings
        }

        let indicator_type = parts[0];
        let period: usize = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue, // Skip if period is not a valid number
        };

        // Validate period
        if period < 2 || period > 200 {
            continue; // Skip invalid periods
        }

        // Calculate indicator based on type
        let values = match indicator_type {
            "sma" => {
                let sma = SMA::new(period);
                sma.calculate(&prices)
            }
            "ema" => {
                let ema = EMA::new(period);
                ema.calculate(&prices)
            }
            _ => continue, // Skip unknown indicator types
        };

        // Convert NaN to None for JSON serialization
        let values_option: Vec<Option<f64>> = values
            .into_iter()
            .map(|v| if v.is_nan() { None } else { Some(v) })
            .collect();

        indicators.insert(indicator_str.to_string(), values_option);
    }

    Ok(Json(IndicatorResponse {
        asset: query.asset,
        timeframe: query.timeframe,
        timestamps,
        prices,
        indicators,
    }))
}
