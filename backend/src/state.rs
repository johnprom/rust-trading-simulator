use crate::models::*;
use crate::db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

const PRICE_WINDOW_SIZE: usize = 17280; // 24h * 60min * 12 (5s intervals) - high frequency
const CANDLE_WINDOW_SIZE: usize = 288;  // 24h * 12 (5min intervals) - low frequency
const OHLC_CANDLE_1M_SIZE: usize = 60;  // 1 hour of 1-minute candles for 1h view
const OHLC_CANDLE_5M_SIZE: usize = 288; // 24 hours of 5-minute candles for 8h/24h views

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<AppStateInner>>,
    pub db: Database,
}

/// Bot instance information for a running bot
pub struct BotInstance {
    pub bot_name: String,
    pub trading_pair: (String, String), // (base_asset, quote_asset)
    pub stoploss_amount: f64,
    pub initial_portfolio_value_usd: f64, // Portfolio value when bot started
    pub task_handle: JoinHandle<()>,
}

pub struct AppStateInner {
    pub users: HashMap<UserId, UserData>,
    pub price_window: Vec<PricePoint>,     // High-frequency: 5-second data (last 1-2 hours of real data)
    pub candle_window: Vec<PricePoint>,    // Low-frequency: 5-minute candles (24 hours of historical data)
    pub ohlc_candles_1m: Vec<Candle>,      // 1-minute OHLC candles for 1h candlestick view
    pub ohlc_candles_5m: Vec<Candle>,      // 5-minute OHLC candles for 8h/24h candlestick views
    pub active_bots: HashMap<UserId, BotInstance>, // One bot per user maximum
}

