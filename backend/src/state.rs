use crate::models::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const PRICE_WINDOW_SIZE: usize = 17280; // 24h * 60min * 12 (5s intervals)

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<AppStateInner>>,
}

pub struct AppStateInner {
    pub users: HashMap<UserId, UserData>,
    pub price_window: Vec<PricePoint>,
    // Phase 4: pub bots: HashMap<UserId, BotTaskHandle>,
}

impl AppState {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        
        // Create demo user for MVP
        users.insert(
            "demo_user".to_string(),
            UserData::new("Demo User".to_string()),
        );

        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                users,
                price_window: Vec::with_capacity(PRICE_WINDOW_SIZE),
            })),
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
                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
}
