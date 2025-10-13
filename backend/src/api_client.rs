use crate::models::PricePoint;
use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize)]
struct CoinbaseResponse {
    data: CoinbaseData,
}

#[derive(Deserialize)]
struct CoinbaseData {
    amount: String,
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
}



