use crate::models::*;
use crate::db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const PRICE_WINDOW_SIZE: usize = 17280; // 24h * 60min * 12 (5s intervals)

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<AppStateInner>>,
    pub db: Database,
}

pub struct AppStateInner {
    pub users: HashMap<UserId, UserData>,
    pub price_window: Vec<PricePoint>,
    // Phase 4: pub bots: HashMap<UserId, BotTaskHandle>,
}

impl AppState {
    pub async fn new(db: Database) -> Self {
        // Load users from database
        let users = crate::db::queries::load_all_users(db.pool())
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load users from database: {}", e);
                HashMap::new()
            });

        // If no users exist, create demo user
        // TODO: After Phase 2 authentication is complete, change this behavior:
        // - Demo user should reset to $10,000 on every restart (not persist)
        // - Only authenticated users should persist
        // - Consider: delete demo_user from DB on startup if exists, or use separate in-memory demo
        let users = if users.is_empty() {
            let mut users = HashMap::new();
            let demo_user = UserData::new("Demo User".to_string());
            users.insert("demo_user".to_string(), demo_user.clone());

            // Save demo user to database
            if let Err(e) = crate::db::queries::save_user(db.pool(), &"demo_user".to_string(), &demo_user).await {
                tracing::error!("Failed to save demo user: {}", e);
            }

            users
        } else {
            users
        };

        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                users,
                price_window: Vec::with_capacity(PRICE_WINDOW_SIZE),
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

                // Persist to database
                let user_clone = user.clone();
                let db_pool = self.db.pool().clone();
                let user_id_clone = user_id.clone();

                // Spawn task to save to DB without blocking
                tokio::spawn(async move {
                    if let Err(e) = crate::db::queries::save_user(&db_pool, &user_id_clone, &user_clone).await {
                        tracing::error!("Failed to persist user {} to database: {}", user_id_clone, e);
                    }
                });

                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
}
