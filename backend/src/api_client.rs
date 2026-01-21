use crate::models::PricePoint;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct CoinbaseResponse {
    data: CoinbaseData,
}

#[derive(Deserialize)]
struct CoinbaseData {
    amount: String,
}

#[derive(Deserialize)]
struct CoinbaseCandle {
    // [timestamp, low, high, open, close, volume]
    // We'll use close price
    #[serde(rename = "0")]
    timestamp: i64,
    #[serde(rename = "4")]
    close: String,
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug)]
pub enum ApiError {
    RequestFailed(String),
    ParseError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.coinbase.com/v2".to_string(),
        }
    }

    pub async fn fetch_btc_price(&self) -> Result<PricePoint, ApiError> {
        let url = format!("{}/prices/BTC-USD/spot", self.base_url);

        let response: CoinbaseResponse = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let price = response.data.amount.parse::<f64>()
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        Ok(PricePoint {
            timestamp: Utc::now(),
            asset: "BTC".to_string(),
            price,
        })
    }

    /// Fetch historical candles from Coinbase Pro (granularity in seconds)
    /// Coinbase supports: 60, 300, 900, 3600, 21600, 86400
    /// We'll use 60 (1 minute) for the best granularity
    pub async fn fetch_historical_candles(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        granularity: i64,
    ) -> Result<Vec<(DateTime<Utc>, f64)>, ApiError> {
        // Use Coinbase Advanced Trade API for historical data
        let url = format!(
            "https://api.exchange.coinbase.com/products/BTC-USD/candles?start={}&end={}&granularity={}",
            start.to_rfc3339(),
            end.to_rfc3339(),
            granularity
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", "rust-trading-simulator/1.0")
            .send()
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

        // Get response text first for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| ApiError::ParseError(format!("Failed to get response text: {}", e)))?;

        // Try to parse as JSON array of arrays: [[timestamp, low, high, open, close, volume], ...]
        let candles: Vec<Vec<serde_json::Value>> = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::ParseError(format!("Failed to parse candles. Response: {}. Error: {}", response_text, e)))?;

        let mut result = Vec::new();
        for candle in candles {
            if candle.len() >= 5 {
                let timestamp = candle[0].as_i64()
                    .ok_or_else(|| ApiError::ParseError("Invalid timestamp".to_string()))?;
                let close = candle[4].as_f64()
                    .or_else(|| candle[4].as_str().and_then(|s| s.parse::<f64>().ok()))
                    .ok_or_else(|| ApiError::ParseError("Invalid close price".to_string()))?;

                let dt = DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| ApiError::ParseError("Invalid timestamp conversion".to_string()))?;

                result.push((dt, close));
            }
        }

        // Sort by timestamp (ascending)
        result.sort_by_key(|(dt, _)| *dt);

        Ok(result)
    }

    /// Interpolate between candles to create smooth 5-second data points
    pub fn interpolate_candles(
        candles: Vec<(DateTime<Utc>, f64)>,
        target_interval_secs: i64,
    ) -> Vec<PricePoint> {
        if candles.len() < 2 {
            return candles
                .into_iter()
                .map(|(timestamp, price)| PricePoint {
                    timestamp,
                    asset: "BTC".to_string(),
                    price,
                })
                .collect();
        }

        let mut result = Vec::new();

        for window in candles.windows(2) {
            let (start_time, start_price) = window[0];
            let (end_time, end_price) = window[1];

            let duration = (end_time - start_time).num_seconds();
            let num_points = (duration / target_interval_secs).max(1);

            // Add interpolated points
            for i in 0..num_points {
                let t = i as f64 / num_points as f64;
                let interpolated_price = start_price + (end_price - start_price) * t;
                let interpolated_time = start_time + chrono::Duration::seconds(i * target_interval_secs);

                result.push(PricePoint {
                    timestamp: interpolated_time,
                    asset: "BTC".to_string(),
                    price: interpolated_price,
                });
            }
        }

        // Add the final point
        if let Some((last_time, last_price)) = candles.last() {
            result.push(PricePoint {
                timestamp: *last_time,
                asset: "BTC".to_string(),
                price: *last_price,
            });
        }

        result
    }
}



