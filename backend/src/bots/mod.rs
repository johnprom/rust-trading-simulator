use crate::models::PricePoint;

pub mod naive_momentum;

/// Core trait that all trading bots must implement
pub trait TradingBot: Send {
    /// Called every 60 seconds with market context
    /// Bot examines context, updates internal state, and returns a decision
    fn tick(&mut self, ctx: &BotContext) -> BotDecision;

    /// Bot display name for UI
    fn name(&self) -> &str;
}

/// Immutable context passed to bot each tick
#[derive(Debug, Clone)]
pub struct BotContext {
    /// Raw 5s price data from polling window
    /// Most recent prices (e.g., last 720 points = 1 hour)
    pub price_window: Vec<PricePoint>,

    /// Current balances
    pub base_balance: f64,
    pub quote_balance: f64,

    /// Current market price (most recent in window)
    pub current_price: f64,

    /// Trading pair info
    pub base_asset: String,
    pub quote_asset: String,

    /// How many ticks since bot started (0-indexed)
    pub tick_count: u64,
}

/// Decision returned by bot after each tick
#[derive(Debug, Clone, PartialEq)]
pub enum BotDecision {
    /// Take no action this tick
    DoNothing,

    /// Buy worth X in quote asset (e.g., "buy $100 worth of BTC")
    /// Framework converts to base quantity using current price
    Buy { quote_amount: f64 },

    /// Sell worth X in quote asset (e.g., "sell $100 worth of BTC")
    /// Framework converts to base quantity using current price
    Sell { quote_amount: f64 },
}

/// Bot template helper: maintains recent price history
/// Useful for bots that need to track price movements
#[derive(Debug, Clone)]
pub struct PriceHistory {
    prices: Vec<f64>,
    max_size: usize,
}

impl PriceHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            prices: Vec::new(),
            max_size,
        }
    }

    /// Add a new price (automatically maintains max_size)
    pub fn push(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.max_size {
            self.prices.remove(0);
        }
    }

    /// Get all tracked prices
    pub fn prices(&self) -> &[f64] {
        &self.prices
    }

    /// Get the most recent N prices (or fewer if not enough data)
    pub fn last_n(&self, n: usize) -> &[f64] {
        let start = self.prices.len().saturating_sub(n);
        &self.prices[start..]
    }

    /// Check if we have at least N prices
    pub fn has_at_least(&self, n: usize) -> bool {
        self.prices.len() >= n
    }

    /// Number of prices tracked
    pub fn len(&self) -> usize {
        self.prices.len()
    }
}
