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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Trade,
    Deposit,
    Withdrawal,
}

fn default_transaction_type() -> TransactionType {
    TransactionType::Trade
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

    #[serde(default = "default_transaction_type")]
    pub transaction_type: TransactionType,

    #[serde(alias = "asset")]  // Backward compat: old trades had "asset" field
    pub base_asset: Asset,      // Asset being traded (e.g., BTC in BTC/USD)
    #[serde(default = "default_quote_asset")]  // Default to USD if missing
    pub quote_asset: Asset,     // Asset used for pricing (e.g., USD in BTC/USD)
    pub side: TradeSide,
    pub quantity: f64,          // Amount of base asset
    pub price: f64,             // Price in quote asset terms
    pub timestamp: DateTime<Utc>,

    // USD snapshots for portfolio analytics (None if unavailable)
    #[serde(default)]
    pub base_usd_price: Option<f64>,   // USD price of base asset at trade time
    #[serde(default)]
    pub quote_usd_price: Option<f64>,  // USD price of quote asset at trade time
}

fn default_quote_asset() -> String {
    "USD".to_string()
}

impl Trade {
    /// Calculate total cost in quote asset
    pub fn quote_cost(&self) -> f64 {
        self.quantity * self.price
    }

    /// Calculate USD value of the trade (what was spent/received)
    pub fn usd_value(&self) -> Option<f64> {
        self.quote_usd_price.map(|q_usd| self.quote_cost() * q_usd)
    }

    /// Get the asset field for backward compatibility (returns base_asset)
    pub fn asset(&self) -> &str {
        &self.base_asset
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl UserData {
    pub fn new(username: String) -> Self {
        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 10000.0);

        Self {
            username,
            cash_balance: 10000.0,  // Kept for backward compatibility during migration
            asset_balances: balances,
            trade_history: Vec::new(),
        }
    }

    /// Get USD balance (helper for convenience)
    pub fn usd_balance(&self) -> f64 {
        self.asset_balances.get("USD").copied().unwrap_or(self.cash_balance)
    }

    /// Get balance for any asset
    pub fn get_balance(&self, asset: &str) -> f64 {
        if asset == "USD" && !self.asset_balances.contains_key("USD") {
            // Backward compatibility: use cash_balance if USD not in map
            return self.cash_balance;
        }
        self.asset_balances.get(asset).copied().unwrap_or(0.0)
    }

    /// Calculate lifetime deposits (excluding initial seed)
    pub fn lifetime_deposits(&self) -> f64 {
        self.trade_history
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Deposit)
            .map(|t| t.quantity)
            .sum()
    }

    /// Calculate lifetime withdrawals
    pub fn lifetime_withdrawals(&self) -> f64 {
        self.trade_history
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Withdrawal)
            .map(|t| t.quantity)
            .sum()
    }

    /// Calculate lifetime funding (seed + deposits)
    pub fn lifetime_funding(&self) -> f64 {
        10000.0 + self.lifetime_deposits()
    }
}