impl AppState {
    pub async fn new(db: Database) -> Self {
        // Delete demo user from database if it exists (demo user should reset on restart)
        if let Err(e) = crate::db::queries::delete_user(db.pool(), &"demo_user".to_string()).await {
            tracing::debug!("No demo user to delete: {}", e);
        }

        // Load authenticated users from database (demo_user is excluded)
        let mut users = crate::db::queries::load_all_users(db.pool())
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load users from database: {}", e);
                HashMap::new()
            });

        // Always create fresh demo user in memory only (not persisted)
        let demo_user = UserData::new("Demo User".to_string());
        users.insert("demo_user".to_string(), demo_user);

        tracing::info!("Initialized with {} authenticated users + demo user", users.len() - 1);

        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                users,
                price_window: Vec::with_capacity(PRICE_WINDOW_SIZE),
                candle_window: Vec::with_capacity(CANDLE_WINDOW_SIZE),
                ohlc_candles_1m: Vec::with_capacity(OHLC_CANDLE_1M_SIZE * 2), // BTC + ETH
                ohlc_candles_5m: Vec::with_capacity(OHLC_CANDLE_5M_SIZE * 2), // BTC + ETH
                active_bots: HashMap::new(),
            })),
            db,
        }
    }

    pub async fn add_price_point(&self, point: PricePoint) {
        let mut state = self.inner.write().await;
        state.price_window.push(point);
        
        // Maintain sliding window (24h)
        if state.price_window.len() > PRICE_WINDOW_SIZE {
            state.price_window.remove(0);
        }
    }

    pub async fn get_latest_price(&self, asset: &str) -> Option<f64> {
        let state = self.inner.read().await;
        state.price_window
            .iter()
            .rev()
            .find(|p| p.asset == asset)
            .map(|p| p.price)
    }

    /// Get price for a trading pair (base/quote)
    /// For USD pairs: returns direct price (e.g., BTC-USD)
    /// For cross pairs: calculates via USD (e.g., BTC-ETH = BTC-USD / ETH-USD)
    pub async fn get_pair_price(&self, base: &str, quote: &str) -> Option<f64> {
        if quote == "USD" {
            // Direct USD price
            self.get_latest_price(base).await
        } else if base == "USD" {
            // Inverted (e.g., USD/BTC = 1 / BTC-USD)
            self.get_latest_price(quote).await.map(|p| 1.0 / p)
        } else {
            // Cross-pair calculation: BTC/ETH = BTC-USD / ETH-USD
            let base_usd = self.get_latest_price(base).await?;
            let quote_usd = self.get_latest_price(quote).await?;
            Some(base_usd / quote_usd)
        }
    }

    pub async fn get_price_window(&self, asset: &str, limit: usize) -> Vec<PricePoint> {
        let state = self.inner.read().await;
        state.price_window
            .iter()
            .filter(|p| p.asset == asset)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Add a 5-minute candle to the candle window (for longer-term data)
    pub async fn add_candle(&self, point: PricePoint) {
        let mut state = self.inner.write().await;
        let asset = point.asset.clone();
        state.candle_window.push(point);

        // Maintain sliding window per asset (24h of 5-minute candles = 288 points per asset)
        let asset_count = state.candle_window.iter().filter(|p| p.asset == asset).count();
        if asset_count > CANDLE_WINDOW_SIZE {
            // Find and remove the oldest candle for this specific asset
            if let Some(index) = state.candle_window.iter().position(|p| p.asset == asset) {
                state.candle_window.remove(index);
            }
        }
    }

    /// Get 5-minute candles for a specific asset
    pub async fn get_candle_window(&self, asset: &str, limit: usize) -> Vec<PricePoint> {
        let state = self.inner.read().await;
        state.candle_window
            .iter()
            .filter(|p| p.asset == asset)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Add 1-minute OHLC candle (for 1h candlestick view)
    pub async fn add_ohlc_candle_1m(&self, candle: Candle) {
        let mut state = self.inner.write().await;
        let asset = candle.asset.clone();
        state.ohlc_candles_1m.push(candle);

        // Maintain window per asset (60 candles per asset)
        let asset_count = state.ohlc_candles_1m.iter().filter(|c| c.asset == asset).count();
        if asset_count > OHLC_CANDLE_1M_SIZE {
            if let Some(index) = state.ohlc_candles_1m.iter().position(|c| c.asset == asset) {
                state.ohlc_candles_1m.remove(index);
            }
        }
    }

    /// Get 1-minute OHLC candles for a specific asset
    pub async fn get_ohlc_candles_1m(&self, asset: &str, limit: usize) -> Vec<Candle> {
        let state = self.inner.read().await;
        state.ohlc_candles_1m
            .iter()
            .filter(|c| c.asset == asset)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Add 5-minute OHLC candle (for 8h/24h candlestick views)
    pub async fn add_ohlc_candle_5m(&self, candle: Candle) {
        let mut state = self.inner.write().await;
        let asset = candle.asset.clone();
        state.ohlc_candles_5m.push(candle);

        // Maintain window per asset (288 candles per asset)
        let asset_count = state.ohlc_candles_5m.iter().filter(|c| c.asset == asset).count();
        if asset_count > OHLC_CANDLE_5M_SIZE {
            if let Some(index) = state.ohlc_candles_5m.iter().position(|c| c.asset == asset) {
                state.ohlc_candles_5m.remove(index);
            }
        }
    }

    /// Get 5-minute OHLC candles for a specific asset
    pub async fn get_ohlc_candles_5m(&self, asset: &str, limit: usize) -> Vec<Candle> {
        let state = self.inner.read().await;
        state.ohlc_candles_5m
            .iter()
            .filter(|c| c.asset == asset)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub async fn get_user(&self, user_id: &UserId) -> Option<UserData> {
        let state = self.inner.read().await;
        state.users.get(user_id).cloned()
    }

    pub async fn update_user<F>(&self, user_id: &UserId, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut UserData),
    {
        let mut state = self.inner.write().await;
        match state.users.get_mut(user_id) {
            Some(user) => {
                f(user);

                // Persist to database (but NOT demo_user - it's memory-only)
                if user_id != "demo_user" {
                    let user_clone = user.clone();
                    let db_pool = self.db.pool().clone();
                    let user_id_clone = user_id.clone();

                    // Spawn task to save to DB without blocking
                    tokio::spawn(async move {
                        if let Err(e) = crate::db::queries::save_user(&db_pool, &user_id_clone, &user_clone).await {
                            tracing::error!("Failed to persist user {} to database: {}", user_id_clone, e);
                        }
                    });
                }

                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
}
