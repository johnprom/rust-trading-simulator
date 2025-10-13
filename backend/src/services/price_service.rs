use crate::{api_client::ApiClient, state::AppState};
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

pub async fn start_price_polling(state: AppState) {
    let api_client = ApiClient::new();
    let mut interval = time::interval(Duration::from_secs(5));

    info!("Starting price polling service (5s interval)");

    loop {
        interval.tick().await;

        match api_client.fetch_btc_price().await {
            Ok(price_point) => {
                info!("Fetched BTC price: ${:.2}", price_point.price);
                state.add_price_point(price_point).await;
            }
            Err(e) => {
                error!("Failed to fetch price: {}", e);
                // Resiliency: Continue polling despite errors
            }
        }
    }
}
