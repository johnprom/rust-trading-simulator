use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UserId = String;
pub type Asset = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub asset: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub cash_balance: f64,
    pub asset_balances: HashMap<Asset, f64>,
    pub trade_history: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub user_id: UserId,
    pub asset: Asset,
    pub side: TradeSide,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl UserData {
    pub fn new(username: String) -> Self {
        Self {
            username,
            cash_balance: 10000.0, // Starting balance
            asset_balances: HashMap::new(),
            trade_history: Vec::new(),
        }
    }
}